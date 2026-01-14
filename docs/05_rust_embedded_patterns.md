# 05: Special Rust Patterns (Embedded)

Embedded Rust feels different from standard Rust. It uses the type system to provide "Hardware Safety" that is impossible in most other languages.

## 1. The "Singleton" Pattern
When you take peripherals in Rust, it is a **one-time** operation:
```rust
let mut pac = hal::pac::Peripherals::take().unwrap();
```
Rust's ownership rules ensure that only **one instance** of the hardware peripherals exists. If a second thread or function tried to call `take()`, it would get `None`. This prevents "Race Conditions" where two parts of code try to change a pin's state at the same time.

## 2. The Typestate Pattern (Hardware Safety)
This is Rust's "Killer Feature" for embedded systems. Look at how we move pins:
```rust
let sda = pins.gpio4
    .into_pull_up_input()            // Transition 1: Pin -> Input
    .into_function::<FunctionI2c>(); // Transition 2: Input -> I2C
```
In most languages, `gpio4` is just an integer (`4`). You can send I2C data to a pin set as an Input and the code will crash or the chip will smoke. 

In Rust, the **Type** of the variable changes. Once you call `into_function::<FunctionI2c>()`, the variable `sda` is no longer a "Pin"—it is an "I2c0 Sda Pin". You literally **cannot** call the `digital_write()` function on it anymore because that function isn't defined for that type.

## 3. The "Layer Cake" (PAC vs HAL)
Our `Cargo.toml` shows two levels of abstraction:
1. **PAC (Peripheral Access Crate)**: The `rp235x-pac`. This is the "low level". It's a 1-to-1 mapping of the chip's memory registers. It's powerful but "unsafe" and tedious.
2. **HAL (Hardware Abstraction Layer)**: The `rp235x-hal`. This is the "high level". It uses the PAC internally but provides the "Safe" Rust API we use (like `timer.delay_ms()`).

## 4. Zero-Copy Data with `include_bytes!`
The video data is baked into our binary:
```rust
const BADAPPLE_RLE: &[u8] = include_bytes!("../badapple.rle");
```
This is a **Zero-Copy** operation. The bytes are stored in the chip's Flash. We aren't "loading" the file into RAM; we are creating a pointer (`&[u8]`) that points directly to the data in the Flash memory. This is why we can play a 1.6MB video even if the chip only has 512KB of RAM.

## 5. Advanced Macros: `#[entry]` and `#[unsafe]`
- **`#[entry]`**: This macro handles the "Boilerplate" of the chip's startup. It sets up the stack pointer, clears memory, and jumps to your `main` function.
- **Link Sections**: 
  ```rust
  #[unsafe(link_section = ".start_block")]
  pub static IMAGE_DEF: hal::block::ImageDef = ...
  ```
  This tells the Linker exactly where to put this data. The RP2350's bootloader expects a specific block of data at the very beginning of the Flash to know how to boot the chip.

## 6. Logging with `defmt`
Standard `println!` is too "heavy" for microcontrollers. It requires strings and formatting logic on the chip. 
**`defmt`** (Deferred Formatting) works differently: 
1. The **Compiler** keeps the strings on your computer.
2. The **Chip** only sends small ID numbers (integers) over the wire.
3. Your **Computer** sees the ID, looks up the string, and prints it.
This allows us to log information without significant performance hits.
