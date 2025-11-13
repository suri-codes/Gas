use ::log::info;
use esp_idf_svc::{hal::units::Hertz, timer::EspTimerService};
use log::error;

use crate::parser::{ListeningForMessage, Parser, Processing};

// const SAMPLE_HERTZ: u64 = 83255;
const SAMPLE_HERTZ: u64 = 83322;
// const SAMPLE_HERTZ: u64 = 83262;
// const SAMPLE_HERTZ: u64 = 1110;

const SAMPLE_STEP: u64 = 100;

const HIGH_THRESHOLD: u16 = 210;

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

    let mut samples = [AdcMeasurement::default(); SAMPLE_STEP as usize];

    let time_service = EspTimerService::new().unwrap();

    let mut start = time_service.now();
    let mut start_listener = Parser::new();
    let mut message_listener: Option<Parser<ListeningForMessage>> = None;
    let mut message_parser: Option<Parser<Processing>> = None;
    let mut perfect_reads = 0;
    let mut successful_reads = 0;
    let mut attempts = 0;
    let mut fastest_perfect = 0.0;

    loop {
        let Ok(num_read) = adc.read(&mut samples, 10) else {
            // its ok if we cant read from the adc
            continue;
        };

        for measurement in &samples[0..num_read] {
            if let Some(ref mut parser) = message_parser {
                match parser.message() {
                    Ok(msg) => {
                        let end = time_service.now();
                        successful_reads += 1;
                        let perf_msg = morse::MSG.to_lowercase();
                        // Convert nanoseconds to seconds (1 second = 1,000,000,000 nanoseconds)
                        let read_rate: f32 = (successful_reads as f32 / attempts as f32) * 100.0;
                        let chars = msg.len();
                        let duration = (end - start).as_nanos();
                        let chars_per_second = (chars as f64) / (duration as f64 / 1_000_000_000.0);
                        if msg == perf_msg {
                            perfect_reads += 1;
                            if chars_per_second > fastest_perfect {
                                fastest_perfect = chars_per_second;
                            }
                        }
                        let perfect_rate: f32 = (perfect_reads as f32 / attempts as f32) * 100.0;
                        // lets crunch some numbers here

                        info!("Message          : {msg}");
                        info!("Read accuracy    : {read_rate}%");
                        info!("Perfect accuracy : {perfect_rate}%");
                        info!("Attempts         : {attempts}");
                        info!("Time Taken       : {duration:?}");
                        info!("current c/s      : {:.2}", chars_per_second);
                        info!("fastest c/s      : {:.2}", fastest_perfect);
                        println!("\n\n")
                    }
                    Err(e) => {
                        error!("failed to parse message! {e:?}");
                        // info!("morse_seq: {:#?}", parser.morse_seq);
                        // info!("bit_seq: {:#?}", parser.bit_seq);
                        // info!("light vals: {:#?}", parser.raw_val_buf);
                    }
                }
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
                        error!("Failed during measurement! {e:?}");
                        // info!("morse_seq: {:#?}", listener.morse_seq);
                        // info!("bit_seq: {:#?}", listener.bit_seq);
                        // info!("light vals: {:#?}", listener.raw_val_buf);
                        start_listener = Parser::default();
                        message_parser = None;
                        message_listener = None;
                    }

                    None => continue,
                }
            } else if let Some(listener) = start_listener.process_light_val(measurement.data()) {
                message_listener = Some(listener);

                start_listener = Parser::default();
                start = time_service.now();
                attempts += 1;
            }
        }
    }
}
