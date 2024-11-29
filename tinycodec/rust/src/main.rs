extern crate bitstream_io as bitstream;
extern crate video_rs as video;

use anyhow::Result;
use bitstream::huffman::compile_write_tree;
use bitstream::huffman::WriteHuffmanTree;
use bitstream::BigEndian;
use bitstream::BitWrite;
use bitstream::BitWriter;
use bitstream::HuffmanWrite;
use clap::Parser;
use clap::Subcommand;
use kdam::tqdm;
use kdam::TqdmIterator;
use kdam::TqdmParallelIterator;
use ndarray::prelude::*;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::mpsc;
use video::decode::Decoder;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Encode {
        #[arg(value_name = "infile")]
        infile: String,
        #[arg(value_name = "outfile")]
        outfile: String,
    },
    Decode {
        #[arg(value_name = "infile")]
        infile: String,
        #[arg(value_name = "outfile")]
        outfile: String,
    },
}

struct HuffmanTable {
    dc: WriteHuffmanTree<BigEndian, i64>,
    ac: WriteHuffmanTree<BigEndian, (i64, i64)>,
    value: WriteHuffmanTree<BigEndian, i64>,
}

impl HuffmanTable {
    fn generate_value_table() -> Vec<(i64, Vec<u8>)> {
        let mut table = Vec::new();

        for value in -2047i64..2048i64 {
            let mut v = Vec::new();
            let size = (64 - value.abs().leading_zeros()) as i64;

            for bit in (0..size).rev() {
                v.push(((value >> bit) & 1) as u8);
            }

            table.push((value, v));
        }

        table
    }

