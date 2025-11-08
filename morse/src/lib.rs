#![no_std]

#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub enum Bit {
    Hi,
    Lo,
}

#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub enum MorseBit {
    Dot,
    Dash,
    CharBreak,
    WordBreak,
    LineBreak,
}

pub type BitSequece = heapless::Vec<Bit, 8>;

pub type MorseBitSequence = heapless::Vec<MorseBit, 100>;

#[derive(Debug)]
pub enum MorseError {
    UnknownBitSequence,
    UnknownMorseSequence,
    UnsupportedChar(char),
    FullBuffer,
}

/// time step defined in period length
// pub const TIME_STEP_MICROS: u64 = 25;
pub const TIME_STEP_MICROS: u64 = 1e5 as u64;

// pub const START_SEQUENCE: [Bit; 9] = [
//     Bit::Hi,
//     Bit::Hi,
//     Bit::Lo,
//     Bit::Lo,
//     Bit::Hi,
//     Bit::Hi,
//     Bit::Hi,
//     Bit::Lo,
//     Bit::Lo,
// ];
pub const START_SEQUENCE: [Bit; 10] = [
    Bit::Hi,
    Bit::Hi,
    Bit::Hi,
    Bit::Hi,
    Bit::Hi,
    Bit::Hi,
    Bit::Hi,
    Bit::Lo,
    Bit::Lo,
    Bit::Lo,
];

pub trait MorseConversion {
    fn to_morse_bit_sequence(&self) -> Result<MorseBitSequence, MorseError>;
    fn from_morse_slice(sequence: &[MorseBit]) -> Result<Self, MorseError>
    where
        Self: Sized;
}

impl MorseConversion for char {
    fn to_morse_bit_sequence(&self) -> Result<MorseBitSequence, MorseError> {
        use MorseBit::*;
        let mut vec = MorseBitSequence::new();
        let slice = {
            let table_result = MORSE_TABLE[*self as usize];
            if table_result.is_empty() {
                match self {
                    ' ' => &[WordBreak],
                    '\n' => &[LineBreak],
                    _ => return Err(MorseError::UnsupportedChar(*self)),
                }
            } else {
                table_result
            }
        };
        vec.extend_from_slice(slice)
            .expect("should never run out of capacity");
        Ok(vec)
    }

    fn from_morse_slice(sequence: &[MorseBit]) -> Result<Self, MorseError> {
        let (_, c) = INVERSE_MORSE_TABLE
            .iter()
            .find(|(e, _)| *e == sequence)
            .ok_or(MorseError::UnknownMorseSequence)?;
        Ok(*c)
    }
}

impl From<MorseBit> for BitSequece {
    fn from(value: MorseBit) -> Self {
        let mut vec = BitSequece::new();
        use Bit::*;
        match value {
            MorseBit::CharBreak => vec.extend_from_slice(&[Lo]),
            MorseBit::Dot => vec.extend_from_slice(&[Hi, Lo]),
            MorseBit::Dash => vec.extend_from_slice(&[Hi, Hi, Lo]),
            MorseBit::WordBreak => vec.extend_from_slice(&[Hi, Hi, Hi, Lo]),
            MorseBit::LineBreak => vec.extend_from_slice(&[Hi, Hi, Hi, Hi, Lo]),
        }
        .expect("should be impossible to error");

        vec
    }
}

impl TryFrom<BitSequece> for MorseBit {
    type Error = MorseError;
    fn try_from(value: BitSequece) -> Result<Self, Self::Error> {
        use Bit::*;
        use MorseBit::*;

        if value.starts_with(&[Lo]) && value.len() == 1 {
            return Ok(CharBreak);
        }

        if value.starts_with(&[Hi, Lo]) && value.len() == 2 {
            return Ok(Dot);
        }
        if value.starts_with(&[Hi, Hi, Lo]) && value.len() == 3 {
            return Ok(Dash);
        }
        if value.starts_with(&[Hi, Hi, Hi, Lo]) && value.len() == 4 {
            return Ok(WordBreak);
        }
        if value.starts_with(&[Hi, Hi, Hi, Hi, Lo]) && value.len() == 5 {
            return Ok(LineBreak);
        }

        Err(MorseError::UnknownBitSequence)
    }
}

