import numpy as np
import cocotb
import os
import sys
import logging
from pathlib import Path
from cocotb.runner import get_runner
from cocotb.triggers import RisingEdge, ClockCycles, FallingEdge, ReadOnly
from cocotb.clock import Clock
from cocotb_bus.drivers import BusDriver
from cocotb_bus.scoreboard import Scoreboard
from cocotb_bus.monitors import BusMonitor
from cocotb.handle import SimHandleBase
from cocotb.binary import BinaryValue
import numpy as np
import random
import matplotlib.pyplot as plt
import numpy as np
from scipy.fft import idct


class AXISMonitor(BusMonitor):
    """
    monitors axi streaming bus
    """

    transactions = 0

    def __init__(self, dut, name, clk, callback=None):
        self._signals = [
            "axis_tvalid",
            "axis_tready",
            "axis_tlast",
            "axis_tdata",
            "axis_tstrb",
        ]
        BusMonitor.__init__(self, dut, name, clk, callback=callback)
        self.clock = clk
        self.transactions = 0
        self.values = []

    async def _monitor_recv(self):
        """
        Monitor receiver
        """
        rising_edge = RisingEdge(self.clock)  # make these coroutines once and reuse
        falling_edge = FallingEdge(self.clock)
        read_only = ReadOnly()  # This is
        while True:
            await rising_edge
            await falling_edge
            await read_only  # readonly (the postline)
            valid = self.bus.axis_tvalid.value
            ready = self.bus.axis_tready.value
            last = self.bus.axis_tlast.value
            data = self.bus.axis_tdata.value
            if valid and ready:
                self.transactions += 1
                thing = dict(
                    data=data, last=last, name=self.name, count=self.transactions
                )
                self.values.append(data.signed_integer)
                #   print(thing)
                self._recv(thing)


class AXISDriver(BusDriver):
    def __init__(self, dut, name, clk):
        self._signals = [
            "axis_tvalid",
            "axis_tready",
            "axis_tlast",
            "axis_tdata",
            "axis_tstrb",
        ]
        BusDriver.__init__(self, dut, name, clk)
        self.clock = clk
        self.bus.axis_tdata.value = 0
        self.bus.axis_tstrb.value = 0
        self.bus.axis_tlast.value = 0
        self.bus.axis_tvalid.value = 0

    async def _driver_send(self, value, sync=True):
        # if sync:
        #     await RisingEdge(self.clock)

        transaction = value
        transaction_type = transaction["type"]
        contents = transaction["contents"]

        if transaction_type == "single":
            await self._send_single(contents)
        elif transaction_type == "burst":
            await self._send_burst(contents)

    async def _send_single(self, contents):
        await FallingEdge(self.clock)
        self.bus.axis_tdata.value = contents["data"]
        self.bus.axis_tstrb.value = contents["strb"]
        self.bus.axis_tlast.value = contents["last"]
        self.bus.axis_tvalid.value = 1

        while True:
            await RisingEdge(self.clock)
            if self.bus.axis_tready.value == 1:
                break

        await FallingEdge(self.clock)
        self.bus.axis_tvalid.value = 0

    async def _send_burst(self, contents):
        data_array = contents["data"]
        for i, data in enumerate(data_array):
            await FallingEdge(self.clock)
            self.bus.axis_tdata.value = int(data)
            self.bus.axis_tstrb.value = 15  # Assuming full byte valid
            self.bus.axis_tlast.value = 1 if i == len(data_array) - 1 else 0
            self.bus.axis_tvalid.value = 1

            while True:
                await RisingEdge(self.clock)
                if self.bus.axis_tready.value == 1:
                    break

        await FallingEdge(self.clock)
        self.bus.axis_tvalid.value = 0


class SqrtScoreboard(Scoreboard):
    def compare(self, got, exp, log, strict_type=True):
        # print(f'GOT: {got}, EXP: {exp}')
        if (
            abs(got["data"].signed_integer - exp["data"].signed_integer) <= 1
        ):  # change to whatever you want for the problem at hand.
            # Don't want to fail the test
            # if we're passed something without __len__
            try:
                log.debug("Received expected transaction %d bytes" % (len(got)))
                log.debug(repr(got))
            except Exception:
                pass
        else:
            self.errors += 1
            # Try our best to print out something useful
            strgot, strexp = str(got), str(exp)
            log.error("Received transaction differed from expected output")
            log.info("Expected:\n" + repr(exp))
            log.info("Received:\n" + repr(got))
            if self._imm:
                assert False, (
                    "Received transaction differed from expected " "transaction"
                )