    fn new() -> Result<Self> {
        let dc = compile_write_tree::<BigEndian, i64>(vec![
            (0, vec![0, 0]),
            (1, vec![0, 1, 0]),
            (2, vec![0, 1, 1]),
            (3, vec![1, 0, 0]),
            (4, vec![1, 0, 1]),
            (5, vec![1, 1, 0]),
            (6, vec![1, 1, 1, 0]),
            (7, vec![1, 1, 1, 1, 0]),
            (8, vec![1, 1, 1, 1, 1, 0]),
            (9, vec![1, 1, 1, 1, 1, 1, 0]),
            (10, vec![1, 1, 1, 1, 1, 1, 1, 0]),
            (11, vec![1, 1, 1, 1, 1, 1, 1, 1, 0]),
        ])?;

        let ac = compile_write_tree::<BigEndian, (i64, i64)>(vec![
            ((0, 0), vec![1, 0, 1, 0]),
            ((0, 1), vec![0, 0]),
            ((0, 2), vec![0, 1]),
            ((0, 3), vec![1, 0, 0]),
            ((0, 4), vec![1, 0, 1, 1]),
            ((0, 5), vec![1, 1, 0, 1, 0]),
            ((0, 6), vec![1, 1, 1, 1, 0, 0, 0]),
            ((0, 7), vec![1, 1, 1, 1, 1, 0, 0, 0]),
            ((0, 8), vec![1, 1, 1, 1, 1, 1, 0, 1, 1, 0]),
            ((0, 9), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0]),
            (
                (0, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1],
            ),
            ((1, 1), vec![1, 1, 0, 0]),
            ((1, 2), vec![1, 1, 0, 1, 1]),
            ((1, 3), vec![1, 1, 1, 1, 0, 0, 1]),
            ((1, 4), vec![1, 1, 1, 1, 1, 0, 1, 1, 0]),
            ((1, 5), vec![1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0]),
            ((1, 6), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0]),
            ((1, 7), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1]),
            ((1, 8), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0]),
            ((1, 9), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1]),
            (
                (1, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0],
            ),
            ((2, 1), vec![1, 1, 1, 0, 0]),
            ((2, 2), vec![1, 1, 1, 1, 1, 0, 0, 1]),
            ((2, 3), vec![1, 1, 1, 1, 1, 1, 0, 1, 1, 1]),
            ((2, 4), vec![1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0]),
            ((2, 5), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1]),
            ((2, 6), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 0]),
            ((2, 7), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1]),
            ((2, 8), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0]),
            ((2, 9), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 1]),
            (
                (2, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0],
            ),
            ((3, 1), vec![1, 1, 1, 0, 1, 0]),
            ((3, 2), vec![1, 1, 1, 1, 1, 0, 1, 1, 1]),
            ((3, 3), vec![1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1]),
            ((3, 4), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1]),
            ((3, 5), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0]),
            ((3, 6), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1]),
            ((3, 7), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0]),
            ((3, 8), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1]),
            ((3, 9), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0]),
            (
                (3, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 0, 1],
            ),
            ((4, 1), vec![1, 1, 1, 0, 1, 1]),
            ((4, 2), vec![1, 1, 1, 1, 1, 1, 1, 0, 0, 0]),
            ((4, 3), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 1, 0]),
            ((4, 4), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1]),
            ((4, 5), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0]),
            ((4, 6), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1]),
            ((4, 7), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 0]),
            ((4, 8), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1]),
            ((4, 9), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0]),
            (
                (4, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1],
            ),
            ((5, 1), vec![1, 1, 1, 1, 0, 1, 0]),
            ((5, 2), vec![1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1]),
            ((5, 3), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0]),
            ((5, 4), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1]),
            ((5, 5), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0]),
            ((5, 6), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1]),
            ((5, 7), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0]),
            ((5, 8), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1]),
            ((5, 9), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0]),
            (
                (5, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1],
            ),
            ((6, 1), vec![1, 1, 1, 1, 0, 1, 1]),
            ((6, 2), vec![1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0]),
            ((6, 3), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 1, 0]),
            ((6, 4), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 1, 1]),
            ((6, 5), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 0]),
            ((6, 6), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 1]),
            ((6, 7), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 0]),
            ((6, 8), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1]),
            ((6, 9), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0]),
            (
                (6, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 1],
            ),
            ((7, 1), vec![1, 1, 1, 1, 1, 0, 1, 0]),
            ((7, 2), vec![1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1]),
            ((7, 3), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0]),
            ((7, 4), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1]),
            ((7, 5), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0]),
            ((7, 6), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1]),
            ((7, 7), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 1, 0]),
            ((7, 8), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 1, 1]),
            ((7, 9), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 0, 0]),
            (
                (7, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 0, 1],
            ),
            ((8, 1), vec![1, 1, 1, 1, 1, 1, 0, 0, 0]),
            ((8, 2), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0]),
            ((8, 3), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0]),
            ((8, 4), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 1]),
            ((8, 5), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0]),
            ((8, 6), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0, 1]),
            ((8, 7), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0]),
            ((8, 8), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1]),
            ((8, 9), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0]),
            (
                (8, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1],
            ),
            ((9, 1), vec![1, 1, 1, 1, 1, 1, 0, 0, 1]),
            ((9, 2), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0]),
            ((9, 3), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1]),
            ((9, 4), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0]),
            ((9, 5), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1]),
            ((9, 6), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0]),
            ((9, 7), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1]),
            ((9, 8), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0]),
            ((9, 9), vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1]),
            (
                (9, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0],
            ),
            ((10, 1), vec![1, 1, 1, 1, 1, 1, 0, 1, 0]),
            (
                (10, 2),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 1],
            ),
            (
                (10, 3),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0],
            ),
            (
                (10, 4),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1],
            ),
            (
                (10, 5),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 0],
            ),
            (
                (10, 6),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 1],
            ),
            (
                (10, 7),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0],
            ),
            (
                (10, 8),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1],
            ),
            (
                (10, 9),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0],
            ),
            (
                (10, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1],
            ),
            ((11, 1), vec![1, 1, 1, 1, 1, 1, 1, 0, 0, 1]),
            (
                (11, 2),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0],
            ),
            (
                (11, 3),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1],
            ),
            (
                (11, 4),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 0],
            ),
            (
                (11, 5),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 1],
            ),
            (
                (11, 6),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0],
            ),
            (
                (11, 7),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1],
            ),
            (
                (11, 8),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0],
            ),
            (
                (11, 9),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1],
            ),
            (
                (11, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0],
            ),
            ((12, 1), vec![1, 1, 1, 1, 1, 1, 1, 0, 1, 0]),
            (
                (12, 2),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0, 1],
            ),
            (
                (12, 3),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 0],
            ),
            (
                (12, 4),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1],
            ),
            (
                (12, 5),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0],
            ),
            (
                (12, 6),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1],
            ),
            (
                (12, 7),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0],
            ),
            (
                (12, 8),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1],
            ),
            (
                (12, 9),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
            ),
            (
                (12, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1],
            ),
            ((13, 1), vec![1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0]),
            (
                (13, 2),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0],
            ),
            (
                (13, 3),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1],
            ),
            (
                (13, 4),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0],
            ),
            (
                (13, 5),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1],
            ),
            (
                (13, 6),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0],
            ),
            (
                (13, 7),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1],
            ),
            (
                (13, 8),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0],
            ),
            (
                (13, 9),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1],
            ),
            (
                (13, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0],
            ),
            (
                (14, 1),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1],
            ),
            (
                (14, 2),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 0],
            ),
            (
                (14, 3),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1],
            ),
            (
                (14, 4),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0],
            ),
            (
                (14, 5),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1],
            ),
            (
                (14, 6),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
            ),
            (
                (14, 7),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1],
            ),
            (
                (14, 8),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0],
            ),
            (
                (14, 9),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1],
            ),
            (
                (14, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0],
            ),
            ((15, 0), vec![1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1]),
            (
                (15, 1),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1],
            ),
            (
                (15, 2),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0],
            ),
            (
                (15, 3),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1],
            ),
            (
                (15, 4),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
            ),
            (
                (15, 5),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1],
            ),
            (
                (15, 6),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0],
            ),
            (
                (15, 7),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1],
            ),
            (
                (15, 8),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
            ),
            (
                (15, 9),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1],
            ),
            (
                (15, 10),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
            ),
        ])?;

        let value = compile_write_tree::<BigEndian, i64>(HuffmanTable::generate_value_table())?;

        Ok(HuffmanTable { dc, ac, value })
    }
}