pub const MORSE_TABLE: [&[MorseBit]; 128] = {
    let mut table = [&[] as &[MorseBit]; 128];
    use MorseBit::*;

    // A-Z
    table[b'A' as usize] = &[Dot, Dash]; // .-
    table[b'B' as usize] = &[Dash, Dot, Dot, Dot]; // -...
    table[b'C' as usize] = &[Dash, Dot, Dash, Dot]; // -.-.
    table[b'D' as usize] = &[Dash, Dot, Dot]; // -..
    table[b'E' as usize] = &[Dot]; // .
    table[b'F' as usize] = &[Dot, Dot, Dash, Dot]; // ..-.
    table[b'G' as usize] = &[Dash, Dash, Dot]; // --.
    table[b'H' as usize] = &[Dot, Dot, Dot, Dot]; // ....
    table[b'I' as usize] = &[Dot, Dot]; // ..
    table[b'J' as usize] = &[Dot, Dash, Dash, Dash]; // .---
    table[b'K' as usize] = &[Dash, Dot, Dash]; // -.-
    table[b'L' as usize] = &[Dot, Dash, Dot, Dot]; // .-..
    table[b'M' as usize] = &[Dash, Dash]; // --
    table[b'N' as usize] = &[Dash, Dot]; // -.
    table[b'O' as usize] = &[Dash, Dash, Dash]; // ---
    table[b'P' as usize] = &[Dot, Dash, Dash, Dot]; // .--.
    table[b'Q' as usize] = &[Dash, Dash, Dot, Dash]; // --.-
    table[b'R' as usize] = &[Dot, Dash, Dot]; // .-.
    table[b'S' as usize] = &[Dot, Dot, Dot]; // ...
    table[b'T' as usize] = &[Dash]; // -
    table[b'U' as usize] = &[Dot, Dot, Dash]; // ..-
    table[b'V' as usize] = &[Dot, Dot, Dot, Dash]; // ...-
    table[b'W' as usize] = &[Dot, Dash, Dash]; // .--
    table[b'X' as usize] = &[Dash, Dot, Dot, Dash]; // -..-
    table[b'Y' as usize] = &[Dash, Dot, Dash, Dash]; // -.--
    table[b'Z' as usize] = &[Dash, Dash, Dot, Dot]; // --..

    // a-z (lowercase)
    table[b'a' as usize] = &[Dot, Dash];
    table[b'b' as usize] = &[Dash, Dot, Dot, Dot];
    table[b'c' as usize] = &[Dash, Dot, Dash, Dot];
    table[b'd' as usize] = &[Dash, Dot, Dot];
    table[b'e' as usize] = &[Dot];
    table[b'f' as usize] = &[Dot, Dot, Dash, Dot];
    table[b'g' as usize] = &[Dash, Dash, Dot];
    table[b'h' as usize] = &[Dot, Dot, Dot, Dot];
    table[b'i' as usize] = &[Dot, Dot];
    table[b'j' as usize] = &[Dot, Dash, Dash, Dash];
    table[b'k' as usize] = &[Dash, Dot, Dash];
    table[b'l' as usize] = &[Dot, Dash, Dot, Dot];
    table[b'm' as usize] = &[Dash, Dash];
    table[b'n' as usize] = &[Dash, Dot];
    table[b'o' as usize] = &[Dash, Dash, Dash];
    table[b'p' as usize] = &[Dot, Dash, Dash, Dot];
    table[b'q' as usize] = &[Dash, Dash, Dot, Dash];
    table[b'r' as usize] = &[Dot, Dash, Dot];
    table[b's' as usize] = &[Dot, Dot, Dot];
    table[b't' as usize] = &[Dash];
    table[b'u' as usize] = &[Dot, Dot, Dash];
    table[b'v' as usize] = &[Dot, Dot, Dot, Dash];
    table[b'w' as usize] = &[Dot, Dash, Dash];
    table[b'x' as usize] = &[Dash, Dot, Dot, Dash];
    table[b'y' as usize] = &[Dash, Dot, Dash, Dash];
    table[b'z' as usize] = &[Dash, Dash, Dot, Dot];

    // 0-9
    table[b'0' as usize] = &[Dash, Dash, Dash, Dash, Dash]; // -----
    table[b'1' as usize] = &[Dot, Dash, Dash, Dash, Dash]; // .----
    table[b'2' as usize] = &[Dot, Dot, Dash, Dash, Dash]; // ..---
    table[b'3' as usize] = &[Dot, Dot, Dot, Dash, Dash]; // ...--
    table[b'4' as usize] = &[Dot, Dot, Dot, Dot, Dash]; // ....-
    table[b'5' as usize] = &[Dot, Dot, Dot, Dot, Dot]; // .....
    table[b'6' as usize] = &[Dash, Dot, Dot, Dot, Dot]; // -....
    table[b'7' as usize] = &[Dash, Dash, Dot, Dot, Dot]; // --...
    table[b'8' as usize] = &[Dash, Dash, Dash, Dot, Dot]; // ---..
    table[b'9' as usize] = &[Dash, Dash, Dash, Dash, Dot]; // ----.

    table
};

