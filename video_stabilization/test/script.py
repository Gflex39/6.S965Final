import cv2
import numpy as np


def block_matching(frame1, frame2, block_size=16, search_area=7):
    h, w = frame1.shape
    motion_vectors = np.zeros((h // block_size, w // block_size, 2), dtype=np.int32)

    for i in range(0, h - block_size, block_size):
        for j in range(0, w - block_size, block_size):
            block = frame1[i : i + block_size, j : j + block_size]
            best_match = (0, 0)
            min_sad = float("inf")

            for x in range(-search_area, search_area + 1):
                for y in range(-search_area, search_area + 1):
                    ref_x, ref_y = i + x, j + y
                    if (
                        ref_x < 0
                        or ref_y < 0
                        or ref_x + block_size > h
                        or ref_y + block_size > w
                    ):
                        continue

                    ref_block = frame2[
                        ref_x : ref_x + block_size, ref_y : ref_y + block_size
                    ]
                    sad = np.sum(np.abs(block - ref_block))

                    if sad < min_sad:
                        min_sad = sad
                        best_match = (x, y)

            motion_vectors[i // block_size, j // block_size] = best_match

    return motion_vectors


def smooth_motion_vectors(motion_vectors, window_size=5):
    smoothed_vectors = np.copy(motion_vectors)
    for i in range(motion_vectors.shape[0]):
        for j in range(motion_vectors.shape[1]):
            x_vals = []
            y_vals = []
            for k in range(
                max(0, i - window_size // 2),
                min(motion_vectors.shape[0], i + window_size // 2 + 1),
            ):
                for l in range(
                    max(0, j - window_size // 2),
                    min(motion_vectors.shape[1], j + window_size // 2 + 1),
                ):
                    x_vals.append(motion_vectors[k, l, 0])
                    y_vals.append(motion_vectors[k, l, 1])
            smoothed_vectors[i, j, 0] = np.mean(x_vals)
            smoothed_vectors[i, j, 1] = np.mean(y_vals)
    return smoothed_vectors


def apply_motion_compensation(frame, motion_vectors, block_size=16):
    h, w = frame.shape
    compensated_frame = np.zeros_like(frame)

    for i in range(0, h - block_size, block_size):
        for j in range(0, w - block_size, block_size):
            dx, dy = motion_vectors[i // block_size, j // block_size]
            ref_x, ref_y = i + dx, j + dy
            if (
                ref_x < 0
                or ref_y < 0
                or ref_x + block_size > h
                or ref_y + block_size > w
            ):
                compensated_frame[i : i + block_size, j : j + block_size] = frame[
                    i : i + block_size, j : j + block_size
                ]
            else:
                compensated_frame[i : i + block_size, j : j + block_size] = frame[
                    ref_x : ref_x + block_size, ref_y : ref_y + block_size
                ]

    return compensated_frame


def stabilize_video(input_path, output_path):
    cap = cv2.VideoCapture(input_path)
    fourcc = cv2.VideoWriter_fourcc(*"XVID")
    out = None

    ret, prev_frame = cap.read()
    if not ret:
        print("Failed to read video")
        return

    prev_frame_gray = cv2.cvtColor(prev_frame, cv2.COLOR_BGR2GRAY)

    frame_count = 0

    while True:
        ret, frame = cap.read()
        if not ret:
            break

        frame_gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
        motion_vectors = block_matching(prev_frame_gray, frame_gray)
        smoothed_vectors = smooth_motion_vectors(motion_vectors)
        compensated_frame = apply_motion_compensation(frame_gray, smoothed_vectors)

        print("compensating frame", frame_count)

        if out is None:
            h, w = compensated_frame.shape
            out = cv2.VideoWriter(output_path, fourcc, 30.0, (w, h), isColor=False)

        out.write(compensated_frame)

        prev_frame_gray = frame_gray
        frame_count += 1

    cap.release()
    out.release()


# Example usage
stabilize_video("video/test.mp4", "video/stabilized_video.avi")
