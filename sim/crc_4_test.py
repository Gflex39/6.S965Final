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
import random
random.seed(1)
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

    await RisingEdge(self.clock)

    for data in value["contents"]["data"]:
        self.bus.axiiv.value = 1
        self.bus.axiid.value = data
        await read_only
        await RisingEdge(self.clock);

    self.bus.axiiv.value = 0
    self.bus.axiid.value = 0
    await ClockCycles(self.dut.clk, 1)
    await reset(self.dut.clk, self.dut.rst)
    await ClockCycles(self.dut.clk, 1)


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
    print("sdakodokasd\n\n")
    tester = EthernetTester(dut)
    await clock(dut.clk)
    await reset(dut.clk, dut.rst)
    await clock(dut.clk)
    preamble=["01010101"]*7
    sfd=["11010101"]
    source_address="00000000"*6
    
    print(source_address)
    destination_address=byte_pad(bin(int("BEEFDEADFEFE",16))[2:])
    source_address=[source_address[8*i:8*i+8] for i in range(len(source_address)//8)]
    destination_address=[destination_address[8*i:8*i+8] for i in range(len(destination_address)//8)]


    packets=[ integer_to_packet_data(random.randint(5,5)) for _ in range(5)] +[['00000000']+['00101110']+['10000001']*46]                                                                                    #['00100111']+['11010011']+['10110010']+['10001101']
                                                                                                #['10101110']+['11001110']+['00100001']+['10010001']
    for data in packets:
        # print(print(int("".join(packet[:-4]),2)))
        # print(hex(int("".join(packet),2)))

        
        # print("3 different FCS")
        packet=destination_address+source_address+data
        print("**")
        print(hex(int("".join(packet),2)))
        network_packet="".join([normal_to_network(byte) for byte in packet])
        print("Hello: "+(hex(int("".join(network_packet),2))))
        # network_packet="".join(packet)
        
        # print(hex(int(network_packet,2))
        # print("---")
        fcs=crc32_calculator(int(network_packet,2))
        if len(fcs)%8!=0:
            fcs="0"*(8-(len(fcs))%8)+fcs
        print(fcs)
        
        print(hex(int(crc32_calculator(int(network_packet+fcs,2)),2)))
        print(hex(int(fcs,2)))
        
        
        #
        # print(hex(int(fcs,2)))
        fcs=[fcs[8*i:8*i+8] for i in range(len(fcs)//8)]
        # print(len(fcs))
        # print()
        # t=hex(int("".join(fcs),2))

        # print(f"FCS is {t}")

        # print(hex(int(network_packet,2)))
        # print(hex(int("".join(packet),2)))
        fcs=[i[::-1] for i in fcs]
        print(hex(int("".join([normal_to_dibit_network(byte) for byte in fcs]),2)))
        
        # network_packet="".join([byte[::-1] for byte in packet])
        network_packet="".join([normal_to_dibit_network(byte) for byte in packet])
        # network_packet="".join(network_packet)
        print(hex(int(network_packet,2)))
        print("--")
        di_bits=[network_packet[4*i:4*i+4][::-1] for i in range(len(network_packet)//4)]
        tester.input_driver.append({ "type": "burst", "contents": { "data": [int(i,2) for i in di_bits] } })

    await ClockCycles(dut.clk, 700)
    # print(hex(int(dut.axiod.value)))

def normal_to_network(byte):
    return byte[::-1]

def normal_to_dibit_network(byte):
    # return byte[::-1]
    split_byte=[byte[4*i:4*i+4] for i in range(len(byte)//4)]
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
        # print(hex(crc & 0xffffffff))
    crc = ~crc
    crc &= 0xffffffff
    return bin(crc)[2:]

def byte_pad(s):
    s_padded=s
    if len(s)%8!=0:
        s_padded="0"*(8-(len(s))%8)+s
    return s_padded

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

    

   




def main():
    hdl_toplevel_lang = os.getenv("HDL_TOPLEVEL_LANG", "verilog")
    sim = os.getenv("SIM", "icarus")
    proj_path = Path(__file__).resolve().parent.parent
    sys.path.append(str(proj_path / "sim" / "model"))
    sources = [proj_path / "hdl" / "crc32_4.sv"]
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
        test_module="crc_4_test",
        test_args=run_test_args,
        waves=True
    )

if __name__ == "__main__":
    # print("fefmwkjfwekjn")
    main()
    # print(crc32_calculator(3735928559))
    # print(normal_to_network("00111000111110110010001010000100"))
    # ebfe|eddaefefeddaebfeeded10101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010
    # ebfe|eddaefef00000000000000e21010101010101010101010101010101010101010101010101010101010101010101010101010101010101010
    