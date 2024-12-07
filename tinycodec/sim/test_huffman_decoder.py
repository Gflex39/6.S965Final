#!/usr/bin/env python3

import cocotb
from cocotb.triggers import *
from cocotb.clock import Clock
from cocotb.runner import get_runner
from cocotb.utils import get_sim_time
from cocotb_bus.bus import Bus
from cocotb_bus.drivers import BusDriver
from cocotb_bus.monitors import Monitor
from cocotb_bus.monitors import BusMonitor
from cocotb_bus.scoreboard import Scoreboard
from cocotb.handle import SimHandleBase
from cocotb.binary import BinaryValue

import numpy as np

import os
import sys
import logging
from pathlib import Path

import matplotlib.pyplot as plt

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

@cocotb.test()
async def test(dut):
    delay = False

    await clock(dut.clk_in)
    await reset(dut.clk_in, dut.rst_in)

    # 5
    await feed_bit(dut, 1, delay)
    await feed_bit(dut, 1, delay)
    await feed_bit(dut, 0, delay)

    await feed_bit(dut, 1, delay)
    await feed_bit(dut, 0, delay)
    await feed_bit(dut, 0, delay)
    await feed_bit(dut, 0, delay)
    await feed_bit(dut, 1, delay)

    # 7
    await feed_bit(dut, 1, delay)
    await feed_bit(dut, 1, delay)
    await feed_bit(dut, 1, delay)
    await feed_bit(dut, 1, delay)
    await feed_bit(dut, 0, delay)

    await feed_bit(dut, 1, delay)
    await feed_bit(dut, 0, delay)
    await feed_bit(dut, 0, delay)
    await feed_bit(dut, 0, delay)
    await feed_bit(dut, 1, delay)
    await feed_bit(dut, 1, delay)
    await feed_bit(dut, 1, delay)

    await ClockCycles(dut.clk_in, 30)


def main():
    hdl_toplevel_lang = os.getenv("HDL_TOPLEVEL_LANG", "verilog")
    sim = os.getenv("SIM", "icarus")
    proj_path = Path(__file__).resolve().parent.parent
    sys.path.append(str(proj_path / "sim" / "model"))
    sources = [proj_path / "hdl" / "huffman_decoder.sv" ]
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