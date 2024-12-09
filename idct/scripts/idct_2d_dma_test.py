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


def yuv_to_rgb(frame):
    """
    Convert a YUV image to RGB in-place.

    The conversion is done according to the standard YUV to RGB conversion formula:
    R = Y + 1.4903 * (V - 128)
    G = Y - 0.344 * (U - 128) - 0.714 * (V - 128)
    B = Y + 1.770 * (U - 128)

    Args:
        frame: numpy array of shape (height, width, 3) containing YUV values

    Returns:
        None (modifies input array in-place)
    """
    y = frame[:, :, 0].astype(np.float64)
    u = frame[:, :, 1].astype(np.float64)
    v = frame[:, :, 2].astype(np.float64)

    r = y + 1.4903 * (v - 128.0)
    g = y - 0.344 * (u - 128.0) - 0.714 * (v - 128.0)
    b = y + 1.770 * (u - 128.0)

    frame[:, :, 0] = np.clip(r, 0, 255).astype(np.uint8)
    frame[:, :, 1] = np.clip(g, 0, 255).astype(np.uint8)
    frame[:, :, 2] = np.clip(b, 0, 255).astype(np.uint8)


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


print(get_yuv_frame("assets/dog.jpg"))
