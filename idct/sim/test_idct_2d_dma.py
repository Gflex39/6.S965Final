import numpy as np
import cocotb
import os
import random
import sys
import logging
from pathlib import Path
from cocotb.triggers import Timer
from cocotb.utils import get_sim_time as gst
from cocotb.runner import get_runner
from cocotb.triggers import RisingEdge, ClockCycles, FallingEdge, ReadOnly
from cocotb.clock import Clock
from cocotb_bus.bus import Bus
from cocotb_bus.drivers import BusDriver
from cocotb_bus.monitors import Monitor
from cocotb_bus.monitors import BusMonitor
import numpy as np
from PIL import Image


def rgb_to_yuv(frame):
    """
    Convert an RGB image to YUV in-place.

    The conversion is done according to the standard RGB to YUV conversion formula:
    Y = 0.299R + 0.587G + 0.114B
    U = 0.565(B-Y) + 128
    V = 0.713(R-Y) + 128

    Args:
        frame: numpy array of shape (height, width, 3) containing RGB values

    Returns:
        None (modifies input array in-place)
    """
    r = frame[:, :, 0].astype(np.float64)
    g = frame[:, :, 1].astype(np.float64)
    b = frame[:, :, 2].astype(np.float64)

    y = 0.299 * r + 0.587 * g + 0.114 * b
    u = 0.565 * (b - y) + 128.0
    v = 0.713 * (r - y) + 128.0

    frame[:, :, 0] = y.astype(np.uint8)
    frame[:, :, 1] = u.astype(np.uint8)
    frame[:, :, 2] = v.astype(np.uint8)


