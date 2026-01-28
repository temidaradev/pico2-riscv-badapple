# Porting Audio Logic to Pico 2 (RISC-V)

This guide helps you port your C++ audio logic (WAV parsing, Ring Buffer, Volume Pot) to the Pico 2 using Rust, with a specific focus on the **Deneyap Hoparlör**.

## 🔊 Hardware: Deneyap Hoparlör

The Deneyap Hoparlör is a compact audio module perfect for the Pico 2.

### Technical Specifications
- **Amplifier**: Diodes PAM8302A (Mono Class D)
- **Speaker**: PUI Audio AS01508MS-SC11-WP-R
- **Frequency Range**: 600Hz - 20kHz
- **Sensitivity**: 89dB @ 3dB
- **Operating Voltage**: 3.3V
- **Connectors**: 2x I2C (JST SH 4-pin 1mm) - **Note**: These are for **power and daisy-chaining only**, not for I2C data transfer.
- **Data Input**: DAC or PWM pins.

### Wiring for Pico 2
| Deneyap Pin | Pico 2 Pin | Function |
| :--- | :--- | :--- |
| **VCC** | 3V3 (Pin 36) | Power |
| **GND** | GND (Pin 38) | Ground |
| **IN+** | GPIO 15 (Pin 20) | PWM Audio Signal |
| **IN-** | GND | Reference Ground |

---

## 🛠 Hardware Mapping

| Feature | C++ (ESP32) | Pico 2 (Rust) |
| :--- | :--- | :--- |
| **Speaker PWM** | GPIO 15 (`ledc`) | GPIO 15 (`PWM Slice 7`) |
| **Volume Pot** | GPIO 4 (`analogRead`) | GPIO 26 (`ADC channel 0`) |
| **Timer** | `hw_timer_t` | `hal::Timer` or `Alarm` |
| **Network** | WiFi (ESP32) | WiFi (`cyw43` + `embassy-net` on Pico 2 W) |

---

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

In your `main.rs`, initialize the PWM for the speaker and ADC for the volume. For clear audio, we set the PWM frequency to ~44.1kHz (or matching your sample rate).

```rust
// GPIO 15 for Speaker (PWM)
let _speaker_pin = pins.gpio15.into_function::<hal::gpio::FunctionPwm>();
let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
let mut speaker_pwm = pwm_slices.pwm7; // GPIO 15 is on PWM7

// Configure PWM frequency: 150MHz / (44100 * 256) ≈ 13.27
// We use a wrap of 255 for 8-bit audio samples
speaker_pwm.set_top(255);
speaker_pwm.set_div_int(13);
speaker_pwm.set_div_frac(4);
speaker_pwm.enable();

// GPIO 26 for Potentiometer (ADC0)
let mut adc = hal::Adc::new(pac.ADC, &mut pac.RESETS);
let mut pot_pin = hal::adc::AdcPin::new(pins.gpio26.into_floating_input()).unwrap();

// Reading & Scaling Volume
let raw_vol: u16 = adc.read(&mut pot_pin).unwrap(); // 0-4095
let volume = (raw_vol >> 4) as u8; // Scale to 0-255
```

---

## 🌐 WiFi Streaming (Pico 2 W)

For WiFi streaming on the Pico 2 W, we use the `cyw43` driver and `embassy-net` stack. This requires an async executor.

### 1. Network Setup
Initialize the `cyw43` chip and start the network stack.

```rust
let p = embassy_rp::init(Default::default());
let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

let pwr = Output::new(p.PIN_23, Level::Low);
let cs = Output::new(p.PIN_25, Level::High);
let mut pio = Pio::new(p.PIO0, Irqs);
let spi = Cyw43Spi::new(&mut pio.common, pio.sm0, pio.irq0, cs, p.PIN_24, p.PIN_29, p.DMA_CH0);

let state = make_static!(cyw43::State::new());
let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
unwrap!(spawner.spawn(cyw43_task(runner)));

control.init(clm).await;
control.set_power_management(cyw43::PowerManagementMode::PowerSave).await;

let config = embassy_net::Config::dhcpv4(Default::default());
let stack = &*make_static!(Stack::new(net_device, config, make_static!(StackResources::<2>::new()), seed));
unwrap!(spawner.spawn(net_task(stack)));
```

### 2. TCP Audio Stream
Open a socket and read chunks into the ring buffer.

```rust
let mut rx_buffer = [0u8; 4096];
let mut tx_buffer = [0u8; 4096];
let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

socket.connect((Ipv4Address::new(192, 168, 1, 100), 8080)).await.unwrap();

let mut buf = [0u8; 1024];
loop {
    let n = socket.read(&mut buf).await.unwrap();
    for &sample in &buf[..n] {
        // Push to ring buffer (SPSC Queue)
        while let Err(_) = producer.enqueue(sample) {
            Timer::after_millis(1).await; // Wait if full
        }
    }
}
```

> [!IMPORTANT]
> Audio streaming requires stable jitter management. Use a large ring buffer (e.g., `heapless::spsc::Queue<u8, 16384>`) to handle network latency spikes.

---

## ⏱ Multicore Synchronization

The Pico 2 has two RISC-V cores. You can run the audio task on Core 1 to ensure it isn't interrupted by heavy video decoding on Core 0.

### 1. Spawning Core 1
Use the `SIO` block to spawn a function on the second core.

```rust
let mut mc = hal::multicore::Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut pac.SIO);
let cores = mc.cores();
let core1 = &mut cores[1];

core1.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
    // Core 1 Audio Loop
    loop {
        if let Some(sample) = consumer.dequeue() {
            // Apply volume and write to PWM
            let vol_sample = (sample as u32 * volume_global as u32 / 255) as u16;
            speaker_pwm.set_chan_level(hal::pwm::Channel::B, vol_sample);
        }
    }
}).unwrap();
```

### 2. Inter-Core FIFO (SIO)
Use the hardware FIFO to pass synchronization signals (e.g., Play/Pause) between cores.

```rust
// Core 0: Send command
sio.fifo.write_blocking(CMD_PLAY);

// Core 1: Read command
if let Some(cmd) = sio.fifo.read() {
    match cmd {
        CMD_PLAY => playing = true,
        _ => {}
    }
}
```

## 🚀 Quick Implementation Steps

1.  **Convert Audio**: Use FFmpeg to prepare your stream: `ffmpeg -i input.mp3 -ar 44100 -ac 1 -f u8 output.raw`.
2.  **Initialize Hardware**: Setup PWM7 (GPIO 15) and ADC0 (GPIO 26) in `main.rs`.
3.  **Setup Spawner**: Use `embassy-executor` to manage the WiFi and Network tasks.
4.  **Bridge Cores**: Use a `StaticCell` to share the `spsc::Queue` between Core 0 (Network) and Core 1 (Audio Out).
5.  **Sync Video**: Use the PWM interrupt or a high-priority timer to keep audio ahead of the 20FPS video frames.