/// Reshapes a plane into an array of 8x8 blocks.
fn reshape_into_blocks(plane: &ArrayView2<u8>) -> Array2<i64> {
    let (height, width) = plane.dim();
    let blocks_y = height / 8;
    let blocks_x = width / 8;
    let mut blocks = Array2::<i64>::zeros((blocks_y * blocks_x, 64));

    for y in 0..blocks_y {
        for x in 0..blocks_x {
            for i in 0..8 {
                for j in 0..8 {
                    blocks[(y * blocks_x + x, i * 8 + j)] = plane[(y * 8 + i, x * 8 + j)] as i64;
                }
            }
        }
    }

    blocks
}

// Quantization matrix
const QUANTIZATION_TABLE: [i64; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56,
    14, 17, 22, 29, 51, 87, 80, 62, 18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113,
    92, 49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99,
];

fn quantize(blocks: &mut Array2<i64>) {
    let (num_blocks, _) = blocks.dim();

    for i in 0..num_blocks {
        for j in 0..64 {
            blocks[[i, j]] = blocks[[i, j]] / QUANTIZATION_TABLE[j];
        }
    }
}

static CONST_BITS: i64 = 13;
static PASS1_BITS: i64 = 2;
static FIX_0_298631336: i64 = 2446;
static FIX_0_390180644: i64 = 3196;
static FIX_0_541196100: i64 = 4433;
static FIX_0_765366865: i64 = 6270;
static FIX_0_899976223: i64 = 7373;
static FIX_1_175875602: i64 = 9633;
static FIX_1_501321110: i64 = 12_299;
static FIX_1_847759065: i64 = 15_137;
static FIX_1_961570560: i64 = 16_069;
static FIX_2_053119869: i64 = 16_819;
static FIX_2_562915447: i64 = 20_995;
static FIX_3_072711026: i64 = 25_172;