def get_yuv_frame(image_path):
    """
    Load an image from path and convert it to YUV format.

    Args:
        image_path: Path to the input image file

    Returns:
        numpy array of shape (height, width, 3) containing YUV values
    """
    # Read image using PIL to ensure consistent handling of different formats

    # Open and convert to RGB array
    img = Image.open(image_path)
    rgb_frame = np.array(img.convert("RGB"))

    # Convert to YUV
    yuv_frame = rgb_frame.copy()
    rgb_to_yuv(yuv_frame)

    height, width, channels = yuv_frame.shape
    print(f"Image dimensions: {width}x{height}, {channels} channels")

    # Split into separate channel arrays and reshape into 8x8 blocks
    height, width = yuv_frame.shape[:2]
    y_channel = (
        yuv_frame[:, :, 0]
        .reshape(height // 8, 8, width // 8, 8)
        .transpose(0, 2, 1, 3)
        .reshape(-1)
    )
    u_channel = (
        yuv_frame[:, :, 1]
        .reshape(height // 8, 8, width // 8, 8)
        .transpose(0, 2, 1, 3)
        .reshape(-1)
    )
    v_channel = (
        yuv_frame[:, :, 2]
        .reshape(height // 8, 8, width // 8, 8)
        .transpose(0, 2, 1, 3)
        .reshape(-1)
    )

    return y_channel, u_channel, v_channel


class AXISMonitor(BusMonitor):
    """
    monitors axi streaming bus
    """

    transactions = 0

    def __init__(self, dut, name, clk):
        self._signals = [
            "axis_tvalid",
            "axis_tready",
            "axis_tlast",
            "axis_tdata",
            "axis_tstrb",
        ]
        BusMonitor.__init__(self, dut, name, clk)
        self.clock = clk
        self.transactions = 0

        self.name = name

    async def _monitor_recv(self):
        """
        Monitor receiver
        """
        if self.name == "m00":
            pass
            # print("Monitor m00 AHHAHHAHAHAHHA")

        rising_edge = RisingEdge(self.clock)  # make these coroutines once and reuse
        falling_edge = FallingEdge(self.clock)
        read_only = ReadOnly()  # This is
        while True:
            await rising_edge
            # await falling_edge #sometimes see in AXI shit
            # await read_only  #readonly (the postline)
            valid = self.bus.axis_tvalid.value
            ready = self.bus.axis_tready.value
            last = self.bus.axis_tlast.value
            data = self.bus.axis_tdata.value

            if valid and ready:
                thing = dict(
                    data=data.signed_integer,
                    last=last,
                    name=self.name,
                    count=self.transactions,
                )
                print(thing)
                self.transactions += 1
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
        # print("Sending data", type(contents["data"]))

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


def combine_to_32bit(values):
    """
    Combines four values into a single 32-bit integer by treating each value
    as an 8-bit number and concatenating them.

    Args:
        values: List or tuple of 4 numbers to be combined

    Returns:
        Integer representing the combined 32-bit value

    Raises:
        ValueError: If input doesn't contain exactly 4 values
    """
    # First, verify we have exactly 4 values
    if len(values) != 4:
        raise ValueError("Must provide exactly 4 values")

    # Initialize our result as 0
    result = 0

    # Process each value in the array
    for i, value in enumerate(values):
        # Convert value which is of type uint8 to normal ints in python
        value = int(value)

        # Truncate to 8 bits by using bitwise AND with 0xFF (11111111 in binary)
        truncated = value & 0xFF

        # Shift the truncated value to its proper position
        # The first value needs to be shifted left by 24 bits
        # The second by 16, third by 8, and the last by 0
        shift_amount = 24 - (i * 8)
        shifted = truncated << shift_amount

        # Combine with our result using bitwise OR
        result |= shifted

    return result


@cocotb.test()
async def test_idct_2d_dma(dut):
    """cocotb test for idct_2d_dma"""
    inm = AXISMonitor(dut, "s00", dut.s00_axis_aclk)
    outm = AXISMonitor(dut, "m00", dut.s00_axis_aclk)
    ind = AXISDriver(dut, "s00", dut.s00_axis_aclk)
    cocotb.start_soon(Clock(dut.s00_axis_aclk, 10, units="ns").start())
    await set_ready(dut, 1)
    await reset(dut.s00_axis_aclk, dut.s00_axis_aresetn, 2, 1)

    # Single

    y_channel, u_channel, v_channel = get_yuv_frame("../../scripts/assets/dog.jpg")

    print("\n\n\n")
    print("Channels: ", y_channel[:4])

    channels = [y_channel]

    print("y_channel len:", len(y_channel))
    print("u_channel len:", len(u_channel))
    print("v_channel len:", len(v_channel))

    matches = []

    cnt = 0

    # Prepping for burst sending

    burst_data = []

    for i in range(len(channels)):
        channel = channels[i]
        for j in range(0, len(channel), 4):
            values = channel[j : j + 4]
            burst_data.append(combine_to_32bit(values))

            if cnt == 14720 - 1:
                matches.append(combine_to_32bit(values))
                cnt = 0
            else:
                cnt += 1

    data = {"type": "burst", "contents": {"data": burst_data}}
    ind.append(data)

    # for i in range(len(channels)):
    #     channel = channels[i]
    #     for j in range(0, len(channel), 4):
    #         values = channel[j : j + 4]
    #         data = {
    #             "type": "single",
    #             "contents": {
    #                 "data": combine_to_32bit(values),
    #                 "last": (
    #                     1 if j + 4 >= len(channel) and i == len(channels) - 1 else 0
    #                 ),
    #                 "strb": 15,
    #             },
    #         }
    #         ind.append(data)

    print(matches)

    # # Burst
    # data = {"type": "burst", "contents": {"data": np.array([0] * 14 + [1])}}
    # ind.append(data)

    # Back pressure test
    # await set_ready(dut, 0)
    # await ClockCycles(dut.s00_axis_aclk, 300)
    # await set_ready(dut, 1)
    # await ClockCycles(dut.s00_axis_aclk, 10)
    # await set_ready(dut, 0)
    # await ClockCycles(dut.s00_axis_aclk, 10)
    # await set_ready(dut, 1)
    # await ClockCycles(dut.s00_axis_aclk, 2000)

    await ClockCycles(dut.s00_axis_aclk, 117881)
    print(inm.transactions)
    print(outm.transactions)
    assert inm.transactions == outm.transactions, f"Transaction Count doesn't match! :/"

    # Additional assertions can be added here to verify FIR filter behavior
    # For example, you could compare the output data with expected FIR filter results


async def set_ready(dut, value):
    """Set the ready signal on the DUT."""
    dut.m00_axis_tready.value = value
    await RisingEdge(dut.s00_axis_aclk)


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


def idct_2d_dma_runner():
    """Simulate the counter using the Python runner."""
    hdl_toplevel_lang = os.getenv("HDL_TOPLEVEL_LANG", "verilog")
    sim = os.getenv("SIM", "icarus")
    proj_path = Path(__file__).resolve().parent.parent
    sys.path.append(str(proj_path / "sim" / "model"))
    sources = [proj_path / "hdl" / "idct_2d_dma.sv"]  # grow/modify this as needed.
    build_test_args = ["-Wall"]  # ,"COCOTB_RESOLVE_X=ZEROS"]
    parameters = {}
    sys.path.append(str(proj_path / "sim"))
    runner = get_runner(sim)
    runner.build(
        sources=sources,
        hdl_toplevel="idct_2d_dma",
        always=True,
        build_args=build_test_args,
        parameters=parameters,
        timescale=("1ns", "1ps"),
        waves=True,
    )
    run_test_args = []
    runner.test(
        hdl_toplevel="idct_2d_dma",
        test_module="test_idct_2d_dma",
        test_args=run_test_args,
        waves=True,
    )


if __name__ == "__main__":
    idct_2d_dma_runner()
