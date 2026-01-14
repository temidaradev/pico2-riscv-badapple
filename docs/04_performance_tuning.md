# 04: Performance Tuning

Embedded systems have limited resources. Achieving smooth 30 FPS video requires squeezing every bit of speed out of the hardware and compiler.

## 1. The Bottleneck: I2C
By default, I2C is very slow (100kHz or 400kHz).
- At 400kHz, sending 1KB (one frame) takes ~25ms.
- 30 FPS requires a frame every 33.3ms.
- 20 FPS (our current setting) requires a frame every 50ms.
- 25ms (transfer) + ~15ms (decode) = **40ms per frame** (Max ~25 FPS at 400kHz).

**Solution**: Increase I2C to **1000kHz (1MHz)**. This drops transfer time to ~10ms, leaving 23ms for decoding.

## 2. Compiler Optimizations
In Rust, the `debug` build (unoptimized) is many times slower than `release`.
- **`opt-level = 3`**: We added this to `[profile.dev]` in `Cargo.toml`. This tells the compiler to optimize bitwise operations even during development.
- **`#[inline(always)]`**: We applied this to the RLE decoder. It removes the overhead of jumping into a function, which adds up when called thousands of times per frame.

## 3. Frame Rate Limiter
If the CPU is too fast, the video will look like a "fast-forward" movie. We use the hardware timer to cap the speed at 20 FPS (50,000 microseconds):

```rust
let elapsed = (timer.get_counter() - frame_start).to_micros();
if elapsed < 50_000 {
    timer.delay_us(50_000 - elapsed as u32);
}
```

## 4. Reducing Overhead
- **Removed Debug Logging**: Printing "Frame Drawn" to the console every frame is a blocking operation. It can slow down the loop significantly.
- **Slice Filling**: Using `.fill(byte)` on a slice is faster than a loop because it uses optimized assembly patterns.
