extern crate bitstream_io as bitstream;
extern crate video_rs as video;

use anyhow::Result;
use bitstream::{
    huffman::{compile_write_tree, WriteHuffmanTree},
    BigEndian, BitWrite, BitWriter, HuffmanWrite,
};
use clap::{Parser, Subcommand};
use kdam::{tqdm, TqdmIterator};
use ndarray::prelude::*;
use rayon::prelude::*;
use std::{fs::File, path::Path};
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
        #[arg(short, long, default_value_t = 24)]
        qp: u16,
    },
    Decode {
        #[arg(value_name = "infile")]
        infile: String,
        #[arg(value_name = "outfile")]
        outfile: String,
    },
}

#[derive(Debug, Clone, Copy)]
enum EntropySymbol {
    DC { value: i64 },
    AC { run: i64, value: i64 },
}

struct WriterCodebook {
    dc: WriteHuffmanTree<BigEndian, i64>,
    ac: WriteHuffmanTree<BigEndian, (i64, i64)>,
    value: WriteHuffmanTree<BigEndian, i64>,
}

impl WriterCodebook {
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

        let value = compile_write_tree::<BigEndian, i64>(WriterCodebook::generate_value_table())?;

        Ok(WriterCodebook { dc, ac, value })
    }
}

/// Reshapes a plane into an array of 4x4 blocks.
fn reshape_into_blocks(plane: &ArrayView2<u8>) -> Array2<i64> {
    let (height, width) = plane.dim();
    let block_size = 4;
    let blocks_y = height / block_size;
    let blocks_x = width / block_size;
    let mut blocks = Array2::<i64>::zeros((blocks_y * blocks_x, block_size * block_size));
    blocks
        .outer_iter_mut()
        .enumerate()
        .for_each(|(index, mut block)| {
            let y = index / blocks_x;
            let x = index % blocks_x;
            block.assign(
                &plane
                    .slice(s![
                        (y * block_size)..((y + 1) * block_size),
                        (x * block_size)..((x + 1) * block_size)
                    ])
                    .to_shape(16)
                    .unwrap()
                    .map(|x| *x as i64),
            );
        });
    blocks
}

/// Quantize and transform all blocks in the given 3D array.
///
/// This function multiplies each block by the H.264 core transform and
/// quantization matrices for the given quantization parameter, which is a
/// function of the block's position in the 3D array. The result is rounded
/// to the nearest integer and stored back in the 3D array.
fn quantize_blocks<'a>(blocks: &mut Array2<i64>, qp: u16) {
    let c = array![[1, 1, 1, 1], [2, 1, -1, -2], [1, -1, -1, 1], [1, -2, 2, -1]];
    let ct = array![[1, 2, 1, 1], [1, 1, -1, -2], [1, -1, -1, 2], [1, -2, 1, -1]];

    let m = match qp % 6 {
        0 => array![
            [13107, 8066, 13107, 8066],
            [8066, 5243, 8066, 5243],
            [13107, 8066, 13107, 8066],
            [8066, 5243, 8066, 5243]
        ],
        1 => array![
            [11916, 7490, 11916, 7490],
            [7490, 4660, 7490, 4660],
            [11916, 7490, 11916, 7490],
            [7490, 4660, 7490, 4660]
        ],
        2 => array![
            [10082, 6554, 10082, 6554],
            [6554, 4194, 6554, 4194],
            [10082, 6554, 10082, 6554],
            [6554, 4194, 6554, 4194]
        ],
        3 => array![
            [9362, 5825, 9362, 5825],
            [5825, 3647, 5825, 3647],
            [9362, 5825, 9362, 5825],
            [5825, 3647, 5825, 3647]
        ],
        4 => array![
            [8192, 5243, 8192, 5243],
            [5243, 3355, 5243, 3355],
            [8192, 5243, 8192, 5243],
            [5243, 3355, 5243, 3355]
        ],
        _ => array![
            [7282, 4559, 7282, 4559],
            [4559, 2893, 4559, 2893],
            [7282, 4559, 7282, 4559],
            [4559, 2893, 4559, 2893]
        ],
    };

    blocks.outer_iter_mut().for_each(|mut block| {
        let bv = block.view();
        let mat = bv.into_shape_with_order((4, 4)).unwrap();
        let mat = (c.dot(&mat.dot(&ct)) * &m) >> (15 + (qp / 6));
        block.assign(&mat.into_shape_with_order(16).unwrap());
    });
}

