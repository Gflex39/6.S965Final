import numpy as np
from scipy.fft import dct, idct
from utils import decimal_to_twos_complement

# CONSTANTS
A = np.cos(np.pi / 4)
B = np.cos(np.pi / 8)
C = np.sin(np.pi / 8)
D = np.cos(np.pi / 16)
E = np.cos(3 * np.pi / 16)
F = np.sin(3 * np.pi / 16)
G = np.sin(np.pi / 16)


def chen_dct(x):
    # Ensure input is numpy array
    x = np.array(x)

    # First stage calculations
    # Upper matrix operation
    x0_plus_x7 = x[0] + x[7]
    x1_plus_x6 = x[1] + x[6]
    x2_plus_x5 = x[2] + x[5]
    x3_plus_x4 = x[3] + x[4]

    # Lower matrix operation
    x0_minus_x7 = x[0] - x[7]
    x1_minus_x6 = x[1] - x[6]
    x2_minus_x5 = x[2] - x[5]
    x3_minus_x4 = x[3] - x[4]

    # Upper matrix multiplication
    X_upper = (
        np.array(
            [
                A * x0_plus_x7 + A * x1_plus_x6 + A * x2_plus_x5 + A * x3_plus_x4,
                B * x0_plus_x7 + C * x1_plus_x6 - C * x2_plus_x5 - B * x3_plus_x4,
                A * x0_plus_x7 - A * x1_plus_x6 - A * x2_plus_x5 + A * x3_plus_x4,
                C * x0_plus_x7 - B * x1_plus_x6 + B * x2_plus_x5 - C * x3_plus_x4,
            ]
        )
        / 2
    )

    # Lower matrix multiplication
    X_lower = (
        np.array(
            [
                D * x0_minus_x7 + E * x1_minus_x6 + F * x2_minus_x5 + G * x3_minus_x4,
                E * x0_minus_x7 - G * x1_minus_x6 - D * x2_minus_x5 - F * x3_minus_x4,
                F * x0_minus_x7 - D * x1_minus_x6 + G * x2_minus_x5 + E * x3_minus_x4,
                G * x0_minus_x7 - F * x1_minus_x6 + E * x2_minus_x5 - D * x3_minus_x4,
            ]
        )
        / 2
    )

    # Combine results in correct order
    result = np.zeros(8)
    result[0::2] = X_upper
    result[1::2] = X_lower

    return result


def chen_idct(X):
    # Ensure input is numpy array
    X = np.array(X)

    # Split into even and odd indices
    X_even = X[0::2]
    X_odd = X[1::2]

    # Upper matrix multiplication
    x_upper = (
        np.array(
            [
                A * X_even[0] + B * X_even[1] + A * X_even[2] + C * X_even[3],
                A * X_even[0] + C * X_even[1] - A * X_even[2] - B * X_even[3],
                A * X_even[0] - C * X_even[1] - A * X_even[2] + B * X_even[3],
                A * X_even[0] - B * X_even[1] + A * X_even[2] - C * X_even[3],
            ]
        )
        / 2
    )

    # Lower matrix multiplication
    x_lower = (
        np.array(
            [
                D * X_odd[0] + E * X_odd[1] + F * X_odd[2] + G * X_odd[3],
                E * X_odd[0] - G * X_odd[1] - D * X_odd[2] - F * X_odd[3],
                F * X_odd[0] - D * X_odd[1] + G * X_odd[2] + E * X_odd[3],
                G * X_odd[0] - F * X_odd[1] + E * X_odd[2] - D * X_odd[3],
            ]
        )
        / 2
    )

    # Combine results to get the reconstructed signal
    result = np.zeros(8)
    result[0:4] = x_upper + x_lower
    result[7:3:-1] = x_upper - x_lower

    return result


# Example usage
if __name__ == "__main__":
    # Test with sample input
    x = np.array([16, 11, 10, 16, 24, 40, 51, 61])

    # Perform DCT
    X = chen_dct(x)
    print("DCT coefficients:", X)

    # SciPy DCT
    Y = dct(x, norm="ortho")
    print("SciPy DCT coefficients:", Y)

    for coeff in X:
        print(decimal_to_twos_complement(int(coeff), 12))

    # Perform Chen's inverse DCT
    x_reconstructed = chen_idct(X)
    print("Reconstructed signal:", x_reconstructed)

    # SciPy inverse DCT
    x_reconstructed_scipy = idct(Y, norm="ortho")
    print("Reconstructed signal (SciPy):", x_reconstructed_scipy)
