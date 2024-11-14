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
import random
import matplotlib.pyplot as plt

random.seed(1)
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
        self._signals = ['axis_tvalid','axis_tdata']
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
            valid = self.bus.axis_tvalid.value
            data = self.bus.axis_tdata.value
            if valid:
              self.transactions += 1
              thing = dict(data=data,name=self.name,count=self.transactions)
              self._recv(thing)

class EtherDriver(BusDriver):
  def __init__(self, dut, name, clk):
    self._signals = ['rxd','crsdv']
    BusDriver.__init__(self, dut, name, clk)
    self.clock = clk
    self.bus.rxd.value = 0
    self.bus.crsdv.value = 0

  async def _driver_send(self, value, sync=True):
    falling_edge = FallingEdge(self.clock)
    read_only = ReadOnly()

    await falling_edge

    for data in value["contents"]["data"]:
        self.bus.crsdv.value = 1
        self.bus.rxd.value = data
        await read_only
        await falling_edge

    self.bus.rxd.value = 0
    self.bus.crsdv.value = 0

class EthernetTester:
    def __init__(self, dut_entity: SimHandleBase, debug=False):
        self.dut = dut_entity
        self.log = logging.getLogger("cocotb.tb")
        self.log.setLevel(logging.DEBUG)
        self.output_mon = EtherMonitor(self.dut,'m00',self.dut.clk, callback=self.model)
        self.input_driver = EtherDriver(self.dut,"ether",self.dut.clk)

        self.outputs = []
        self.expected_outputs = []

    def stop(self) -> None:
        self.output_mon.stop()
        self.input_driver.stop()

    def model(self, transaction):
        x = int(transaction["data"])
        # z = np.int32(x)
        self.outputs.append(x)

@cocotb.test()
async def test(dut):
    tester = EthernetTester(dut)
    await clock(dut.clk)
    await reset(dut.clk, dut.rst)
    preamble=["01010101"]*7
    sfd=["11010101"]
    source_address=byte_pad(bin(int("DEADBEEFDEDE",16))[2:])
    destination_address=byte_pad(bin(int("BEEFDEADFEFE",16))[2:])
    source_address=[source_address[8*i:8*i+8] for i in range(len(source_address)//8)]
    destination_address=[destination_address[8*i:8*i+8] for i in range(len(destination_address)//8)]

    
    packets=[ integer_to_packet_data(random.randint(0,2**1024)) for _ in range(10)]
    for data in packets:
        # print(print(int("".join(packet[:-4]),2)))
        allow=random.random()>0.5
        data_value=data[2:]
        
        if allow:
            destination=destination_address
            tester.expected_outputs.append(int("".join(data_value),2))
        else:
            destination=source_address

        packet=destination+source_address+data

        network_packet="".join([normal_to_network(byte) for byte in packet])

        fcs=crc32_calculator(int(network_packet,2))
        # print(hex(int(fcs,2)))
        if len(fcs)%8!=0:
            fcs="0"*(8-(len(fcs))%8)+fcs
        fcs=[fcs[8*i:8*i+8] for i in range(len(fcs)//8)]
        fcs=[i[::-1] for i in fcs]
        network_packet="".join([normal_to_dibit_network(byte) for byte in preamble+sfd+packet+fcs])

        di_bits=[network_packet[2*i:2*i+2] for i in range(len(network_packet)//2)]
        # print(di_bits)
        tester.input_driver.append({ "type": "burst", "contents": { "data": [int(i,2) for i in di_bits] } })


    await ClockCycles(dut.clk,10000 )
    print(len(tester.outputs))
    print(len(tester.expected_outputs))
    assert tester.outputs == tester.expected_outputs
    


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

def integer_to_packet_data(integer):
    binary=byte_pad(bin(integer)[2:])

    length=byte_pad(bin(len(binary)//8)[2:])
    if len(length)<16:
        length="0"*(16-len(length))+length
    # print(length)
    
    
    data=[binary[8*i:8*i+8] for i in range(len(binary)//8)]
    length=[length[8*i:8*i+8] for i in range(len(length)//8)]
    # print([\
    #     normal_to_dibit_network(i) for i in length+data])
    return length+data
    

def byte_pad(s):
    s_padded=s
    if len(s)%8!=0:
        s_padded="0"*(8-(len(s))%8)+s
    return s_padded




def main():
    hdl_toplevel_lang = os.getenv("HDL_TOPLEVEL_LANG", "verilog")
    sim = os.getenv("SIM", "icarus")
    proj_path = Path(__file__).resolve().parent.parent
    sys.path.append(str(proj_path / "sim" / "model"))
    sources = [proj_path / "hdl" / "ether.sv",proj_path / "hdl" / "crc32.sv"]
    build_test_args = ["-Wall"]
    parameters = {}
    sys.path.append(str(proj_path / "sim"))
    runner = get_runner(sim)
    runner.build(
        sources=sources,
        hdl_toplevel="ether",
        always=True,
        build_args=build_test_args,
        parameters=parameters,
        timescale = ('1ns','1ps'),
        waves=True
    )
    run_test_args = []
    runner.test(
        hdl_toplevel="ether",
        test_module="ether_test",
        test_args=run_test_args,
        waves=True
    )

if __name__ == "__main__":
    main()
    # normal_to_network("1223485859669")