class SSSTester:
    """
    Checker of a split square sum instance
    Args
      dut_entity: handle to an instance of split-square-sum
    """

    def __init__(self, dut_entity: SimHandleBase, debug=False):
        self.dut = dut_entity
        self.log = logging.getLogger("cocotb.tb")
        self.log.setLevel(logging.DEBUG)
        self.input_mon = AXISMonitor(self.dut, "s00", self.dut.clk_in)
        self.output_mon = AXISMonitor(self.dut, "m00", self.dut.clk_in)
        self.input_driver = AXISDriver(self.dut, "s00", self.dut.clk_in)
        self._checker = None
        self.calcs_sent = 0

        # # Create a scoreboard on the stream_out bus
        # self.expected_output = []  # contains list of expected outputs (Growing)
        # self.scoreboard = SqrtScoreboard(self.dut, fail_immediately=False)
        # self.scoreboard.add_interface(self.output_mon, self.expected_output)

    def stop(self) -> None:
        """Stops everything"""
        if self._checker is None:
            raise RuntimeError("Monitor never started")
        self.input_mon.stop()
        self.output_mon.stop()
        self.input_driver.stop()

    def model(self, transaction):
        data = transaction["data"]
        result = transaction.copy()
        result["name"] = "m00"

        self.expected_output.append(result)

    def plot_i_q_time_series(self, top, bott, length):
        # Create time array
        t = np.arange(length)

        # Create figure and axis objects
        fig, ax = plt.subplots(figsize=(10, 6))

        # Plot I and Q components
        ax.plot(t, top, label="I", color="blue")
        ax.plot(t, bott, label="Q", color="orange")

        ax.set_xlabel("Sample")
        ax.set_ylabel("Amplitude")
        ax.set_title("I and Q Components")
        ax.legend()
        ax.grid(True)

        # Adjust layout and display plot
        plt.tight_layout()
        plt.show()

    def plot_result(self, length):
        # input_vals = np.array(self.input_mon.values)
        output_vals = np.array(self.output_mon.values)
        print("output_vals: ", output_vals)
        top = ((output_vals >> 16) & 0xFFFF).astype(np.int16)
        bott = (output_vals & 0xFFFF).astype(np.int16)
        print(top)  # for sanity checking
        print(bott)  # for sanity checking
        self.plot_i_q_time_series(top, bott, length)


@cocotb.test()
async def test_idct_1d(dut):
    """cocotb test for idct_1d"""
    cocotb.start_soon(Clock(dut.clk_in, 10, units="ns").start())

    # await set_ready(dut, 1)
    await reset(dut.clk_in, dut.rst_in, 2, 1)

    await ClockCycles(dut.clk_in, 2000)

    # Create a 96-bit test input value
    test_input = [461, -194, 32, -2, 2, 0, -9, 8]
    await input_data(dut, test_input)

    # Wait for output
    await ClockCycles(dut.clk_in, 6)

    # Read the output value
    idct_out_0 = dut.idct_out_0.value
    idct_out_1 = dut.idct_out_1.value
    idct_out_2 = dut.idct_out_2.value
    idct_out_3 = dut.idct_out_3.value
    idct_out_4 = dut.idct_out_4.value
    idct_out_5 = dut.idct_out_5.value
    idct_out_6 = dut.idct_out_6.value
    idct_out_7 = dut.idct_out_7.value

    print(
        f"IDCT Output: {int(idct_out_0)}, {int(idct_out_1)}, {int(idct_out_2)}, {int(idct_out_3)}, {int(idct_out_4)}, {int(idct_out_5)}, {int(idct_out_6)}, {int(idct_out_7)}"
    )

    print("Inverse IDCT from Scipy: ", idct(test_input, norm="ortho"))


async def input_data(dut, value):
    """Input data to the DUT."""

    dut.idct_in_0.value = value[0]
    dut.idct_in_1.value = value[1]
    dut.idct_in_2.value = value[2]
    dut.idct_in_3.value = value[3]
    dut.idct_in_4.value = value[4]
    dut.idct_in_5.value = value[5]
    dut.idct_in_6.value = value[6]
    dut.idct_in_7.value = value[7]

    await RisingEdge(dut.clk_in)


async def reset(clk, rst, duration, active_low=True):
    """Reset the DUT."""
    if active_low:
        rst.value = 0
    else:
        rst.value = 1
    await ClockCycles(clk, duration)
    if active_low:
        rst.value = 1
    else:
        rst.value = 0
    await RisingEdge(clk)


def idct_1d_runner():
    """Simulate the counter using the Python runner."""
    hdl_toplevel_lang = os.getenv("HDL_TOPLEVEL_LANG", "verilog")
    sim = os.getenv("SIM", "icarus")
    proj_path = Path(__file__).resolve().parent.parent
    sys.path.append(str(proj_path / "sim" / "model"))

    print("\n\n\n\n\n\nproj_path:", proj_path)

    # # Add the current directory to Python path
    # current_dir = Path(__file__).resolve().parent
    # sys.path.append(str(current_dir))

    sources = [
        proj_path / "hdl" / "idct_1d.sv",
    ]  # Added mixer.sv, sine_generator.sv to the sources
    build_test_args = ["-Wall"]  # ,"COCOTB_RESOLVE_X=ZEROS"]
    parameters = {}
    sys.path.append(str(proj_path / "sim"))
    runner = get_runner(sim)
    runner.build(
        sources=sources,
        hdl_toplevel="idct_1d",
        always=True,
        build_args=build_test_args,
        parameters=parameters,
        timescale=("1ns", "1ps"),
        waves=True,
    )
    run_test_args = []
    runner.test(
        hdl_toplevel="idct_1d",
        test_module="test_idct_1d",
        test_args=run_test_args,
        waves=True,
    )


if __name__ == "__main__":
    idct_1d_runner()
