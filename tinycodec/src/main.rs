extern crate bitstream_io as bitstream;
extern crate video_rs as video;

use anyhow::{anyhow, Result};
use bitstream::{
    huffman::{compile_read_tree, compile_write_tree, ReadHuffmanTree, WriteHuffmanTree},
    BigEndian, BitRead, BitReader, BitWrite, BitWriter, HuffmanRead, HuffmanWrite,
};
use clap::{Parser, Subcommand};
use core::str;
use kdam::{tqdm, TqdmIterator};
use ndarray::prelude::*;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};
use video::{decode::Decoder, encode::Settings, Encoder, Frame, Time};

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
    dc_write: WriteHuffmanTree<BigEndian, i64>,
    ac_write: WriteHuffmanTree<BigEndian, (i64, i64)>,
    dc_read: Box<[ReadHuffmanTree<BigEndian, i64>]>,
    ac_read: Box<[ReadHuffmanTree<BigEndian, (i64, i64)>]>,
}

impl HuffmanTable {
    fn new() -> Result<Self> {
        let dc_table = vec![
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
            (-1, vec![1, 1, 1, 1, 1, 1, 1, 1, 1]),
        ];

        let ac_table = vec![
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
            (
                (-1, -1),
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            ),
        ];

        let dc_write = compile_write_tree::<BigEndian, i64>(dc_table.clone())?;
        let ac_write = compile_write_tree::<BigEndian, (i64, i64)>(ac_table.clone())?;
        let dc_read = compile_read_tree::<BigEndian, i64>(dc_table)?;
        let ac_read = compile_read_tree::<BigEndian, (i64, i64)>(ac_table)?;

        Ok(HuffmanTable {
            dc_write,
            ac_write,
            dc_read,
            ac_read,
        })
    }
}

fn fdct(mut blocks: ArrayViewMut2<i64>) {
    let (num_blocks, _) = blocks.dim();
    let t = array![
        [
            0.35355339, 0.35355339, 0.35355339, 0.35355339, 0.35355339, 0.35355339, 0.35355339,
            0.35355339,
        ],
        [
            0.49039264,
            0.41573481,
            0.27778512,
            0.09754516,
            -0.09754516,
            -0.27778512,
            -0.41573481,
            -0.49039264,
        ],
        [
            0.46193977,
            0.19134172,
            -0.19134172,
            -0.46193977,
            -0.46193977,
            -0.19134172,
            0.19134172,
            0.46193977,
        ],
        [
            0.41573481,
            -0.09754516,
            -0.49039264,
            -0.27778512,
            0.27778512,
            0.49039264,
            0.09754516,
            -0.41573481,
        ],
        [
            0.35355339,
            -0.35355339,
            -0.35355339,
            0.35355339,
            0.35355339,
            -0.35355339,
            -0.35355339,
            0.35355339,
        ],
        [
            0.27778512,
            -0.49039264,
            0.09754516,
            0.41573481,
            -0.41573481,
            -0.09754516,
            0.49039264,
            -0.27778512,
        ],
        [
            0.19134172,
            -0.46193977,
            0.46193977,
            -0.19134172,
            -0.19134172,
            0.46193977,
            -0.46193977,
            0.19134172,
        ],
        [
            0.09754516,
            -0.27778512,
            0.41573481,
            -0.49039264,
            0.49039264,
            -0.41573481,
            0.27778512,
            -0.09754516,
        ],
    ];
    let mut block = Array2::<f64>::zeros((8, 8));
    let tt = t.t();

    blocks -= 128;

    for n in 0..num_blocks {
        block.indexed_iter_mut().for_each(|((i, j), v)| {
            *v = blocks[(n, i * 8 + j)] as f64;
        });
        let x = t.dot(&block.dot(&tt));
        blocks
            .slice_mut(s![n, ..])
            .indexed_iter_mut()
            .for_each(|(i, v)| {
                *v = (x[[i / 8, i % 8]] / QUANTIZATION_TABLE[i] as f64).round() as i64;
            })
    }
}

