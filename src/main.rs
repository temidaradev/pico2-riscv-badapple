#![no_std]
#![no_main]
#![feature(riscv_ext_intrinsics)]

mod display;
mod rle;

use defmt::*;
use defmt_rtt as _;
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use rp235x_hal::fugit::RateExtU32;

use hal::entry;
use rp235x_hal as hal;

use crate::display::{init_display, render_frame};
use crate::rle::{RleDecoder, FRAME_SIZE};

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

const XTAL_FREQ_HZ: u32 = 12_000_000u32;
const BADAPPLE_RLE: &[u8] = include_bytes!("../badapple.rle");
const TARGET_FPS: u32 = 20;

#[entry]
fn main() -> ! {
    info!("Bad Apple on Pico2 RISC-V - Starting");
    
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

    let mut display = init_display(i2c);
    info!("Display initialized");

    let mut decoder = RleDecoder::new();
    let mut frame_buffer = [0u8; FRAME_SIZE];
    let frame_interval_us: u64 = 1_000_000 / TARGET_FPS as u64;

    info!("Starting video playback at {} FPS", TARGET_FPS);

    loop {
        let frame_start = timer.get_counter();

        if decoder.decode_frame(&mut frame_buffer, BADAPPLE_RLE) {
            render_frame(&mut display, &frame_buffer);

            let elapsed = (timer.get_counter() - frame_start).to_micros();
            if elapsed < frame_interval_us {
                timer.delay_us((frame_interval_us - elapsed) as u32);
            }
        }
    }
}
