# 01: Embedded Rust Setup & RISC-V on RP2350

This project uses the **RP2350**, the powerhouse behind the Raspberry Pi Pico 2. What makes it unique is its dual-core architecture that can switch between ARM Cortex-M33 and RISC-V Hazard3 cores.

## RISC-V on RP2350
While many Pico projects use ARM, we are targeting the **RISC-V Hazard3** cores. This is an open-standard Instruction Set Architecture (ISA).

### Key Configuration Files:
- **`.cargo/config.toml`**: This tells Cargo to use the RISC-V target `riscv32imac-unknown-none-elf` and links the correct runtime scripts.
- **`rp2350_riscv.x`**: The linker script that defines the memory layout (where Flash and RAM are located) specifically for the RISC-V cores.

## The `no_std` Environment
Embedded Rust typically runs without an OS (bare metal). We use `#![no_std]` to tell the compiler we don't have access to the standard library (`std`).

- **What we lose**: `Box`, `Vec`, `String`, and file/network I/O.
- **What we keep**: Everything else! Integers, structs, enums, and powerful abstractions like `embedded-hal`.

## Critical Crates used:
1. **`rp235x-hal`**: The Hardware Abstraction Layer. It provides safe Rust wrappers for the chip's pins, timers, and I2C controllers.
2. **`defmt`**: A highly efficient logging framework ("deferred formatting") that prints over a debug probe without slowing down the chip significantly.
3. **`embedded-hal`**: A set of common traits (like "I can read from I2C") that allows drivers like the SSD1306 one to work on any chip.
