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
        // info!("bit_seq: {:?}", self.bit_seq);
        self.bit_seq.push(bit).expect("should never be full");
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
            info!("slice: {bit_slice:?}");
            let c = char::from_morse_slice(bit_slice)?;
            msg.push(c);
        }

        Ok(msg)
    }
}

// struct Parser {
//     bit_buf: BitSequece,
//     morse_bit_buf: MorseBitSequence,
//     message_buf: String,

//     state: ParserState,
// }

// enum ParserState {
//     WaitForStart,
//     ListeningMessage,
// }

// impl Parser {
//     fn new() -> Self {
//         Parser {
//             bit_buf: BitSequece::new(),
//             morse_bit_buf: MorseBitSequence::new(),
//             message_buf: String::new(),
//             state: ParserState::WaitForStart,
//         }
//     }
//     fn process_bit(&mut self, bit: morse::Bit) {
//         use morse::Bit::*;
//         use ParserState::*;
//         self.bit_buf.push(bit).expect("should never overflow");
//         match self.state {
//             WaitForStart => {
//                 if self.bit_buf.is_full() {
//                     if self.bit_buf.as_slice() == START_SEQUENCE {
//                         info!("start sequence received");
//                         self.bit_buf = BitSequece::new();
//                         self.state = ListeningMessage;
//                     } else {
//                         self.bit_buf = BitSequece::new();
//                     }
//                 }

//                 if bit == Bit::Lo {
//                     self.bit_buf = BitSequece::new();
//                 }
//             }

//             ListeningMessage => {
//                 // info!("{bit:#?}");
//                 if bit == morse::Bit::Lo {
//                     match TryInto::<MorseBit>::try_into(self.bit_buf.clone()) {
//                         Ok(m_bit) => {
//                             self.bit_buf = BitSequece::new();
//                             // info!("{m_bit:?}");
//                             self.process_m_bit(m_bit);
//                         }
//                         Err(e) => {
//                             error!("failed to parse bit pattern: {e:?}");
//                             error!("pattern: {:?}", self.bit_buf);
//                             // self.bit_buf = BitSequece::new();

//                             self.message_buf = String::new();
//                             self.bit_buf = BitSequece::new();
//                             self.morse_bit_buf = MorseBitSequence::new();
//                             self.state = ParserState::WaitForStart;
//                         }
//                     }
//                 }

//                 if self.bit_buf.is_full() {
//                     info!("Unable to parse that shit, restting");
//                     self.message_buf = String::new();
//                     self.bit_buf = BitSequece::new();
//                     self.morse_bit_buf = MorseBitSequence::new();
//                     self.state = ParserState::WaitForStart;
//                 }
//             }
//         }

//         // info!("bit_buf: {:?}", self.bit_buf);
//         // we process bit buf on lo
//     }

//     fn process_m_bit(&mut self, m_bit: morse::MorseBit) {
//         if m_bit == MorseBit::CharBreak {
//             match char::from_morse_bit_sequence(&self.morse_bit_buf) {
//                 Ok(char) => {
//                     if char != '\0' {
//                         self.message_buf.push(char);
//                         // info!("{char}");
//                     }
//                 }
//                 Err(_) => {
//                     // error!("{e:?}")
//                 }
//             }
//             self.morse_bit_buf = MorseBitSequence::new();
//         } else {
//             self.morse_bit_buf
//                 .push(m_bit)
//                 .expect("should never overflow");
//         }

//         if m_bit == MorseBit::LineBreak {
//             info!("{:?}", self.message_buf);
//             self.message_buf = String::new();
//             self.bit_buf = BitSequece::new();
//             self.morse_bit_buf = MorseBitSequence::new();
//             self.state = ParserState::WaitForStart;
//         }
//     }
// }
