#!/usr/bin/env python3

import cocotb
from cocotb.triggers import *
from cocotb.clock import Clock
from cocotb.runner import get_runner

import numpy as np

import os
import sys
from pathlib import Path

from model import *

async def reset(clk, rst, cycles=2):
    rst.value = 1
    await ClockCycles(clk, cycles)
    rst.value = 0

async def clock(clk):
    await cocotb.start(Clock(clk, 10, 'ns').start(start_high=True))

async def feed_bit(dut, bit):
    await FallingEdge(dut.clk_in)

    dut.serial_in.value = bit
    dut.valid_in.value = 1

async def off(dut):
    await FallingEdge(dut.clk_in)
    dut.valid_in.value = 0

async def send_block(dut, block):
    for b in encode_block(block):
        await feed_bit(dut, b)
    await off(dut)

@cocotb.test()
async def test(dut):
    await clock(dut.clk_in)
    await reset(dut.clk_in, dut.rst_in)

    A = np.array([
        [ 52,  55,  61,  66,  70,  61,  64,  73],
        [ 63,  59,  55,  90, 109,  85,  89,  72],
        [ 62,  59,  68, 113, 144, 104,  66,  73],
        [ 63,  58,  71, 122, 154, 106,  70,  69],
        [ 67,  61,  68, 104, 126,  88,  68,  70],
        [ 79,  65,  60,  70,  77,  68,  58,  75],
        [ 85,  71,  64,  59,  55,  61,  65,  83],
        [ 87,  79,  69,  68,  65,  76,  78,  94]
    ])
    B = A.copy()

    C = transform(B)
    D = quantize(C)
    E = zigzag(D)
    F = unquantize(D)
    G = untransform(F)

    # print(f"{A.reshape((8,8))}")
    # print(f"{C.reshape((8,8)).astype(int)}")
    # print(f"{D.reshape((8,8))}")
    # print(f"{E.reshape((8,8))}")
    # print(f"{F.reshape((8,8))}")
    print(f"{G.reshape((8,8))}")

    await send_block(dut, E)
    await send_block(dut, E)

    await ClockCycles(dut.clk_in, 300)


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
        hdl_toplevel="jpeg_decoder",
        always=True,
        build_args=build_test_args,
        parameters=parameters,
        timescale = ('1ns','1ps'),
        waves=True
    )
    run_test_args = []
    runner.test(
        hdl_toplevel="jpeg_decoder",
        test_module="test_jpeg_decoder",
        test_args=run_test_args,
        waves=True
    )

if __name__ == "__main__":
    main()