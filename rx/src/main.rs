use std::ops::BitAnd;

use ::log::info;
use esp_idf_svc::hal::units::Hertz;
use heapless::Vec;
use log::error;
use morse::{Bit, BitSequece, MorseBit, MorseBitSequence, MorseConversion, START_SEQUENCE};

const SAMPLE_HERTZ: u64 = 2000;

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
        if let Ok(num_read) = adc.read(&mut samples, 1) {
            // assert!(num_read == SAMPLE_STEP as usize);
            let sum: u64 = samples[0..num_read].iter().map(|e| e.data() as u64).sum();
            let avg = sum as f64 / num_read as f64;

            let bit = if avg <= 60.0 {
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
    message_buf: String,

    state: ParserState,
}

enum ParserState {
    WaitForStart,
    ListeningMessage,
}

impl Parser {
    fn new() -> Self {
        Parser {
            bit_buf: BitSequece::new(),
            morse_bit_buf: MorseBitSequence::new(),
            message_buf: String::new(),
            state: ParserState::WaitForStart,
        }
    }
    fn process_bit(&mut self, bit: morse::Bit) {
        use morse::Bit::*;
        use ParserState::*;
        self.bit_buf.push(bit).expect("should never overflow");
        match self.state {
            WaitForStart => {
                if self.bit_buf.is_full() {
                    if self.bit_buf.as_slice() == START_SEQUENCE {
                        info!("start sequence received");
                        self.bit_buf = BitSequece::new();
                        self.state = ListeningMessage;
                    } else {
                        self.bit_buf = BitSequece::new();
                    }
                }

                if bit == Bit::Lo {
                    self.bit_buf = BitSequece::new();
                }
            }

            ListeningMessage => {
                info!("{bit:#?}");
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
                            // self.bit_buf = BitSequece::new();

                            self.message_buf = String::new();
                            self.bit_buf = BitSequece::new();
                            self.morse_bit_buf = MorseBitSequence::new();
                            self.state = ParserState::WaitForStart;
                        }
                    }
                }
            }
        }

        // info!("bit_buf: {:?}", self.bit_buf);
        // we process bit buf on lo
    }

    fn process_m_bit(&mut self, m_bit: morse::MorseBit) {
        if m_bit == MorseBit::CharBreak {
            match char::from_morse_bit_sequence(&self.morse_bit_buf) {
                Ok(char) => {
                    if char != '\0' {
                        self.message_buf.push(char);
                        // info!("{char}");
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

        if m_bit == MorseBit::LineBreak {
            info!("{:?}", self.message_buf);
            self.message_buf = String::new();
            self.bit_buf = BitSequece::new();
            self.morse_bit_buf = MorseBitSequence::new();
            self.state = ParserState::WaitForStart;
        }
    }
}
