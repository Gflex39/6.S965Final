from scipy.fft import dctn
import random

import random


def generate_matrix():
    """
    Generates an 8x8 matrix with random integers between -4096 and 4095 (inclusive)
    using nested lists.

    Returns:
        list: 8x8 matrix represented as a nested list
    """
    return [[random.randint(-128, 127) for _ in range(8)] for _ in range(8)]


def test_idct_2d():
    matrix = generate_matrix()

    print("Matrix: [")
    for row in matrix:
        print("    [", end="")
        print(", ".join(map(str, row)), end="")
        print("],")
    print("]")

    idct_result = dctn(matrix, norm="ortho")
    print("DCT from Scipy: [")
    for row in idct_result.astype(int):
        print("    [", end="")
        print(", ".join(map(str, row)), end="")
        print("],")
    print("]")


test_idct_2d()
