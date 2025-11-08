#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::iter::once;

use defmt::{error, info};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use morse::{
    Bit, BitSequece, MSG, MorseBit, MorseConversion, MorseError, START_SEQUENCE, TIME_STEP_MICROS,
};
use {esp_backtrace as _, esp_println as _};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

type DataPacket = heapless::Vec<Bit, 100>;

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

    // lets form the data packet
    let data_packet = form_data_packet()
        .inspect_err(|_| {
            error!("error forming data packet!");
        })
        .unwrap();

    let delay = Delay::new();

    let mut running_avg_recv_freq: f64 = 0.0;
    let mut transmits = 0;
    loop {
        // send start sequence
        info!("sending start sequence!");
        let bit_start = Instant::now();
        for bit in &START_SEQUENCE {
            hold_bit_for_time_step(&mut led, bit, &delay);
        }

        let char_start = Instant::now();

        for bit in &data_packet {
            hold_bit_for_time_step(&mut led, bit, &delay);
        }

        let elapsed_char_micros = char_start.elapsed().as_micros();
        let elapsed_bit_micros = bit_start.elapsed().as_micros();

        let bits = data_packet.len() + START_SEQUENCE.len();
        let char_bits = data_packet.len();

        let char_per_sec = (MSG.len() as f64 * 1.0e6) / elapsed_char_micros as f64;
        let bits_per_sec = (bits as f64 * 1.0e6) / elapsed_bit_micros as f64;
        let expected_time = char_bits as usize * TIME_STEP_MICROS as usize;

        // should calculate what the ideal receiver freq should be
        let optimal_receiver_freq = 1e6
            / (((elapsed_char_micros - expected_time as u64) as f64 / char_bits as f64)
                + TIME_STEP_MICROS as f64);

        transmits += 1;
        running_avg_recv_freq = running_avg_recv_freq
            + ((optimal_receiver_freq - running_avg_recv_freq) / transmits as f64);

        info!("Message           :  {}", MSG);
        info!("transmission time :  {:#?} micros", elapsed_char_micros);
        info!("optimal recv freq :  {} Hz", running_avg_recv_freq);
        info!("chars per second  :  {:#?}", char_per_sec);
        info!("bits per second   :  {:#?}", bits_per_sec);
        info!("total msg bits    :  {:#?}", char_bits);
        info!("total bits        :  {:#?}", bits);
        info!("\n\n");

        // print_data_packet(&data_packet);

        // print the light vals we sent
        // delay.delay(Duration::from_secs(1));
        delay.delay(Duration::from_millis(500));
    }
}

fn print_data_packet(data_packet: &DataPacket) {
    for bit in data_packet {
        match bit {
            Bit::Hi => info!("Hi"),
            Bit::Lo => info!("Lo"),
        }
    }
}

fn form_data_packet() -> Result<DataPacket, MorseError> {
    let mut packet = DataPacket::new();

    for char in MSG.to_lowercase().chars().into_iter() {
        let m_seq = char
            .to_morse_bit_sequence()
            .expect("should be a valid bit sequence");
        for m_bit in m_seq {
            let b_seq: BitSequece = m_bit.into();
            for bit in b_seq {
                packet.push(bit).map_err(|_| MorseError::FullBuffer)?;
            }
        }

        let x: BitSequece = MorseBit::CharBreak.into();
        for bit in x {
            packet.push(bit).map_err(|_| MorseError::FullBuffer)?;
        }
    }

    // send line break
    let x: BitSequece = MorseBit::LineBreak.into();
    for bit in x {
        packet.push(bit).map_err(|_| MorseError::FullBuffer)?;
    }
    Ok(packet)
}

#[inline(always)]
fn hold_bit_for_time_step(led: &mut Output<'_>, bit: &morse::Bit, delay: &Delay) {
    match bit {
        morse::Bit::Hi => led.set_high(),
        morse::Bit::Lo => led.set_low(),
    }
    delay.delay_micros(morse::TIME_STEP_MICROS as u32);
}