/// Performs a forward discrete cosine transform on each 8x8 block of
/// pixels in the given array.
///
/// The DCT is a lossy, linear transformation of the input data. It is
/// an important step in the JPEG compression process, as it allows for
/// the vast majority of the data to be discarded while still
/// preserving a good image quality.
///
/// Taken from image-rs/image/src/codecs/jpeg/transform.rs which was
/// in turn taken from jfdctint.c from libjpeg version 9a.
fn fdct(blocks: &mut Array2<i64>) {
    let (num_blocks, _) = blocks.dim();
    let mut coeffs = [0i64; 64];
    let mut coeffs = unsafe { ArrayViewMut1::from_shape_ptr(64, coeffs.as_mut_ptr()) };

    for n in 0..num_blocks {
        for y in 0usize..8 {
            let y0 = y * 8;

            let t0 = blocks[[n, y0]] + blocks[[n, y0 + 7]];
            let t1 = blocks[[n, y0 + 1]] + blocks[[n, y0 + 6]];
            let t2 = blocks[[n, y0 + 2]] + blocks[[n, y0 + 5]];
            let t3 = blocks[[n, y0 + 3]] + blocks[[n, y0 + 4]];

            let t10 = t0 + t3;
            let t12 = t0 - t3;
            let t11 = t1 + t2;
            let t13 = t1 - t2;

            let t0 = blocks[[n, y0]] - blocks[[n, y0 + 7]];
            let t1 = blocks[[n, y0 + 1]] - blocks[[n, y0 + 6]];
            let t2 = blocks[[n, y0 + 2]] - blocks[[n, y0 + 5]];
            let t3 = blocks[[n, y0 + 3]] - blocks[[n, y0 + 4]];

            coeffs[[y0]] = (t10 + t11 - 8 * 128) << PASS1_BITS;
            coeffs[[y0 + 4]] = (t0 - t11) << PASS1_BITS;

            let mut z1 = (t12 + t13) * FIX_0_541196100;
            z1 += 1 << (CONST_BITS - PASS1_BITS - 1);

            let mut t12 = t12 * (-FIX_0_390180644);
            let mut t13 = t13 * (-FIX_1_961570560);
            t12 += z1;
            t13 += z1;

            let z1 = (t0 + t3) * (-FIX_0_899976223);
            let mut t0 = t0 * FIX_1_501321110;
            let mut t3 = t3 * FIX_0_298631336;
            t0 += z1 + t12;
            t3 += z1 + t13;

            let z1 = (t1 + t2) * (-FIX_2_562915447);
            let mut t1 = t1 * FIX_3_072711026;
            let mut t2 = t2 * FIX_2_053119869;
            t1 += z1 + t13;
            t2 += z1 + t12;

            coeffs[[y0 + 1]] = t0 >> (CONST_BITS - PASS1_BITS);
            coeffs[[y0 + 3]] = t1 >> (CONST_BITS - PASS1_BITS);
            coeffs[[y0 + 5]] = t2 >> (CONST_BITS - PASS1_BITS);
            coeffs[[y0 + 7]] = t3 >> (CONST_BITS - PASS1_BITS);
        }

        for x in (0usize..8).rev() {
            // Even part
            let t0 = coeffs[[x]] + coeffs[[x + 8 * 7]];
            let t1 = coeffs[[x + 8]] + coeffs[[x + 8 * 6]];
            let t2 = coeffs[[x + 8 * 2]] + coeffs[[x + 8 * 5]];
            let t3 = coeffs[[x + 8 * 3]] + coeffs[[x + 8 * 4]];

            // Add fudge factor here for final descale
            let t10 = t0 + t3 + (1 << (PASS1_BITS - 1));
            let t12 = t0 - t3;
            let t11 = t1 + t2;
            let t13 = t1 - t2;

            let t0 = coeffs[[x]] - coeffs[[x + 8 * 7]];
            let t1 = coeffs[[x + 8]] - coeffs[[x + 8 * 6]];
            let t2 = coeffs[[x + 8 * 2]] - coeffs[[x + 8 * 5]];
            let t3 = coeffs[[x + 8 * 3]] - coeffs[[x + 8 * 4]];

            coeffs[[x]] = (t10 + t11) >> PASS1_BITS;
            coeffs[[x + 8 * 4]] = (t10 - t11) >> PASS1_BITS;

            let mut z1 = (t12 + t13) * FIX_0_541196100;
            // Add fudge factor here for final descale
            z1 += 1 << (CONST_BITS + PASS1_BITS - 1);

            coeffs[[x + 8 * 2]] = (z1 + t12 * FIX_0_765366865) >> (CONST_BITS + PASS1_BITS);
            coeffs[[x + 8 * 6]] = (z1 - t13 * FIX_1_847759065) >> (CONST_BITS + PASS1_BITS);

            // Odd part
            let t12 = t0 + t2;
            let t13 = t1 + t3;

            let mut z1 = (t12 + t13) * FIX_1_175875602;
            // Add fudge factor here for final descale
            z1 += 1 << (CONST_BITS - PASS1_BITS - 1);

            let mut t12 = t12 * (-FIX_0_390180644);
            let mut t13 = t13 * (-FIX_1_961570560);
            t12 += z1;
            t13 += z1;

            let z1 = (t0 + t3) * (-FIX_0_899976223);
            let mut t0 = t0 * FIX_1_501321110;
            let mut t3 = t3 * FIX_0_298631336;
            t0 += z1 + t12;
            t3 += z1 + t13;

            let z1 = (t1 + t2) * (-FIX_2_562915447);
            let mut t1 = t1 * FIX_3_072711026;
            let mut t2 = t2 * FIX_2_053119869;
            t1 += z1 + t13;
            t2 += z1 + t12;

            coeffs[x + 8] = t0 >> (CONST_BITS + PASS1_BITS);
            coeffs[x + 8 * 3] = t1 >> (CONST_BITS + PASS1_BITS);
            coeffs[x + 8 * 5] = t2 >> (CONST_BITS + PASS1_BITS);
            coeffs[x + 8 * 7] = t3 >> (CONST_BITS + PASS1_BITS);
        }

        blocks.slice_mut(s![n, ..]).assign(&coeffs);
    }
}