use MorseBit::*;
pub const INVERSE_MORSE_TABLE: &[(&[MorseBit], char)] = &[
    (&[CharBreak], '\0'),
    (&[LineBreak], '\n'),
    (&[WordBreak], ' '),
    (&[Dot, Dash], 'A'),
    (&[Dash, Dot, Dot, Dot], 'B'),
    (&[Dash, Dot, Dash, Dot], 'C'),
    (&[Dash, Dot, Dot], 'D'),
    (&[Dot], 'E'),
    (&[Dot, Dot, Dash, Dot], 'F'),
    (&[Dash, Dash, Dot], 'G'),
    (&[Dot, Dot, Dot, Dot], 'H'),
    (&[Dot, Dot], 'I'),
    (&[Dot, Dash, Dash, Dash], 'J'),
    (&[Dash, Dot, Dash], 'K'),
    (&[Dot, Dash, Dot, Dot], 'L'),
    (&[Dash, Dash], 'M'),
    (&[Dash, Dot], 'N'),
    (&[Dash, Dash, Dash], 'O'),
    (&[Dot, Dash, Dash, Dot], 'P'),
    (&[Dash, Dash, Dot, Dash], 'Q'),
    (&[Dot, Dash, Dot], 'R'),
    (&[Dot, Dot, Dot], 'S'),
    (&[Dash], 'T'),
    (&[Dot, Dot, Dash], 'U'),
    (&[Dot, Dot, Dot, Dash], 'V'),
    (&[Dot, Dash, Dash], 'W'),
    (&[Dash, Dot, Dot, Dash], 'X'),
    (&[Dash, Dot, Dash, Dash], 'Y'),
    (&[Dash, Dash, Dot, Dot], 'Z'),
    (&[Dash, Dash, Dash, Dash, Dash], '0'),
    (&[Dot, Dash, Dash, Dash, Dash], '1'),
    (&[Dot, Dot, Dash, Dash, Dash], '2'),
    (&[Dot, Dot, Dot, Dash, Dash], '3'),
    (&[Dot, Dot, Dot, Dot, Dash], '4'),
    (&[Dot, Dot, Dot, Dot, Dot], '5'),
    (&[Dash, Dot, Dot, Dot, Dot], '6'),
    (&[Dash, Dash, Dot, Dot, Dot], '7'),
    (&[Dash, Dash, Dash, Dot, Dot], '8'),
    (&[Dash, Dash, Dash, Dash, Dot], '9'),
];
