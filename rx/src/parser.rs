use std::{collections::VecDeque, marker::PhantomData};

use morse::{
    Bit, BitSequece, MorseBit, MorseBitSequence, MorseConversion, MorseError, START_SEQUENCE,
};

use crate::HIGH_THRESHOLD;

pub struct WaitingForStart;

pub struct ListeningForMessage;

pub struct Processing;

pub type RawValBuf = heapless::Vec<u16, 200>;

pub struct Parser<State = WaitingForStart> {
    state: core::marker::PhantomData<State>,
    start_queue: Option<VecDeque<Bit>>,
    pub bit_seq: BitSequece,
    pub raw_val_buf: RawValBuf,
    pub morse_seq: MorseBitSequence,
}

impl Default for Parser<WaitingForStart> {
    fn default() -> Self {
        Parser::new()
    }
}

impl Parser<WaitingForStart> {
    pub fn new() -> Self {
        Self {
            state: PhantomData,
            start_queue: Some(VecDeque::with_capacity(12)),
            bit_seq: BitSequece::new(),
            raw_val_buf: RawValBuf::new(),
            morse_seq: MorseBitSequence::new(),
        }
    }

    pub fn process_light_val(&mut self, raw_val: u16) -> Option<Parser<ListeningForMessage>> {
        let bit = if raw_val < HIGH_THRESHOLD {
            Bit::Lo
        } else {
            Bit::Hi
        };
        let start_queue = self.start_queue.as_mut().unwrap();
        if start_queue.len() == START_SEQUENCE.len() {
            start_queue.pop_front();
            start_queue.push_back(bit);
            let pattern = start_queue.make_contiguous();

            if pattern == START_SEQUENCE {
                return Some(Parser {
                    state: PhantomData,
                    start_queue: None,
                    raw_val_buf: RawValBuf::new(),
                    bit_seq: BitSequece::new(),
                    morse_seq: MorseBitSequence::new(),
                });
            }
        } else {
            start_queue.push_back(bit);
        }

        None
    }
}

impl Parser<ListeningForMessage> {
    pub fn process_light_val(
        &mut self,
        raw_val: u16,
    ) -> Option<Result<Parser<Processing>, MorseError>> {
        if self.raw_val_buf.push(raw_val).is_err() {
            return Some(Err(MorseError::FullBuffer));
        }
        let bit = if raw_val < HIGH_THRESHOLD {
            Bit::Lo
        } else {
            Bit::Hi
        };

        if self.bit_seq.push(bit).is_err() {
            return Some(Err(MorseError::FullBuffer));
        }

        if bit == morse::Bit::Lo {
            match TryInto::<MorseBit>::try_into(self.bit_seq.clone()) {
                Ok(m_bit) => {
                    self.bit_seq.clear();

                    if self.morse_seq.push(m_bit).is_err() {
                        return Some(Err(MorseError::FullBuffer));
                    }

                    if m_bit == MorseBit::LineBreak {
                        return Some(Ok(Parser {
                            state: PhantomData,
                            raw_val_buf: self.raw_val_buf.clone(),
                            start_queue: None,
                            bit_seq: self.bit_seq.clone(),
                            morse_seq: self.morse_seq.clone(),
                        }));
                    }
                }
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }

        None
    }
}

impl Parser<Processing> {
    pub fn message(&mut self) -> Result<String, MorseError> {
        let mut msg = String::new();

        for bit_slice in self.morse_seq.split(|e| *e == MorseBit::CharBreak) {
            let c = char::from_morse_slice(bit_slice)?;
            msg.push(c);
        }

        let msg = msg
            .to_lowercase()
            .strip_suffix('\n')
            .ok_or(MorseError::UnknownMorseSequence)?
            .to_owned();

        Ok(msg)
    }
}
