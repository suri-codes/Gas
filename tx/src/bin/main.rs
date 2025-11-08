#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use morse::{BitSequece, MorseConversion, START_SEQUENCE, TIME_STEP_MICROS};
use {esp_backtrace as _, esp_println as _};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

// this is fasters with a shit ton of error: 637 char/s
// const MSG: &'static str = "eeeeeeeeeeeeeeeeeeeeeeeee\n";

// this is fastest with 0 error: 496 char/s
// const MSG: &'static str = "ee\n";

// const MSG: &'static str = "\n";

// const MSG: &'static str = "aaa\n";
// const MSG: &'static str = "eis\n";
const MSG: &'static str = "suri\n";

// ~ 300 char/s
// const MSG: &'static str = "Surendra\n";

// const MSG: &'static str = "Hello ESP32\n";

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut led = Output::new(
        peripherals.GPIO5,
        esp_hal::gpio::Level::Low,
        OutputConfig::default(),
    );

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    loop {
        // send start sequence
        info!("sending start sequence!");
        let bit_start = Instant::now();
        let mut bits = 0;
        for bit in START_SEQUENCE {
            bits += 1;
            hold_bit_for_time_step(&mut led, bit);
        }

        let char_start = Instant::now();
        let mut char_bits = 0;
        for char in MSG.chars().into_iter() {
            let m_seq = char
                .to_morse_bit_sequence()
                .expect("should be a valid bit sequence");
            for m_bit in m_seq {
                let b_seq: BitSequece = m_bit.into();
                for bit in b_seq {
                    bits += 1;
                    char_bits += 1;
                    hold_bit_for_time_step(&mut led, bit);
                }
            }
            // char break
            bits += 1;
            char_bits += 1;
            hold_bit_for_time_step(&mut led, morse::Bit::Lo);
        }
        let elapsed_char_micros = char_start.elapsed().as_micros();
        let elapsed_bit_micros = bit_start.elapsed().as_micros();
        let char_per_sec = (MSG.len() as f64 * 1.0e6) / elapsed_char_micros as f64;
        let bits_per_sec = (bits as f64 * 1.0e6) / elapsed_bit_micros as f64;
        info!("Message: {}", MSG);
        info!("transmission time: {:#?}", elapsed_char_micros);
        let expected_time = char_bits as usize * TIME_STEP_MICROS as usize;
        // + START_SEQUENCE.len() * TIME_STEP_MICROS as usize;
        info!("expected transmission time: {:#?}", expected_time);
        info!("chars per second : {:#?}", char_per_sec);
        info!("char bits : {:#?}", char_bits);
        info!("bits per second  : {:#?}", bits_per_sec);
        info!("total bits : {:#?}", bits);
        // spin_wait(Duration::from_secs(2));
    }
}

fn hold_bit_for_time_step(led: &mut Output<'_>, bit: morse::Bit) {
    match bit {
        morse::Bit::Hi => {
            // info!("HI");
            led.set_high();
        }
        morse::Bit::Lo => {
            // info!("LO");
            led.set_low();
        }
    }
    spin_wait(Duration::from_micros(morse::TIME_STEP_MICROS));
}

#[inline(always)]
fn spin_wait(duration: Duration) {
    let start = Instant::now();
    while start.elapsed() < duration {
        core::hint::spin_loop(); // Tells CPU this is a spin loop
    }
}
