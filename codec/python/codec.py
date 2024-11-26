#!/usr/bin/env python3

from bitarray import bitarray, decodetree
from rich.progress import Progress, TimeElapsedColumn

import numpy as np
import cv2 as cv
import pickle
import typer

H = pickle.load(open('data/huffman.pkl', 'rb'))

M0 = np.array([13107,11916,10082,9362,8192,7282])
M1 = np.array([5243,4660,4194,3647,3355,2893])
M2 = np.array([8066,7490,6554,5825,5243,4559])

def M(qp: int) -> np.ndarray:
    m0 = M0[qp%6]
    m1 = M1[qp%6]
    m2 = M2[qp%6]
    return np.array([[m0,m2,m0,m2],[m2,m1,m2,m1],[m0,m2,m0,m2],[m2,m1,m2,m1]])

def quantize(block: np.ndarray, qp: int) -> np.ndarray:
    C = np.array([[1,1,1,1],[2,1,-1,-2],[1,-1,-1,1],[1,-2,2,-1]])
    return np.fix(((C@block@C.T*M(qp)))/(2.**(15+(qp//6)))).astype(np.int64)

z_rows = np.array([0,0,1,2,1,0,0,1,2,3,3,2,1,2,3,3])
z_cols = np.array([0,1,0,0,1,2,3,2,1,0,1,2,3,3,2,3])

def zigzag(X: np.ndarray) -> np.ndarray:
    return X.reshape(-1,4,4)[:,z_rows,z_cols].reshape(-1,16)

def unzigzag(Y: np.ndarray) -> np.ndarray:
    Y = Y.reshape(-1,16)
    X = np.zeros_like(Y).reshape(-1,4,4)
    X[:,z_rows,z_cols] = Y[:,:]
    return X

def count_zeros_between(X: np.ndarray) -> np.ndarray:
    nonzero_indices = np.nonzero(X)
    result = np.zeros_like(X, dtype=np.int64)
    diff = np.diff(nonzero_indices[1], prepend=-1)
    result[nonzero_indices] = diff - 1
    result[result < 0] = 0
    return result

def run_size_encode(X: np.ndarray) -> np.ndarray:
    return np.dstack([count_zeros_between(X), X])

def entropy_code(X: np.ndarray, s: bitarray):
    Y = zigzag(X)
    Z = run_size_encode(Y)
    for block in Z.reshape(-1, 16, 2):
        written = 0
        for run, value in block:
            if run == 0 and value == 0 and written != 0:
                continue
            s.encode(H, [(run, value)])
            written += run + 1
        if written != 16:
            s.encode(H, [(np.int64(0),np.int64(0))])

def encode_plane(plane: np.ndarray, qp: int, s: bitarray):
    height, width = plane.shape
    hblocks, wblocks = height//4, width//4
    blocks = (
        plane
            .reshape(hblocks, 4, wblocks, 4)
            .transpose(0,2,1,3)
            .reshape(-1,4,4)
    )
    blocks = quantize(blocks, qp)
    blocks[1:,0,0] = blocks[1:,0,0] - blocks[:-1,0,0]
    entropy_code(blocks, s)

def encode_frame(yuv: np.ndarray, qp: int, s: bitarray):
    y,u,v = cv.split(yuv)
    u = u[::2,::2]
    v = v[::2,::2]
    encode_plane(y, qp, s)
    encode_plane(u, qp, s)
    encode_plane(v, qp, s)

app = typer.Typer()

@app.command()
def encode(infile: str, outfile: str, qp: int = 24):
    """
    Encode a video file using JPEG compression.
    """
    s = bitarray()
    cap = cv.VideoCapture(infile)

    if not cap.isOpened():
        exit(1)
        return

    total_frames = int(cap.get(cv.CAP_PROP_FRAME_COUNT))

    columns = [ *Progress.get_default_columns(), TimeElapsedColumn() ]

    with Progress(*columns) as progress:
        task = progress.add_task("[blue]Encoding...[/blue]", total=total_frames)
        for i in range(total_frames):
            progress.update(task, advance=1, description=f"[blue]Encoding frame {i+1}/{total_frames}[/blue]")
            ret, frame = cap.read()
            if not ret: break
            yuv = cv.cvtColor(frame, cv.COLOR_BGR2YUV)
            encode_frame(yuv, qp, s)

    with open(outfile, "wb") as f:
        s.tofile(f)

    cap.release()
    cv.destroyAllWindows()

@app.command()
def decode(file: str):
    """
    Decode a video file using JPEG compression.
    """
    print(file)

if __name__ == "__main__":
    app()