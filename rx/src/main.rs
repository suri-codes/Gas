use ::log::info;
use esp_idf_svc::hal::units::Hertz;
use log::error;
use morse::{Bit, TIME_STEP_MICROS};

use crate::parser::{ListeningForMessage, Parser, Processing};

// max it allows is 80_000

// const SAMPLE_HERTZ: u64 = 1_000;
// const SAMPLE_HERTZ: u64 = 9_894;
// const SAMPLE_HERTZ: u64 = 19_575;
// const SAMPLE_HERTZ: u64 = 38_332;
// const SAMPLE_HERTZ: u64 = 47_567;
const SAMPLE_HERTZ: u64 = 62_428;
//

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

    let mut perfect_reads = 0;
    let mut successful_reads = 0;
    let mut attempts = 0;

    loop {
        let Ok(num_read) = adc.read(&mut samples, 10) else {
            // error!("Failed to perform adc read!");
            continue;
        };

        for measurement in &samples[0..num_read] {
            if let Some(ref mut parser) = message_parser {
                match parser.message() {
                    Ok(msg) => {
                        successful_reads += 1;
                        info!("Message: {msg}");
                        let perf_msg = morse::MSG.to_lowercase();
                        if msg == perf_msg {
                            perfect_reads += 1;
                        }

                        // lets crunch some numbers here
                        let read_rate: f32 = (successful_reads as f32 / attempts as f32) * 100.0;
                        let perfect_rate: f32 = (perfect_reads as f32 / attempts as f32) * 100.0;

                        info!("Read accuracy    : {read_rate}%");
                        info!("Perfect accuracy : {perfect_rate}%");
                        info!("Attempts         : {attempts}");
                        println!("\n\n")
                    }
                    Err(e) => {
                        // error!("failed to parse message! {e:?}");
                        // info!("light vals: {:#?}", parser.raw_val_buf);
                        // info!("bit_seq: {:#?}", parser.bit_seq);
                        // info!("morse_seq: {:#?}", parser.morse_seq);
                    }
                }
                // info!("light vals: {:#?}", parser.raw_val_buf);

                message_listener = None;
                message_parser = None;
                start_listener = Parser::default();
            } else if let Some(ref mut listener) = message_listener {
                match listener.process_light_val(measurement.data()) {
                    Some(Ok(parser)) => {
                        message_parser = Some(parser);
                        message_listener = None;
                    }
                    Some(Err(e)) => {
                        // error!("morse error! {e:?}");
                        // info!("bit_seq: {:#?}", listener.bit_seq);
                        // info!("morse_seq: {:#?}", listener.morse_seq);
                        // info!("light vals: {:#?}", listener.raw_val_buf);
                        start_listener = Parser::default();
                        message_parser = None;
                        message_listener = None;
                    }

                    None => continue,
                }
            } else if let Some(listener) = start_listener.process_light_val(measurement.data()) {
                // info!("start received");
                attempts += 1;
                message_listener = Some(listener);
                start_listener = Parser::default();
            }
        }
    }
}
