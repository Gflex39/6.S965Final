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
    encoded = encode_block(block)
    print(encoded)

    for b in encoded:
        await feed_bit(dut, b)

    await off(dut)

@cocotb.test()
async def test(dut):
    await clock(dut.clk_in)
    await reset(dut.clk_in, dut.rst_in)

    with open("../tinycodec/data/frame.bin", "rb") as f:
        A = f.read()

    for a in A:
        for bit in range(8):
            await feed_bit(dut, (a >> bit) & 1)

    # A = np.array(
    #   [[228, 228, 228, 228, 228, 228, 228, 228],
    #    [228, 228, 228, 228, 228, 228, 228, 228],
    #    [228, 228, 228, 228, 228, 228, 228, 228],
    #    [228, 228, 228, 228, 228, 228, 228, 228],
    #    [228, 228, 228, 228, 228, 228, 228, 228],
    #    [228, 228, 228, 228, 228, 228, 228, 228],
    #    [228, 228, 228, 228, 228, 228, 228, 228],
    #    [228, 228, 228, 228, 228, 228, 228, 228]]
    # )
    # B = A.copy()

    # C = transform(B)
    # D = quantize(C)
    # E = zigzag(D)
    # F = unquantize(D)
    # G = untransform(F)

    # # print(f"{A.reshape((8,8))}")
    # print(f"{C.reshape((8,8)).astype(int)}")
    # print(f"{D.reshape((8,8))}")
    # print(f"{E.reshape((8,8))}")
    # print(f"{F.reshape((8,8))}")
    # print(f"{G.reshape((8,8))}")

    # await send_block(dut, E)
    # await send_block(dut, E)

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