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

async def ready(dut, value):
    await FallingEdge(dut.clk)
    dut.m00_axis_tready.value = value

async def clock(clk):
    await cocotb.start(Clock(clk, 10, 'ns').start(start_high=True))

class EtherMonitor(BusMonitor):
    def __init__(self, dut, name, clk, **kwargs):
        self._signals = ['axiov','axiod']
        BusMonitor.__init__(self, dut, name, clk, **kwargs)
        self.clock = clk
        self.transactions = 0
        self.sample = 0

    async def _monitor_recv(self):
        falling_edge = FallingEdge(self.clock)
        read_only = ReadOnly()
        while True:
            await falling_edge
            await read_only
            valid = self.bus.axiov.value
            data = self.bus.axiod.value
            if valid:
              self.transactions += 1
              thing = dict(data=data,name=self.name,count=self.transactions)
              self._recv(thing)

class EtherDriver(BusDriver):
  def __init__(self, dut, name, clk):
    self._signals = ['axiiv','axiid']
    BusDriver.__init__(self, dut, name, clk)
    self.clock = clk
    self.dut=dut
    self.bus.axiid.value = 0
    self.bus.axiiv.value = 0

  async def _driver_send(self, value, sync=True):
    falling_edge = FallingEdge(self.clock)
    read_only = ReadOnly()

    await falling_edge

    for data in value["contents"]["data"]:
        self.bus.axiiv.value = 1
        self.bus.axiid.value = data
        await read_only
        await falling_edge

    self.bus.axiiv.value = 0
    self.bus.axiid.value = 0
    await clock(self.dut.clk)
    await reset(self.dut.clk, self.dut.rst)
    await clock(self.dut.clk)

class EthernetTester:
    def __init__(self, dut_entity: SimHandleBase, debug=False):
        self.dut = dut_entity
        self.log = logging.getLogger("cocotb.tb")
        self.log.setLevel(logging.DEBUG)
        self.output_mon = EtherMonitor(self.dut,None,self.dut.clk, callback=self.model)
        self.input_driver = EtherDriver(self.dut,None,self.dut.clk)
        self.outputs = []

    def stop(self) -> None:
        self.output_mon.stop()
        self.input_driver.stop()

    def model(self, transaction):
        x = transaction["data"].signed_integer
        # z = np.int32(x)
        self.outputs.append(x)

@cocotb.test()
async def test(dut):
    tester = EthernetTester(dut)
    await clock(dut.clk)
    await reset(dut.clk, dut.rst)
    await clock(dut.clk)
    preamble=["01010101"]
    sfd=["11010101"]
    address=["00001111"]
    length=[]
    string='11011110101011011011111011101111'
                                                                                                                                                                                    
    packets=[
       ['11011110', '10101101', '10111110', '11101111', '11011110', '11011110', '10111110', '11101111', '11011110', '10101101', '11111110', '11111110', '00000001', '00000011'],
       ['00000001']*46,
       ['01000010', '10000110', '01001110', '01001110', '10011110', '10000100', '00000100', '01000010', '01001110', '10100110', '10000110', '11010110', '01100110', '10000110', '11001110', '00101110', '00000100', '00101110', '10010110', '10110110', '10100110']
    ]                                                                                           #['00100111']+['11010011']+['10110010']+['10001101']
                                                                                                #['10101110']+['11001110']+['00100001']+['10010001']
    for packet in packets:
        # print(print(int("".join(packet[:-4]),2)))
        # print(hex(int("".join(packet),2)))

        
        # print("3 different FCS")
        # print(hex(int(crc32_calculator(int("".join(packet),2)),2)))
        network_packet="".join([normal_to_network(byte) for byte in packet])
        # print(hex(int(network_packet,2)))
        fcs=crc32_calculator(int(network_packet,2))
        print(hex(int(fcs,2)))
        if len(fcs)%8!=0:
            fcs="0"*(8-(len(fcs))%8)+fcs
        fcs=[fcs[8*i:8*i+8] for i in range(len(fcs)//8)]
        # print(len(fcs))
        # print()
        # t=hex(int("".join(fcs),2))

        # print(f"FCS is {t}")
        print(hex(int(network_packet,2)))
        print(hex(int("".join(packet),2)))
        
        fcs=[i[::-1] for i in fcs]
        print(hex(int("".join(fcs),2)))
        network_packet="".join([normal_to_dibit_network(byte) for byte in packet+fcs])

        di_bits=[network_packet[2*i:2*i+2] for i in range(len(network_packet)//2)]
        tester.input_driver.append({ "type": "burst", "contents": { "data": [int(i,2) for i in di_bits] } })

    await ClockCycles(dut.clk, 400)
    # print(hex(int(dut.axiod.value)))

def normal_to_network(byte):
    return byte[::-1]

def normal_to_dibit_network(byte):
    split_byte=[byte[2*i:2*i+2] for i in range(len(byte)//2)]
    split_byte.reverse()

    return "".join(split_byte)

def fields_to_packet(fields):
    return "".join(fields)

def crc32_calculator(data):
    a = data.to_bytes((data.bit_length() + 7) // 8,byteorder='big')
    crc = 0xffffffff
    # print(len(a))
    for x in a:
        crc ^= x << 24;
        for k in range(8):
            crc = (crc << 1) ^ 0x04c11db7 if crc & 0x80000000 else crc << 1
        # print(x)
    crc = ~crc
    crc &= 0xffffffff
    return bin(crc)[2:]

    

   




def main():
    hdl_toplevel_lang = os.getenv("HDL_TOPLEVEL_LANG", "verilog")
    sim = os.getenv("SIM", "icarus")
    proj_path = Path(__file__).resolve().parent.parent
    sys.path.append(str(proj_path / "sim" / "model"))
    sources = [proj_path / "hdl" / "crc32.sv"]
    build_test_args = ["-Wall"]
    parameters = {}
    sys.path.append(str(proj_path / "sim"))
    runner = get_runner(sim)
    runner.build(
        sources=sources,
        hdl_toplevel="crc32",
        always=True,
        build_args=build_test_args,
        parameters=parameters,
        timescale = ('1ns','1ps'),
        waves=True
    )
    run_test_args = []
    runner.test(
        hdl_toplevel="crc32",
        test_module="crc_test",
        test_args=run_test_args,
        waves=True
    )

if __name__ == "__main__":
    main()
    # print(crc32_calculator(3735928559))
    # print(normal_to_network("00111000111110110010001010000100"))