// Scan order matrix
const SCAN_ORDER_TABLE: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

/// Reorder the elements of each block in the given array in a
/// zigzag pattern. The output is a 2D array with the same number of
/// blocks as the input, but with the elements of each block reordered
/// in a zigzag pattern.
fn zigzag_order(input: &mut Array2<i64>) {
    let (num_blocks, _) = input.dim();
    let mut temp = [0i64; 64];
    let mut temp = unsafe { ArrayViewMut1::from_shape_ptr(64, temp.as_mut_ptr()) };

    for n in 0..num_blocks {
        for i in 0..64 {
            temp[[i]] = input[[n, SCAN_ORDER_TABLE[i]]];
        }

        input.slice_mut(s![n, ..]).assign(&temp);
    }
}

/// Delta encode the first column of the input array in-place.
///
/// This is a lossless encoding step, used for the DC component of the DCT.
/// The first element of the column is left unchanged, and each subsequent element
/// is replaced by the difference between it and the previous element.
fn delta_encode(input: &mut Array2<i64>) {
    let mut prev = input[[0, 0]];
    for i in 1..input.len_of(Axis(0)) {
        let curr = input[[i, 0]];
        input[[i, 0]] = curr - prev;
        prev = curr;
    }
}

/// Convert an RGB image to YUV in-place.
///
/// This is a destructive conversion, so the input array will be modified.
/// The conversion is done according to the standard RGB to YUV conversion
/// formula, which is:
///
/// Y = 0.299R + 0.587G + 0.114B
/// U = 0.565(B-Y) + 128
/// V = 0.713(R-Y) + 128
///
/// The resulting YUV values are stored in the input array, with the Y component
/// in the first channel, the U component in the second channel, and the V
/// component in the third channel.
fn rgb_to_ycbcr(frame: &mut Array3<u8>) {
    let (height, width, _) = frame.dim();
    for i in 0..height {
        for j in 0..width {
            let r = *frame.get((i, j, 0)).unwrap() as f64;
            let g = *frame.get((i, j, 1)).unwrap() as f64;
            let b = *frame.get((i, j, 2)).unwrap() as f64;
            let y = 0.299 * r + 0.587 * g + 0.114 * b;
            let u = 0.565 * (b - y) + 128.0;
            let v = 0.713 * (r - y) + 128.0;
            frame[[i, j, 0]] = y as u8;
            frame[[i, j, 1]] = u as u8;
            frame[[i, j, 2]] = v as u8;
        }
    }
}

#[derive(Debug, Clone)]
struct EncodedFrame {
    y: Array2<i64>,
    u: Array2<i64>,
    v: Array2<i64>,
}

const ZRL: (i64, i64) = (15, 0);
const EOB: (i64, i64) = (0, 0);

