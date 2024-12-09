#!/usr/bin/env python3

import cocotb
from cocotb.triggers import *
from cocotb.clock import Clock
from cocotb.runner import get_runner

import numpy as np

import os
import sys
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

async def send_coeff(dut, coeff, run, size, dc = False):
    await FallingEdge(dut.clk_in)

    dut.value_in.value = coeff
    dut.run_in.value = run
    dut.size_in.value = size
    dut.valid_in.value = 1
    dut.dc_in.value = 1 if dc else 0

@cocotb.test()
async def test(dut):

    await clock(dut.clk_in)
    await reset(dut.clk_in, dut.rst_in)

    await send_coeff(dut, 0b0000000, 2, 7, True)
    await send_coeff(dut, 0b111, 3, 3, True)
    await send_coeff(dut, 0b1000, 4, 4, False)
    await send_coeff(dut, 0b100000000, 16, 9, True)

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
        hdl_toplevel="entropy_decoder",
        always=True,
        build_args=build_test_args,
        parameters=parameters,
        timescale = ('1ns','1ps'),
        waves=True
    )
    run_test_args = []
    runner.test(
        hdl_toplevel="entropy_decoder",
        test_module="test_entropy_decoder",
        test_args=run_test_args,
        waves=True
    )

if __name__ == "__main__":
    main()