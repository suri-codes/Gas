// struct Parser<ParserState> {
//     bit_buf: BitSequece,
//     morse_bit_buf: MorseBitSequence,
//     message_buf: String,
// }

use std::{collections::VecDeque, marker::PhantomData};

use log::info;
use morse::{
    Bit, BitSequece, MorseBit, MorseBitSequence, MorseConversion, MorseError, START_SEQUENCE,
};

pub struct WaitingForStart;

pub struct ListeningForMessage;

pub struct Processing;

pub struct Parser<State = WaitingForStart> {
    state: core::marker::PhantomData<State>,
    start_queue: VecDeque<Bit>,
    bit_seq: BitSequece,
    morse_seq: MorseBitSequence,
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
            start_queue: VecDeque::with_capacity(6),
            bit_seq: BitSequece::new(),
            morse_seq: MorseBitSequence::new(),
        }
    }

    pub fn process_start_bit(&mut self, bit: Bit) -> Option<Parser<ListeningForMessage>> {
        // info!("{bit:?}");
        // we want to process start as a sliding window
        // here we do sliding window
        if self.start_queue.len() == START_SEQUENCE.len() {
            self.start_queue.pop_front();
            self.start_queue.push_back(bit);
            let pattern = self.start_queue.make_contiguous();

            if pattern == START_SEQUENCE {
                return Some(Parser {
                    state: PhantomData,
                    start_queue: VecDeque::new(),
                    bit_seq: BitSequece::new(),
                    morse_seq: MorseBitSequence::new(),
                });
            }
        } else {
            self.start_queue.push_back(bit);
        }

        None
    }
}

impl Parser<ListeningForMessage> {
    pub fn process_data_bit(&mut self, bit: Bit) -> Option<Result<Parser<Processing>, MorseError>> {
        // info!("{bit:#?}");
        self.bit_seq.push(bit).expect("should never be full");
        // info!("bit_seq: {:?}", self.bit_seq);
        if bit == morse::Bit::Lo {
            match TryInto::<MorseBit>::try_into(self.bit_seq.clone()) {
                Ok(m_bit) => {
                    self.bit_seq = BitSequece::new();

                    // info!("{m_bit:?}");

                    self.morse_seq
                        .push(m_bit)
                        .expect("should never run out of capacity");

                    if m_bit == MorseBit::LineBreak {
                        return Some(Ok(Parser {
                            state: PhantomData,
                            start_queue: VecDeque::new(),
                            bit_seq: BitSequece::new(),
                            morse_seq: self.morse_seq.clone(),
                        }));
                    }
                }
                Err(e) => {
                    return Some(Err(e));
                }
            }
        }

        if self.bit_seq.is_full() {
            return Some(Err(MorseError::FullBuffer));
        }

        None
    }
}

impl Parser<Processing> {
    pub fn message(&mut self) -> Result<String, MorseError> {
        let mut msg = String::new();

        for bit_slice in self.morse_seq.split(|e| *e == MorseBit::CharBreak) {
            // info!("slice: {bit_slice:?}");
            let c = char::from_morse_slice(bit_slice)?;
            msg.push(c);
        }

        Ok(msg)
    }
}