fn idct(mut blocks: ArrayViewMut2<i64>) {
    let (num_blocks, _) = blocks.dim();
    let mut block = Array2::<f64>::zeros((8, 8));
    let t = array![
        [
            0.35355339, 0.35355339, 0.35355339, 0.35355339, 0.35355339, 0.35355339, 0.35355339,
            0.35355339,
        ],
        [
            0.49039264,
            0.41573481,
            0.27778512,
            0.09754516,
            -0.09754516,
            -0.27778512,
            -0.41573481,
            -0.49039264,
        ],
        [
            0.46193977,
            0.19134172,
            -0.19134172,
            -0.46193977,
            -0.46193977,
            -0.19134172,
            0.19134172,
            0.46193977,
        ],
        [
            0.41573481,
            -0.09754516,
            -0.49039264,
            -0.27778512,
            0.27778512,
            0.49039264,
            0.09754516,
            -0.41573481,
        ],
        [
            0.35355339,
            -0.35355339,
            -0.35355339,
            0.35355339,
            0.35355339,
            -0.35355339,
            -0.35355339,
            0.35355339,
        ],
        [
            0.27778512,
            -0.49039264,
            0.09754516,
            0.41573481,
            -0.41573481,
            -0.09754516,
            0.49039264,
            -0.27778512,
        ],
        [
            0.19134172,
            -0.46193977,
            0.46193977,
            -0.19134172,
            -0.19134172,
            0.46193977,
            -0.46193977,
            0.19134172,
        ],
        [
            0.09754516,
            -0.27778512,
            0.41573481,
            -0.49039264,
            0.49039264,
            -0.41573481,
            0.27778512,
            -0.09754516,
        ],
    ];
    let tt = t.t();

    for n in 0..num_blocks {
        block.indexed_iter_mut().for_each(|((i, j), v)| {
            *v = (blocks[(n, i * 8 + j)] * QUANTIZATION_TABLE[i * 8 + j]) as f64;
        });
        let x = tt.dot(&block.dot(&t));
        blocks
            .slice_mut(s![n, ..])
            .indexed_iter_mut()
            .for_each(|(i, v)| {
                *v = (x[[i / 8, i % 8]].round() as i64).clamp(-128, 127);
            })
    }

    blocks += 128;
}

