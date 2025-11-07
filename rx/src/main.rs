use std::ops::BitAnd;

use ::log::info;
use esp_idf_svc::hal::units::Hertz;
use log::error;
use morse::{BitSequece, MorseBit, MorseBitSequence, MorseConversion};

const SAMPLE_HERTZ: u64 = 1000;

const SAMPLE_PERIOD: u64 = 1_000_000 / SAMPLE_HERTZ;

const SAMPLE_STEP: u64 = morse::TIME_STEP_MICROS / SAMPLE_PERIOD;

fn main() -> anyhow::Result<()> {
    use esp_idf_svc::hal::adc::{AdcContConfig, AdcContDriver, AdcMeasurement, Attenuated};
    use esp_idf_svc::hal::peripherals::Peripherals;

    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    // let config = AdcContConfig::default().sample_freq(Hertz::from(80000));
    let config = AdcContConfig::default().sample_freq(Hertz::from(SAMPLE_HERTZ as u32));

    // need to calculate sample step

    let adc_1_channel_0 = Attenuated::db11(peripherals.pins.gpio2);
    let mut adc = AdcContDriver::new(peripherals.adc1, &config, adc_1_channel_0)?;

    adc.start()?;

    info!("SAMPLE_STEP: {}", SAMPLE_STEP);

    //Default to just read 100 measurements per each read
    let mut samples = [AdcMeasurement::default(); SAMPLE_STEP as usize];

    let mut parser = Parser::new();

    loop {
        if let Ok(num_read) = adc.read(&mut samples, 10) {
            // info!("Read {} measurement.", num_read);
            let len = num_read;
            let sum: u64 = samples.map(|e| e.data() as u64).into_iter().sum();
            let avg = sum as f64 / len as f64;

            let bit = if avg <= 50.0 {
                morse::Bit::Lo
            } else {
                morse::Bit::Hi
            };
            // info!("{bit:#?}");
            parser.process_bit(bit);
            // for index in 0..num_read {
            //     info!("{}", samples[index].data());
            // }
        }
    }
}

struct Parser {
    bit_buf: BitSequece,
    morse_bit_buf: MorseBitSequence,
}

impl Parser {
    fn new() -> Self {
        Parser {
            bit_buf: BitSequece::new(),
            morse_bit_buf: MorseBitSequence::new(),
        }
    }
    fn process_bit(&mut self, bit: morse::Bit) {
        self.bit_buf.push(bit).expect("should never overflow");

        // info!("bit_buf: {:?}", self.bit_buf);
        // we process bit buf on lo
        if bit == morse::Bit::Lo {
            match TryInto::<MorseBit>::try_into(self.bit_buf.clone()) {
                Ok(m_bit) => {
                    self.bit_buf = BitSequece::new();
                    // info!("{m_bit:?}");
                    self.process_m_bit(m_bit);
                }
                Err(e) => {
                    error!("failed to parse bit pattern: {e:?}");
                    error!("pattern: {:?}", self.bit_buf);
                    self.bit_buf = BitSequece::new();
                }
            }
        }
    }

    fn process_m_bit(&mut self, m_bit: morse::MorseBit) {
        if m_bit == MorseBit::CharBreak {
            match char::from_morse_bit_sequence(&self.morse_bit_buf) {
                Ok(char) => {
                    if char != '\0' {
                        info!("{char}");
                    }
                }
                Err(_) => {
                    // error!("{e:?}")
                }
            }
            self.morse_bit_buf = MorseBitSequence::new();
        } else {
            self.morse_bit_buf
                .push(m_bit)
                .expect("should never overflow");
        }
    }
}
