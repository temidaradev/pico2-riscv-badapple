# 05: Special Rust Patterns (Embedded)

Embedded Rust feels different from standard Rust. Here are the core patterns used in this project.

## 1. The "Singleton" Pattern (Move `pac`)
When you take peripherals in Rust, it is a **one-time** operation:
```rust
let mut pac = hal::pac::Peripherals::take().unwrap();
```
If you try to call this twice, it returns `None`. This prevents two different parts of your code from trying to control the same hardware simultaneously—a common source of bugs in C.

## 2. Ownership & Pins
In `main.rs`, we move pins into their peripheral functions:
```rust
let sda = pins.gpio4.into_function::<hal::gpio::FunctionI2c>();
```
The type system now knows that GPIO 4 is an I2C pin. You can no longer accidentally use it as a standard Digital Output. The compiler prevents hardware errors at compile time!

## 3. Static Lifetimes & `include_bytes!`
The video file is baked into the binary:
```rust
const BADAPPLE_RLE: &[u8] = include_bytes!("../badapple.rle");
```
This is a "Zero-Copy" operation. The data lives in the executable's Flash memory. We refer to it using a reference with a `'static` lifetime (implied for `const`), meaning it is available as long as the program is running.

## 4. Error Handling (`unwrap`)
In desktop apps, `unwrap()` is often discouraged. In embedded systems:
- If `display.init().unwrap()` fails, the hardware is likely broken or disconnected.
- There is no "user" to report an error to, so "crashing" (panicking) is often the safest and only logical response to hardware initialization failure.

## 5. Traits & Generic Abstractions
The `ssd1306` crate doesn't know about the RP2350 specifically. It works because the RP2350 HAL implements the `embedded-hal` I2C traits. This is the power of Rust's **Trait system**—true hardware portability.
