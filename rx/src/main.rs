use ::log::info;
use esp_idf_svc::hal::units::Hertz;
use log::error;
use morse::{Bit, TIME_STEP_MICROS};

use crate::parser::{ListeningForMessage, Parser, Processing};

// max it allows is 80_000
// const SAMPLE_HERTZ: u64 = 50_000;
// const SAMPLE_HERTZ: u64 = 10_000;
// const SAMPLE_HERTZ: u64 = 1_000;
// const SAMPLE_HERTZ: u64 = 1980;
// const SAMPLE_HERTZ: u64 = 5015;
const SAMPLE_HERTZ: u64 = 38_288;
// const SAMPLE_HERTZ: u64 = 20_000;
// const SAMPLE_HERTZ: u64 = 10_000;

// const SAMPLE_STEP: u64 = morse::TIME_STEP_MICROS / SAMPLE_PERIOD;
const SAMPLE_STEP: u64 = 100;

mod parser;
fn main() -> anyhow::Result<()> {
    use esp_idf_svc::hal::adc::{AdcContConfig, AdcContDriver, AdcMeasurement, Attenuated};
    use esp_idf_svc::hal::peripherals::Peripherals;

    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let config = AdcContConfig::default().sample_freq(Hertz::from(SAMPLE_HERTZ as u32));

    let adc_1_channel_0 = Attenuated::db11(peripherals.pins.gpio2);
    let mut adc = AdcContDriver::new(peripherals.adc1, &config, adc_1_channel_0)?;

    adc.start()?;

    info!("SAMPLE_STEP: {}", SAMPLE_STEP);

    //Default to just read 100 measurements per each read
    let mut samples = [AdcMeasurement::default(); SAMPLE_STEP as usize];

    let mut start_listener = Parser::new();
    let mut message_listener: Option<Parser<ListeningForMessage>> = None;
    let mut message_parser: Option<Parser<Processing>> = None;

    loop {
        let Ok(num_read) = adc.read(&mut samples, 10) else {
            // error!("Failed to perform adc read!");
            continue;
        };

        for measurement in &samples[0..num_read] {
            let bit = if measurement.data() <= 40 {
                Bit::Lo
            } else {
                Bit::Hi
            };

            if let Some(ref mut parser) = message_parser {
                match parser.message() {
                    Ok(msg) => {
                        info!("{msg}");
                    }
                    Err(e) => {
                        // error!("failed to parse message! {e:?}");
                    }
                }

                message_listener = None;
                message_parser = None;
                start_listener = Parser::default();
            } else if let Some(ref mut listener) = message_listener {
                match listener.process_data_bit(bit) {
                    Some(Ok(parser)) => {
                        message_parser = Some(parser);
                        message_listener = None;
                    }
                    Some(Err(e)) => {
                        // error!("morse error! {e:?}");
                        start_listener = Parser::default();
                        message_parser = None;
                        message_listener = None;
                    }

                    None => continue,
                }
            } else if let Some(listener) = start_listener.process_start_bit(bit) {
                // info!("start received");
                message_listener = Some(listener);
                start_listener = Parser::default();
            }
        }
    }
}