/// Reorder the elements of each block in the given 3D array in a
/// zigzag pattern. The output is a 2D array with the same number of
/// blocks as the input, but with the elements of each block reordered
/// in a zigzag pattern.
fn zigzag_encode(input: &mut Array2<i64>) {
    input.outer_iter_mut().for_each(|mut block| {
        let x2 = block[2];
        let x3 = block[3];
        let x4 = block[4];
        let x5 = block[5];
        let x6 = block[6];
        let x7 = block[7];
        let x8 = block[8];
        let x9 = block[9];
        let x10 = block[10];
        let x11 = block[11];
        let x12 = block[12];
        let x13 = block[13];
        block[2] = x4;
        block[3] = x8;
        block[4] = x5;
        block[5] = x2;
        block[6] = x3;
        block[7] = x6;
        block[8] = x9;
        block[9] = x12;
        block[10] = x13;
        block[11] = x10;
        block[12] = x7;
        block[13] = x11;
    });
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

fn convert_rgb_to_yuv(frame: &mut Array3<u8>) {
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

fn entropy_encode(
    yblocks: ArrayView2<i64>,
    ublocks: ArrayView2<i64>,
    vblocks: ArrayView2<i64>,
) -> Vec<EntropySymbol> {
    let mut symbols = Vec::new();

    let mut encode_blocks = |blocks: ArrayView2<i64>| {
        for i in 0..blocks.len_of(Axis(0)) {
            let mut run = 0;
            symbols.push(EntropySymbol::DC {
                value: blocks[[i, 0]],
            });
            for j in 1..blocks.len_of(Axis(1)) {
                if blocks[[i, j]] != 0 {
                    symbols.push(EntropySymbol::AC {
                        run,
                        value: blocks[[i, j]],
                    });
                    run = 0;
                } else {
                    run += 1;
                }
            }
            if run > 0 {
                symbols.push(EntropySymbol::AC { run: 0, value: 0 });
            }
        }
    };

    encode_blocks(yblocks);
    encode_blocks(ublocks);
    encode_blocks(vblocks);

    symbols
}

fn encode_frame(frame: &mut Array3<u8>, qp: u16) -> Vec<EntropySymbol> {
    convert_rgb_to_yuv(frame);

    let (h, w, _) = frame.dim();
    let y = frame.slice(s![0..h, 0..w, 0]);
    let u = frame.slice(s![0..h;2, 0..w;2, 1]);
    let v = frame.slice(s![0..h;2, 0..w;2, 2]);

    let mut yblocks = reshape_into_blocks(&y);
    quantize_blocks(&mut yblocks, qp);
    zigzag_encode(&mut yblocks);
    delta_encode(&mut yblocks);

    let mut ublocks = reshape_into_blocks(&u);
    quantize_blocks(&mut ublocks, qp);
    zigzag_encode(&mut ublocks);
    delta_encode(&mut ublocks);

    let mut vblocks = reshape_into_blocks(&v);
    quantize_blocks(&mut vblocks, qp);
    zigzag_encode(&mut vblocks);
    delta_encode(&mut vblocks);

    entropy_encode(yblocks.view(), ublocks.view(), vblocks.view())
}

fn encode(infile: &str, outfile: &str, qp: u16) -> Result<()> {
    let codebook = WriterCodebook::new()?;
    let mut writer = BitWriter::endian(File::create(outfile)?, BigEndian);
    let mut decoder = Decoder::new(Path::new(infile))?;
    let frame_count = decoder.frames()? as usize;
    let (width, height) = decoder.size();
    let frame_rate = decoder.frame_rate() as u32;

    let symbols: Vec<_> = decoder
        .decode_iter()
        .take_while(Result::is_ok)
        .map(Result::unwrap)
        .tqdm_with_bar(tqdm!(total = frame_count))
        .par_bridge()
        .flat_map_iter(|(_, mut frame)| encode_frame(&mut frame, qp))
        .collect();

    writer.write_bytes(b"tiny")?;
    writer.write_out::<16, _>(height as u16)?;
    writer.write_out::<16, _>(width as u16)?;
    writer.write_out::<16, _>(frame_rate as u16)?;
    writer.write_out::<16, _>(frame_count as u16)?;

    symbols.into_iter().tqdm().for_each(|symbol| match symbol {
        EntropySymbol::DC { value } => {
            let size = (64 - (value.abs().leading_zeros())) as i64;
            writer.write_huffman(&codebook.dc, size).unwrap();
            writer.write_huffman(&codebook.value, value).unwrap();
        }
        EntropySymbol::AC { run, value } => {
            let size = (64 - (value.abs().leading_zeros())) as i64;
            writer.write_huffman(&codebook.ac, (run, size)).unwrap();
            writer.write_huffman(&codebook.value, value).unwrap();
        }
    });

    writer.byte_align()?;
    writer.flush()?;

    Ok(())
}

/// Parse command line arguments and execute the corresponding command.
fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Encode {
            infile,
            outfile,
            qp,
        } => encode(&infile, &outfile, *qp)?,
        Commands::Decode { .. } => (),
    };

    Ok(())
}