/// Encode the given frame using Huffman coding.
///
/// # Arguments
///
/// * `frame` - Frame to encode.
/// * `writer` - Writer to write the encoded frame to.
/// * `codebook` - Huffman codebook to use for encoding.
///
/// # Notes
///
/// This function assumes that the given `frame` has already been transformed
/// into the frequency domain using the DCT and that the quantized coefficients
/// are stored in the `y`, `u`, and `v` fields of the `frame`.
///
/// The encoding process is as follows:
///
/// 1. For each plane (Y, U, V), the first coefficient is encoded using the
///    DC codebook.
/// 2. The remaining coefficients are encoded using the AC codebook.
/// 3. The length of each run of zeros is encoded using the AC codebook.
/// 4. The coefficient value is encoded using the AC codebook.
/// 5. If the last run of zeros is not 15, an EOB (End Of Block) code is
///    written to indicate the end of the block.
fn entropy_encode<H>(frame: &EncodedFrame, writer: &mut H, codebook: &HuffmanTable)
where
    H: HuffmanWrite<BigEndian>,
{
    for plane in [&frame.y, &frame.u, &frame.v] {
        for n in 0..plane.len_of(Axis(0)) {
            let mut run = 0;

            let size = 64 - plane[[n, 0]].abs().leading_zeros() as i64;
            writer.write_huffman(&codebook.dc, size).unwrap();
            writer
                .write_huffman(&codebook.value, plane[[n, 0]])
                .unwrap();

            for i in 1..64 {
                let v = plane[[n, i]];
                if v == 0 {
                    run += 1;
                } else {
                    while run > 15 {
                        writer.write_huffman(&codebook.ac, ZRL).unwrap();
                        run -= 16;
                    }
                    let size = 64 - v.abs().leading_zeros() as i64;
                    writer.write_huffman(&codebook.ac, (run, size)).unwrap();
                    writer.write_huffman(&codebook.value, v).unwrap();
                    run = 0;
                }
            }

            if run > 0 {
                writer.write_huffman(&codebook.ac, EOB).unwrap();
            }
        }
    }
}

/// Encode a single frame.
///
/// This function takes a mutable reference to an image represented as an RGB array of
/// u8 values and returns an `EncodedFrame` containing the Y, U, and V components of the image
/// after they have been DCT'd, quantized, zigzagged, and delta encoded.
///
/// This function does not return an error. It is the caller's responsibility to ensure that the
/// array is a valid image with a power of two width and height, and that it is large enough to
/// fit into memory.
fn encode_frame(mut frame: Array3<u8>) -> EncodedFrame {
    rgb_to_ycbcr(&mut frame);

    let (h, w, _) = frame.dim();
    let y = frame.slice(s![0..h, 0..w, 0]);
    let u = frame.slice(s![0..h;2, 0..w;2, 1]);
    let v = frame.slice(s![0..h;2, 0..w;2, 2]);

    let mut yblocks = reshape_into_blocks(&y);
    fdct(&mut yblocks);
    quantize(&mut yblocks);
    zigzag_order(&mut yblocks);
    delta_encode(&mut yblocks);

    let mut ublocks = reshape_into_blocks(&u);
    fdct(&mut ublocks);
    quantize(&mut ublocks);
    zigzag_order(&mut ublocks);
    delta_encode(&mut ublocks);

    let mut vblocks = reshape_into_blocks(&v);
    fdct(&mut vblocks);
    quantize(&mut vblocks);
    zigzag_order(&mut vblocks);
    delta_encode(&mut vblocks);

    EncodedFrame {
        y: yblocks,
        u: ublocks,
        v: vblocks,
    }
}

fn encode(infile: &str, outfile: &str) -> Result<()> {
    let codebook = HuffmanTable::new()?;
    let mut writer = BitWriter::endian(
        BufWriter::with_capacity(1024 * 1024, File::create(outfile)?),
        BigEndian,
    );
    let mut decoder = Decoder::new(Path::new(infile))?;

    let frame_count = decoder.frames()? as usize;
    let (width, height) = decoder.size();
    let frame_rate = decoder.frame_rate() as u32;

    writer.write_bytes(b"tiny")?;
    writer.write_out::<16, _>(height as u16)?;
    writer.write_out::<16, _>(width as u16)?;
    writer.write_out::<16, _>(frame_rate as u16)?;
    writer.write_out::<16, _>(frame_count as u16)?;

    decoder
        .decode_iter()
        .tqdm_with_bar(tqdm!(total = frame_count))
        .take_while(Result::is_ok)
        .par_bridge()
        .map(|x| x.unwrap().1)
        .map(encode_frame)
        .collect::<Vec<_>>()
        .into_iter()
        .tqdm()
        .for_each(|frame| entropy_encode(&frame, &mut writer, &codebook));

    writer.byte_align()?;
    writer.flush()?;

    Ok(())
}

/// Parse command line arguments and execute the corresponding command.
fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encode { infile, outfile } => encode(&infile, &outfile)?,
        Commands::Decode { .. } => (),
    };

    Ok(())
}
