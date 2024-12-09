def rgb_to_ycrcb(r: float, g: float, b: float) -> tuple[float, float, float]:
    """
    Convert RGB color values to YCrCb color space.

    Args:
        r (float): Red component (0-255)
        g (float): Green component (0-255)
        b (float): Blue component (0-255)

    Returns:
        tuple[float, float, float]: YCrCb values (Y, Cr, Cb)
    """
    # Convert RGB values to [0,1] range
    r, g, b = r / 255.0, g / 255.0, b / 255.0

    # Calculate Y (luminance)
    y = 0.299 * r + 0.587 * g + 0.114 * b

    # Calculate Cr and Cb (chrominance)
    cr = (r - y) * 0.713 + 0.5
    cb = (b - y) * 0.564 + 0.5

    # Scale Y to [0,1] and Cr,Cb to [0,1]
    return (y, cr, cb)


# Example usage
rgb = (255, 128, 0)  # Orange color
y, cr, cb = rgb_to_ycrcb(*rgb)
print(f"Y: {y:.3f}, Cr: {cr:.3f}, Cb: {cb:.3f}")
