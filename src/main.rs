#![no_std]
#![no_main]
#![feature(riscv_ext_intrinsics)]

use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{image::ImageRaw, pixelcolor::BinaryColor, prelude::*};
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use rp235x_hal::fugit::RateExtU32;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

use hal::entry;

use rp235x_hal as hal;

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

const BADAPPLE_RLE: &[u8] = include_bytes!("../badapple.rle");
const FRAME_SIZE: usize = 1024;

#[inline(always)]
fn put_bytes(frame_buffer: &mut [u8; FRAME_SIZE], byte: u8, count: i32, bytes_written: &mut usize) {
    let count_usize = count as usize;
    let end = (*bytes_written + count_usize).min(FRAME_SIZE);
    frame_buffer[*bytes_written..end].fill(byte);
    *bytes_written = end;
}

#[inline(always)]
fn decode_rle(
    frame_buffer: &mut [u8; FRAME_SIZE],
    c: u8,
    runlength: &mut i32,
    c_to_dup: &mut i32,
    bytes_written: &mut usize,
) {
    if *c_to_dup == -1 {
        if c == 0x55 || c == 0xaa {
            *c_to_dup = c as i32;
        } else {
            put_bytes(frame_buffer, c, 1, bytes_written);
        }
    } else {
        if *runlength == -1 {
            if c == 0 {
                put_bytes(frame_buffer, (*c_to_dup & 0xff) as u8, 1, bytes_written);
                *c_to_dup = -1;
            } else if (c & 0x80) == 0 {
                let val = if *c_to_dup == 0x55 { 0 } else { 255 };
                put_bytes(frame_buffer, val, c as i32, bytes_written);
                *c_to_dup = -1;
            } else {
                *runlength = (c & 0x7f) as i32;
            }
        } else {
            *runlength |= (c as i32) << 7;
            let val = if *c_to_dup == 0x55 { 0 } else { 255 };
            put_bytes(frame_buffer, val, *runlength, bytes_written);
            *c_to_dup = -1;
            *runlength = -1;
        }
    }
}

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = hal::pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let sda = pins
        .gpio4
        .into_pull_up_input()
        .into_function::<hal::gpio::FunctionI2c>();
    let scl = pins
        .gpio5
        .into_pull_up_input()
        .into_function::<hal::gpio::FunctionI2c>();

    let i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda,
        scl,
        1000.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    let mut frame_buffer = [0u8; FRAME_SIZE];
    let mut runlength: i32 = -1;
    let mut c_to_dup: i32 = -1;
    let mut bytes_written: usize = 0;

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();
    display.flush().unwrap();

    let mut src_pos = 0;
    let frame_interval_us = 1_000_000 / 20;

    loop {
        let frame_start = timer.get_counter();

        if src_pos >= BADAPPLE_RLE.len() {
            src_pos = 0;
            runlength = -1;
            c_to_dup = -1;
            bytes_written = 0;
        }

        while bytes_written < FRAME_SIZE && src_pos < BADAPPLE_RLE.len() {
            decode_rle(
                &mut frame_buffer,
                BADAPPLE_RLE[src_pos],
                &mut runlength,
                &mut c_to_dup,
                &mut bytes_written,
            );
            src_pos += 1;
        }

        if bytes_written == FRAME_SIZE {
            let image = ImageRaw::<BinaryColor>::new(&frame_buffer, 128);
            let _ = image.draw(&mut display);
            display.flush().unwrap();

            bytes_written = 0;

            let elapsed = (timer.get_counter() - frame_start).to_micros();
            if elapsed < frame_interval_us {
                timer.delay_us((frame_interval_us - elapsed) as u32);
            }
        }
    }
}
