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
use morse::{BitSequece, MorseConversion, START_SEQUENCE};
use {esp_backtrace as _, esp_println as _};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// const MSG: &'static str = "WHAT the actual fuck dude We are cooking\n";
const MSG: &'static str = "na\n";
// const MSG: &'static str = "Surendra\n";

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
        for bit in START_SEQUENCE {
            hold_bit_for_time_step(&mut led, bit);
        }

        let start = Instant::now();
        for char in MSG.chars().into_iter() {
            let m_seq = char
                .to_morse_bit_sequence()
                .expect("should be a valid bit sequence");
            for m_bit in m_seq {
                let b_seq: BitSequece = m_bit.into();
                for bit in b_seq {
                    hold_bit_for_time_step(&mut led, bit);
                }
            }
            // char break
            hold_bit_for_time_step(&mut led, morse::Bit::Lo);
        }
        // here we can wait for a bit
        info!("BREAK");

        info!("Message: {}", MSG);
        let elapsed_micros = start.elapsed().as_micros();
        info!("transmission time: {:#?}", start.elapsed());
        let char_per_sec = (MSG.len() as f64 * 1.0e6) / elapsed_micros as f64;
        info!("chars per second: {:#?}", char_per_sec);
        spin_wait(Duration::from_secs(2));
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
