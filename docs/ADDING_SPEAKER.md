# Porting Audio Logic to Pico 2 (RISC-V)

This guide helps you port your C++ audio logic (WAV parsing, Ring Buffer, Volume Pot) to the Pico 2 using Rust.

## 🛠 Hardware Mapping

| Feature | C++ (ESP32) | Pico 2 (Rust) |
| :--- | :--- | :--- |
| **Speaker PWM** | GPIO 15 (`ledc`) | GPIO 15 (`PWM Slice 7`) |
| **Volume Pot** | GPIO 4 (`analogRead`) | GPIO 26 (`ADC channel 0`) |
| **Timer** | `hw_timer_t` | `hal::Timer` or `Alarm` |
| **File System** | `fs::FS` | `BADAPPLE_AUDIO` (embedded bytes) |

## 🏗 Rust Implementation Guide

### 1. Data Structures

Instead of `volatile` globals, we use a struct to manage the audio state. For the ring buffer in `no_std` Rust, `heapless::spsc::Queue` is the safest way to handle producer/consumer logic.

```rust
// src/audio.rs sample structure
pub struct AudioSystem {
    pub volume: u8,
    pub sample_rate: u32,
    pub channels: u16,
}

// WAV Header structure (first 44 bytes)
#[repr(C)]
pub struct WavHeader {
    pub riff: [u8; 4],
    pub file_size: u32,
    pub wave: [u8; 4],
    pub fmt: [u8; 4],
    pub fmt_len: u32,
    pub format_type: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    pub data: [u8; 4],
    pub data_len: u32,
}
```

### 2. PWM & ADC Setup

In your `main.rs`, initialize the PWM for the speaker and ADC for the volume:

```rust
// GPIO 15 for Speaker (PWM)
let _speaker_pin = pins.gpio15.into_function::<hal::gpio::FunctionPwm>();
let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
let mut speaker_pwm = pwm_slices.pwm7; // GPIO 15 is on PWM7

// GPIO 26 for Potentiometer (ADC0)
let mut adc = hal::Adc::new(pac.ADC, &mut pac.RESETS);
let mut pot_pin = hal::adc::AdcPin::new(pins.gpio26.into_floating_input()).unwrap();
```

### 3. Porting the logic

#### WAV Loading
In Rust, we can cast the first 44 bytes of your `include_bytes!` array to the `WavHeader` struct to check the `RIFF` marker and extract `sample_rate`.

#### Volume Map
Instead of `map(val, 0, 4095, 0, 255)`, use:
```rust
let val: u16 = adc.read(&mut pot_pin).unwrap();
let volume = (val >> 4) as u8; // Convert 12-bit ADC to 8-bit volume
```

#### Audio Processing
The bit-shifting logic from your C++ code:
```rust
// 16-bit to 8-bit PWM value
let sample: i16 = ...; 
let pwm_value = ((sample as i32 + 32768) >> 8) as u8;
// Apply volume
let scaled = (pwm_value as u16 * volume as u16) >> 8;
```

## ⏱ Synchronization

Since the Pico 2 RISC-V doesn't use `vTaskDelay` (FreeRTOS), you have two choices for the "Feeder" logic:

1.  **Direct Loop**: Run the audio feeder logic inside your main `loop {}` alongside the video decoding.
2.  **Multicore**: Since the Pico 2 has two cores, you can run the `audioTask` on Core 1 while Video stays on Core 0, mimicking your `xTaskCreatePinnedToCore` setup.

## 🚀 Quick Implementation Steps

1.  **Prepare Audio**: Convert your file to 22050Hz, Mono, 16-bit WAV.
2.  **Add ADC**: Enable the ADC in `main.rs` to read the volume.
3.  **Buffer**: Use a small buffer (e.g., 2048 samples) to keep the audio flowing between frames.
4.  **PWM**: Set PWM frequency to ~40kHz (twice your sample rate or higher) for clear audio.
