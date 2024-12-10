import numpy as np


def reshape_yuv_data(raw_data):
    # First, split the data into Y, U, and V components
    y_data = raw_data[:235520]  # 640 x 368 pixels
    u_data = raw_data[235520 : 235520 + (320 * 184)]  # 320 x 184 pixels
    v_data = raw_data[
        235520 + (320 * 184) : 235520 + 2 * (320 * 184)
    ]  # 320 x 184 pixels

    # Initialize the output arrays
    y_array = np.zeros((368, 640), dtype=np.uint8)
    u_array = np.zeros((184, 320), dtype=np.uint8)
    v_array = np.zeros((184, 320), dtype=np.uint8)

    # Process Y channel
    block_idx = 0
    for row in range(0, 368, 8):
        for col in range(0, 640, 8):
            block_data = y_data[block_idx * 64 : (block_idx + 1) * 64]
            # Reshape the block from 1D to 8x8
            block_2d = np.reshape(block_data, (8, 8))
            y_array[row : row + 8, col : col + 8] = block_2d
            block_idx += 1

    # Process U channel
    block_idx = 0
    for row in range(0, 184, 8):
        for col in range(0, 320, 8):
            block_data = u_data[block_idx * 64 : (block_idx + 1) * 64]
            block_2d = np.reshape(block_data, (8, 8))
            u_array[row : row + 8, col : col + 8] = block_2d
            block_idx += 1

    # Process V channel
    block_idx = 0
    for row in range(0, 184, 8):
        for col in range(0, 320, 8):
            block_data = v_data[block_idx * 64 : (block_idx + 1) * 64]
            block_2d = np.reshape(block_data, (8, 8))
            v_array[row : row + 8, col : col + 8] = block_2d
            block_idx += 1

    return y_array, u_array, v_array


# Usage example:
# Assuming your input data is in a numpy array called 'raw_data'
# y_channel, u_channel, v_channel = reshape_yuv_data(raw_data)
