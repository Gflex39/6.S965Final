#!/usr/bin/env python3

import cocotb
from cocotb.triggers import *
from cocotb.clock import Clock
from cocotb.runner import get_runner

import numpy as np

import os
import sys
import struct
from pathlib import Path

async def reset(clk, rst, cycles=2):
    rst.value = 1
    await ClockCycles(clk, cycles)
    rst.value = 0

async def clock(clk):
    await cocotb.start(Clock(clk, 10, 'ns').start(start_high=True))

async def off(dut):
    await FallingEdge(dut.clk_in)
    dut.valid_in.value = 0

async def send_block(dut, block):
    for col in range(8):
        await FallingEdge(dut.clk_in)
        x = ((int(block[7,col])<<84)
            |(int(block[6,col])<<72)
            |(int(block[5,col])<<60)
            |(int(block[4,col])<<48)
            |(int(block[3,col])<<36)
            |(int(block[2,col])<<24)
            |(int(block[1,col])<<12)
            |(int(block[0,col])<< 0))

        dut.column_in.value = int(x)
        dut.valid_in.value = 1

    await FallingEdge(dut.clk_in)
    dut.valid_in.value = 0

@cocotb.test()
async def test(dut):
    await clock(dut.clk_in)
    await reset(dut.clk_in, dut.rst_in)

    await send_block(dut, np.ones((8,8), dtype=int))
    await send_block(dut, 2*np.ones((8,8), dtype=int))

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
        hdl_toplevel="inverse_quantizer",
        always=True,
        build_args=build_test_args,
        parameters=parameters,
        timescale = ('1ns','1ps'),
        waves=True
    )
    run_test_args = []
    runner.test(
        hdl_toplevel="inverse_quantizer",
        test_module="test_inverse_quantizer",
        test_args=run_test_args,
        waves=True
    )

if __name__ == "__main__":
    main()