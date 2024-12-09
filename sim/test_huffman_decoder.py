#!/usr/bin/env python3

import cocotb
from cocotb.triggers import *
from cocotb.clock import Clock
from cocotb.runner import get_runner

import numpy as np

import os
import sys
from pathlib import Path

from model import encode_block

async def reset(clk, rst, cycles=2):
    rst.value = 1
    await ClockCycles(clk, cycles)
    rst.value = 0

async def clock(clk):
    await cocotb.start(Clock(clk, 10, 'ns').start(start_high=True))

async def feed_bit(dut, bit, delay):
    await FallingEdge(dut.clk_in)

    dut.serial_in.value = bit
    dut.valid_in.value = 1

    if delay:
        await FallingEdge(dut.clk_in)
        dut.valid_in.value = 0

async def off(dut):
    await FallingEdge(dut.clk_in)
    dut.valid_in.value = 0

async def send_block(dut, block, delay):
    for b in encode_block(block):
        await feed_bit(dut, b, delay)

@cocotb.test()
async def test(dut):
    delay = False

    await clock(dut.clk_in)
    await reset(dut.clk_in, dut.rst_in)

    await send_block(dut, np.array([
        [2047,0,0,1,512,0,0,1],
        [0,0,0,1,0,0,0,-1],
        [0,0,0,1,0,0,0,-1],
        [0,0,0,1,0,0,0,-1],
        [0,-23,0,1,0,0,0,-1],
        [0,0,0,0,0,0,0,0],
        [0,34,0,0,245,0,0,0],
        [0,0,0,0,0,0,0,9]
    ]), delay)

    for i in range(1,8):
        await send_block(dut, i*np.ones((8,8), dtype=np.int64), delay)
        for i in range(5):
            await off(dut)

    await send_block(dut, np.zeros((8,8), dtype=np.int64), delay)

    await off(dut)
    await ClockCycles(dut.clk_in, 30)


def main():
    sim = os.getenv("SIM", "icarus")
    proj_path = Path(__file__).resolve().parent.parent
    sys.path.append(str(proj_path / "sim" / "model"))
    sources = [proj_path / "hdl" / f for f in os.listdir(proj_path / "hdl")]
    build_test_args = ["-Wall"]
    parameters = {}
    sys.path.append(str(proj_path / "sim"))
    runner = get_runner(sim)
    runner.build(
        sources=sources,
        hdl_toplevel="huffman_decoder",
        always=True,
        build_args=build_test_args,
        parameters=parameters,
        timescale = ('1ns','1ps'),
        waves=True
    )
    run_test_args = []
    runner.test(
        hdl_toplevel="huffman_decoder",
        test_module="test_huffman_decoder",
        test_args=run_test_args,
        waves=True
    )

if __name__ == "__main__":
    main()