/// Reshapes a plane into an array of 8x8 blocks.
fn reshape_into_blocks(plane: ArrayView2<u8>) -> Array2<i64> {
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

fn reshape_into_plane(height: usize, width: usize, blocks: ArrayView2<i64>) -> Array2<u8> {
    let mut plane = Array2::<u8>::zeros((height, width));
    let blocks_y = height / 8;
    let blocks_x = width / 8;

    for y in 0..blocks_y {
        for x in 0..blocks_x {
            for i in 0..8 {
                for j in 0..8 {
                    plane[[y * 8 + i, x * 8 + j]] = blocks[[y * blocks_x + x, i * 8 + j]] as u8;
                }
            }
        }
    }

    plane
}

// Quantization matrix
const QUANTIZATION_TABLE: [i64; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69, 56,
    14, 17, 22, 29, 51, 87, 80, 62, 18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81, 104, 113,
    92, 49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99,
];

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
fn zigzag_order(mut input: ArrayViewMut2<i64>) {
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

fn unzigzag_order(mut input: ArrayViewMut2<i64>) {
    let (num_blocks, _) = input.dim();
    let mut temp = [0i64; 64];
    let mut temp = unsafe { ArrayViewMut1::from_shape_ptr(64, temp.as_mut_ptr()) };

    for n in 0..num_blocks {
        for i in 0..64 {
            temp[[SCAN_ORDER_TABLE[i]]] = input[[n, i]];
        }

        input.slice_mut(s![n, ..]).assign(&temp);
    }
}

/// Delta encode the first column of the input array in-place.
///
/// This is a lossless encoding step, used for the DC component of the DCT.
/// The first element of the column is left unchanged, and each subsequent element
/// is replaced by the difference between it and the previous element.
fn delta_encode(mut input: ArrayViewMut2<i64>) {
    let mut prev = input[[0, 0]];
    for i in 1..input.len_of(Axis(0)) {
        let curr = input[[i, 0]];
        input[[i, 0]] = curr - prev;
        prev = curr;
    }
}

fn delta_decode(mut input: ArrayViewMut2<i64>) {
    for i in 1..input.len_of(Axis(0)) {
        input[[i, 0]] += input[[i - 1, 0]];
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
fn rgb_to_yuv(mut frame: ArrayViewMut3<u8>) {
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

/// Convert a YUV image to RGB in-place.
///
/// This is a destructive conversion, so the input array will be modified.
/// The conversion is done according to the standard YUV to RGB conversion
/// formula, which is:
///
/// R = Y + 1.4903 * (V - 128)
/// G = Y - 0.344 * (U - 128) - 0.714 * (V - 128)
/// B = Y + 1.770 * (U - 128)
///
/// The resulting RGB values are stored in the input array, with the R component
/// in the first channel, the G component in the second channel, and the B
/// component in the third channel.
fn yuv_to_rgb(mut frame: ArrayViewMut3<u8>) {
    let (height, width, _) = frame.dim();
    for i in 0..height {
        for j in 0..width {
            let y = *frame.get((i, j, 0)).unwrap() as f64;
            let u = *frame.get((i, j, 1)).unwrap() as f64;
            let v = *frame.get((i, j, 2)).unwrap() as f64;
            let r = y + 1.4903 * (v - 128.0);
            let g = y - 0.344 * (u - 128.0) - 0.714 * (v - 128.0);
            let b = y + 1.770 * (u - 128.0);
            frame[[i, j, 0]] = r as u8;
            frame[[i, j, 1]] = g as u8;
            frame[[i, j, 2]] = b as u8;
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
fn entropy_encode<W>(
    frame: &EncodedFrame,
    writer: &mut BitWriter<W, BigEndian>,
    codebook: &HuffmanTable,
) where
    W: Write,
{
    for plane in [&frame.y, &frame.u, &frame.v] {
        for n in 0..plane.len_of(Axis(0)) {
            let mut run = 0;
            let v = plane[[n, 0]];
            let size = 64 - v.abs().leading_zeros() as i64;
            let v = if v < 0 {
                (v - 1) & ((1 << (size)) - 1)
            } else {
                v
            };

            writer.write_huffman(&codebook.dc_write, size).unwrap();

            if size > 0 {
                writer.write(size as u32, v).unwrap();
            }

            for i in 1..64 {
                let v = plane[[n, i]];
                let size = 64 - v.abs().leading_zeros() as i64;
                let v = if v < 0 {
                    (v - 1) & ((1 << (size)) - 1)
                } else {
                    v
                };

                if plane[[n, i]] == 0 {
                    run += 1;
                } else {
                    while run > 15 {
                        writer.write_huffman(&codebook.ac_write, ZRL).unwrap();
                        run -= 16;
                    }
                    writer
                        .write_huffman(&codebook.ac_write, (run, size))
                        .unwrap();
                    if size > 0 {
                        writer.write(size as u32, v).unwrap();
                    }
                    run = 0;
                }
            }

            if run > 0 {
                writer.write_huffman(&codebook.ac_write, EOB).unwrap();
            }
        }
    }
}

/// Decodes the given reader using the given Huffman codebook and returns
/// an array of `num_blocks` blocks of 64 coefficients each. The input is
/// assumed to be a sequence of blocks of delta-encoded coefficients.
///
/// The decoding process is as follows:
///
/// 1. The first coefficient is decoded using the DC codebook.
/// 2. The remaining coefficients are decoded using the AC codebook.
/// 3. The length of each run of zeros is decoded using the AC codebook.
/// 4. The coefficient value is decoded using the AC codebook.
/// 5. If the last run of zeros is not 15, an EOB (End Of Block) code is
///    written to indicate the end of the block.
///
/// The output is an array of `num_blocks` blocks of 64 coefficients each.
fn entropy_decode<R>(
    reader: &mut BitReader<R, BigEndian>,
    codebook: &HuffmanTable,
    num_blocks: usize,
) -> Array2<i64>
where
    R: Read,
{
    let mut result = Array2::zeros((num_blocks, 64));

    for n in 0..num_blocks {
        let mut position = 0;
        let size = reader.read_huffman(&codebook.dc_read).unwrap();

        let v = if size > 0 {
            let v: i64 = reader.read(size as u32).unwrap();
            if v >= (1 << (size - 1)) {
                v
            } else {
                v - (1 << size) + 1
            }
        } else {
            0
        };

        result[[n, position]] = v;
        position += 1;

        'inner: while position < 64 {
            let (run, size) = reader.read_huffman(&codebook.ac_read).unwrap();

            if run == 0 && size == 0 {
                break 'inner;
            }

            let v = if size > 0 {
                let v: i64 = reader.read(size as u32).unwrap();
                if v >= (1 << (size - 1)) {
                    v
                } else {
                    v - (1 << size) + 1
                }
            } else {
                0
            };

            position += run as usize;
            result[[n, position]] = v;
            position += 1;
        }
    }

    result
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
    rgb_to_yuv(frame.view_mut());

    let (h, w, _) = frame.dim();
    let y = frame.slice(s![0..h, 0..w, 0]);
    let u = frame.slice(s![0..h;2, 0..w;2, 1]);
    let v = frame.slice(s![0..h;2, 0..w;2, 2]);

    let mut yblocks = reshape_into_blocks(y);
    fdct(yblocks.view_mut());
    zigzag_order(yblocks.view_mut());
    // delta_encode(yblocks.view_mut());

    let mut ublocks = reshape_into_blocks(u);
    fdct(ublocks.view_mut());
    zigzag_order(ublocks.view_mut());
    // delta_encode(ublocks.view_mut());

    let mut vblocks = reshape_into_blocks(v);
    fdct(vblocks.view_mut());
    zigzag_order(vblocks.view_mut());
    // delta_encode(vblocks.view_mut());

    EncodedFrame {
        y: yblocks,
        u: ublocks,
        v: vblocks,
    }
}

fn decode_frame<R>(
    reader: &mut BitReader<R, BigEndian>,
    codebook: &HuffmanTable,
    height: usize,
    width: usize,
) -> Frame
where
    R: Read,
{
    let hblocks = height / 8;
    let wblocks = width / 8;

    let mut yblocks = entropy_decode(reader, codebook, hblocks * wblocks);
    let mut ublocks = entropy_decode(reader, codebook, hblocks * wblocks / 4);
    let mut vblocks = entropy_decode(reader, codebook, hblocks * wblocks / 4);

    // delta_decode(yblocks.view_mut());
    unzigzag_order(yblocks.view_mut());
    idct(yblocks.view_mut());

    // delta_decode(ublocks.view_mut());
    unzigzag_order(ublocks.view_mut());
    idct(ublocks.view_mut());

    // delta_decode(vblocks.view_mut());
    unzigzag_order(vblocks.view_mut());
    idct(vblocks.view_mut());

    let y = reshape_into_plane(height, width, yblocks.view());
    let u = reshape_into_plane(height / 2, width / 2, ublocks.view());
    let v = reshape_into_plane(height / 2, width / 2, vblocks.view());

    let mut frame = Array3::<u8>::zeros((height, width, 3));

    frame.indexed_iter_mut().for_each(|((i, j, c), elem)| {
        *elem = match c {
            0 => y[[i, j]],
            1 => u[[i >> 1, j >> 1]],
            _ => v[[i >> 1, j >> 1]],
        };
    });

    yuv_to_rgb(frame.view_mut());

    frame
}

fn encode(infile: &str, outfile: &str) -> Result<()> {
    let codebook = HuffmanTable::new()?;
    let mut writer = BitWriter::endian(
        BufWriter::with_capacity(20 * 1024 * 1024, File::create(outfile)?),
        BigEndian,
    );
    let mut decoder = Decoder::new(Path::new(infile))?;

    let frame_count = decoder.frames()? as usize;
    let (width, height) = decoder.size();
    let frame_rate = decoder.frame_rate() as u32;

    // writer.write_bytes(b"tiny")?;
    // writer.write_out::<16, _>(height as u16)?;
    // writer.write_out::<16, _>(width as u16)?;
    // writer.write_out::<16, _>(frame_rate as u16)?;
    // writer.write_out::<16, _>(frame_count as u16)?;

    decoder
        .decode_iter()
        .tqdm_with_bar(tqdm!(total = frame_count))
        .take_while(Result::is_ok)
        .for_each(|frame| {
            let frame = encode_frame(frame.unwrap().1);
            entropy_encode(&frame, &mut writer, &codebook);
        });

    writer.byte_align()?;
    writer.flush()?;

    Ok(())
}

fn decode(infile: &str, outfile: &str) -> Result<()> {
    let codebook = HuffmanTable::new()?;
    let mut reader = BitReader::endian(
        BufReader::with_capacity(20 * 1024 * 1024, File::open(infile)?),
        BigEndian,
    );
    let mut header = [0u8; 4];

    reader.read_bytes(&mut header)?;

    if !header.eq(b"tiny") {
        return Err(anyhow!(
            "Invalid header: {:?}",
            str::from_utf8(&header).unwrap()
        ));
    }

    let height = reader.read_in::<16, u32>().unwrap() as usize;
    let width = reader.read_in::<16, u32>().unwrap() as usize;
    let frame_rate = reader.read_in::<16, u32>().unwrap() as usize;
    let frame_count = reader.read_in::<16, u32>().unwrap() as usize;

    let mut encoder = Encoder::new(
        Path::new(outfile),
        Settings::preset_h264_yuv420p(width, height, false),
    )?;

    let duration = Time::from_nth_of_a_second(frame_rate);
    let mut position = Time::zero();

    for _ in tqdm!(0..frame_count) {
        let frame = decode_frame(&mut reader, &codebook, height, width);
        encoder.encode(&frame, position).unwrap();
        position = position.aligned_with(duration).add();
    }

    encoder.finish()?;

    Ok(())
}

/// Parse command line arguments and execute the corresponding command.
fn main() -> Result<()> {
    match &Cli::parse().command {
        Commands::Encode { infile, outfile } => encode(infile, outfile),
        Commands::Decode { infile, outfile } => decode(infile, outfile),
    }
}
