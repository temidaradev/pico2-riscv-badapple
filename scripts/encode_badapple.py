#!/usr/bin/env python3
"""
Encode badapple.raw into a frame-local RLE stream.

Input:
- badapple.raw: 1 byte per pixel (0x00 or 0xFF)

Output:
- badapple.rle: marker-based RLE stream
"""

import sys

IN  = sys.argv[1] if len(sys.argv) > 1 else "badapple.raw"
OUT = sys.argv[2] if len(sys.argv) > 2 else "badapple.rle"

MARKER_ZERO = 0x55
MARKER_ONE  = 0xAA

MAX_RUN = 0x7FFF
FRAME_SIZE = 1024


def emit_run(out, value, length):
    marker = MARKER_ZERO if value == 0x00 else MARKER_ONE
    while length > 0:
        chunk = min(length, MAX_RUN)
        out.append(marker)

        if chunk <= 127:
            out.append(chunk)
        else:
            out.append((chunk & 0x7F) | 0x80)
            out.append((chunk >> 7) & 0xFF)

        length -= chunk


def emit_literal(out, byte):
    if byte in (MARKER_ZERO, MARKER_ONE):
        out.append(byte)
        out.append(0x00)   # escape
    else:
        out.append(byte)


with open(IN, "rb") as f:
    data = f.read()

out = bytearray()

offset = 0
while offset < len(data):
    frame = data[offset:offset + FRAME_SIZE]
    offset += FRAME_SIZE

    i = 0
    while i < len(frame):
        v = frame[i]

        if v in (0x00, 0xFF):
            j = i + 1
            while j < len(frame) and frame[j] == v:
                j += 1

            run_len = j - i
            if run_len >= 2:
                emit_run(out, v, run_len)
            else:
                emit_literal(out, v)

            i = j
        else:
            emit_literal(out, v)
            i += 1


with open(OUT, "wb") as f:
    f.write(out)

print(f"Wrote {len(out)} bytes to {OUT}")
