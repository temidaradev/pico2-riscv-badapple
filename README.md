# Bad Apple on RP2350 (RISC-V) with SSD1306

![IMG_6729](https://github.com/user-attachments/assets/87fdb751-242b-4b91-9770-496c9ee19e0a)

This project demonstrates playing the iconic "Bad Apple" video on a Raspberry Pi Pico 2 (RP2350) using the RISC-V hazard3 cores. It uses an SSD1306 OLED display over I2C and a custom Run-Length Encoding (RLE) format to fit the video into the chip's memory.

## Special Learning Guides
We've prepared detailed deep-dives into the core concepts used in this project:
1. [Embedded Rust Setup & RISC-V](docs/01_embedded_rust_setup.md)
2. [Bad Apple RLE Compression](docs/02_rle_compression.md)
3. [SSD1306 and I2C Internals](docs/03_ssd1306_i2c.md)
4. [Performance Tuning](docs/04_performance_tuning.md)
5. [Special Rust Patterns (Embedded)](docs/05_rust_embedded_patterns.md)

## Table of Contents
- [Architecture](#architecture)
- [How it Works](#how-it-works)
    - [RLE Decoding](#rle-decoding)
    - [Display Interfacing](#display-interfacing)
    - [Performance Optimizations](#performance-optimizations)
- [Hardware Setup](#hardware-setup)
- [Running the Project](#running-the-project)

## Architecture

- **Microcontroller**: RP2350 (Dual-core RISC-V Hazard3 / Cortex-M33). This project specifically targets the **RISC-V** architecture.
- **Language**: Rust (`no_std` environment).
- **HAL**: `rp235x-hal` for peripheral access.
- **Display**: SSD1306 (128x64 OLED) via I2C.

## How it Works

### RLE Decoding
The video is stored in `badapple.rle`, included directly in the binary using `include_bytes!`. To save space, the frames are compressed using a custom RLE algorithm:
1. **Control Bytes**: Specifically `0x55` and `0xAA` indicate the start of a run of pixels.
2. **Expansion**: The decoder processes these runs to fill a 1024-byte frame buffer (128x64 pixels = 8192 bits = 1024 bytes).
3. **Efficiency**: Using RLE allows a multi-minute video to fit within the limited Flash and RAM of the RP2350.

### Display Interfacing
We use the `ssd1306` and `embedded-graphics` crates:
- **Interface**: I2C at 1MHz (High Speed).
- **Buffer**: The SSD1306 is driven in "Buffered Graphics Mode". The frame is first decoded into a local RAM buffer and then flushed to the display's internal memory over I2C in one go.

### Performance Optimizations
To achieve a smooth 20 FPS (or more) on a RISC-V core:
- **I2C Overclocking**: The I2C bus is set to 1000kHz (Standard is 100kHz, Fast is 400kHz). Most SSD1306 modules can handle 1MHz reliably on a short breadboard connection.
- **Inlining**: Critical paths like `put_bytes` and `decode_rle` are marked as `#[inline(always)]` to reduce function call overhead.
- **Release Profile in Dev**: We forced `opt-level = 3` in the development profile. Without optimizations, bit-level operations in Rust are too slow for real-time video playback on embedded systems.
- **Frame Rate Limiting**: We use the RP2350's hardware timer to ensure the video plays at a consistent 20 FPS, regardless of how fast the CPU can decode.

## Hardware Setup

| RP2350 Pin | SSD1306 Pin |
|------------|-------------|
| 3.3V       | VCC         |
| GND        | GND         |
| GPIO 4     | SDA         |
| GPIO 5     | SCL         |

*Note: GPIO 4 and 5 are the default I2C0 pins used in this project.*

## Running the Project

### Prerequisites
1. Install the RISC-V target: `rustup target add riscv32imac-unknown-none-elf`
2. Install `flip-link`: `cargo install flip-link`
3. Install `picotool` (for loading onto the board).

### Execution
Simply run the following command to build and upload:
```bash
cargo run
```

The project is configured to use `opt-level 3` even in debug mode, so `cargo run` will be fast enough for 20 FPS playback.

https://github.com/user-attachments/assets/fdab9752-14bc-4caf-baf9-2cd7debb56bf


