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


def yuv2rgb(y, u, v):
    u = u.repeat(2, axis=0).repeat(2, axis=1)
    v = v.repeat(2, axis=0).repeat(2, axis=1)

    # return y, u, v

    # yuv = np.dstack(y, u, v)

    # # Create the conversion matrix based on the equations
    # rgb = np.zeros_like(yuv)

    # # Apply the conversion equations
    # rgb[:, :, 0] = yuv[:, :, 0] + 1.4903 * (yuv[:, :, 2] - 128)  # R channel
    # rgb[:, :, 1] = (
    #     yuv[:, :, 0] - 0.344 * (yuv[:, :, 1] - 128) - 0.714 * (yuv[:, :, 2] - 128)
    # )  # G channel
    # rgb[:, :, 2] = yuv[:, :, 0] + 1.770 * (yuv[:, :, 1] - 128)  # B channel

    # # Clip values to ensure they're in valid range [0, 255]
    # rgb = np.clip(rgb, 0, 255)

    # # Convert to uint8 for standard image format
    # return rgb.astype(np.uint8)


def print_blocks(array):
    blocks = []
    for i in range(0, array.shape[0], 8):
        for j in range(0, array.shape[1], 8):
            blocks.append(array[i : i + 8, j : j + 8])

    return blocks


if __name__ == "__main__":
    r = np.arange(0, 3680 * 64, dtype=np.uint8)
    g = np.arange(0, 920 * 64, dtype=np.uint8)
    b = np.arange(0, 920 * 64, dtype=np.uint8)

    rgb = np.concatenate((r, g, b))

    # print(rgb)

    result = yuv2rgb(*reshape_yuv_data(rgb))

    print(result[1].shape)
    print(print_blocks(result[1])[0])
    # print(result[0])

    pass

# Usage example:
# Assuming your input data is in a numpy array called 'raw_data'
# y_channel, u_channel, v_channel = reshape_yuv_data(raw_data)
