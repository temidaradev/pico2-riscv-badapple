# 02: Bad Apple RLE Compression

Fitting 3 minutes of video (6500+ frames) into a microcontroller's Flash memory is impossible if you store raw pixels.

## Why Compress?
- **Raw size**: 128x64 px = 1024 bytes per frame.
- **Total size**: 1024 bytes * 6572 frames ≈ **6.7 MB**.
- **RP2350 Flash**: Usually limited to 2MB or 4MB.

We use **Run-Length Encoding (RLE)** to shrink the video to ~1.6MB.

## The Custom RLE Protocol
The algorithm used in `main.rs` is a state-based decoder.

### 1. The Core Idea
Instead of saying `White, White, White, White`, we say `4x White`.

### 2. State Machine in `decode_rle`
The decoder looks for special "Control Bytes":
- **`0x55`**: Signifies a run of **Black** pixels.
- **`0xAA`**: Signifies a run of **White** pixels.

### 3. Step-by-Step Logic
When the decoder sees a byte:
1. **If it's not in a run**: It checks if the byte is `0x55` or `0xAA`.
2. **If it is a control byte**: The *next* byte tells us the **length** of the run.
3. **Variable Length**:
   - If the length byte has the top bit (`0x80`) clear, it's a short run.
   - If the top bit is set, it's part of a multi-byte run length (allowing runs longer than 127 pixels).

### 4. Code Implementation
```rust
fn put_bytes(frame_buffer: &mut [u8; 1024], byte: u8, count: i32, bytes_written: &mut usize) {
    let end = (*bytes_written + count as usize).min(1024);
    frame_buffer[*bytes_written..end].fill(byte);
    *bytes_written = end;
}
```
This function is called by the decoder to "splat" a color across the buffer efficiently using Rust's `slice::fill`.

## Why this is fast
Because we use `slice::fill`, the compiler can turn this into highly optimized assembly (often using SIMD-like instructions) rather than a slow manual `for` loop.
