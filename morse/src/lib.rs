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

pub type MorseBitSequence = heapless::Vec<MorseBit, 350>;

#[derive(Debug)]
pub enum MorseError {
    UnknownBitSequence,
    UnknownMorseSequence,
    UnsupportedChar(char),
    FullBuffer,
}

// pub const TIME_STEP_MICROS: u64 = 1e3 as u64 / 2;
// pub const TIME_STEP_MICROS: u64 = 1e3 as u64 / 2;
// pub const TIME_STEP_MICROS: u64 = 1000;
// pub const TIME_STEP_MICROS: u64 = 900;
// pub const TIME_STEP_MICROS: u64 = 100;
// pub const TIME_STEP_MICROS: u64 = 50;
// pub const TIME_STEP_MICROS: u64 = 20;
// pub const TIME_STEP_MICROS: u64 = 15;
pub const TIME_STEP_MICROS: u64 = 11;
// pub const TIME_STEP_MICROS: u64 = 10;

// pub const MSG: &str = "Surendra";
// pub const MSG: &str = "suri.codes";

// pub const MSG: &str = "e";
// pub const MSG: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeee";
// pub const MSG: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

// pub const MSG: &str = "EEEEEEEEEEEEETTTTTTTTTTAAAAAAAAAOOOOOOOOOIIIIIIIINNNNNNNSSSSSSHHHHHHRRRRRRDDDDDDLLLLLCCCCUUUUMMMMWWWFFFGGGYYYPPPBBVVKKJJXQZ";
// pub const MSG: &str = "Hello ESP32";
pub const MSG: &str = "UCSC CSE 121 ABCDEFGHIJKLM NOPQRSTUVWXYZ 12345 67890";

pub const START_SEQUENCE: [Bit; 10] = [
    Bit::Hi,
    Bit::Hi,
    Bit::Hi,
    Bit::Hi,
    Bit::Hi,
    Bit::Lo,
    Bit::Lo,
    Bit::Lo,
    Bit::Hi,
    Bit::Hi,
];

// gets picked up really easily, but corrupts data
// pub const START_SEQUENCE: [Bit; 2] = [Bit::Hi, Bit::Hi];

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
            MorseBit::Dot => vec.extend_from_slice(&[Lo]), // 2, 3
            // MorseBit::Dot => vec.extend_from_slice(&[Hi, Lo]),
            // MorseBit::Dash => vec.extend_from_slice(&[Hi, Lo]), // 2
            MorseBit::Dash => vec.extend_from_slice(&[Hi, Hi, Lo]), // 3
            // MorseBit::Dash => vec.extend_from_slice(&[Hi, Hi, Lo]),
            // MorseBit::CharBreak => vec.extend_from_slice(&[Hi, Hi, Lo]), // 2
            MorseBit::CharBreak => vec.extend_from_slice(&[Hi, Lo]), // 3
            MorseBit::LineBreak => vec.extend_from_slice(&[Hi, Hi, Hi, Lo]),
            MorseBit::WordBreak => vec.extend_from_slice(&[Hi, Hi, Hi, Hi, Lo]),
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
            return Ok(Dot);
        }

        if value.starts_with(&[Hi, Lo]) && value.len() == 2 {
            return Ok(CharBreak);
            // return Ok(Dash);
        }
        if value.starts_with(&[Hi, Hi, Lo]) && value.len() == 3 {
            return Ok(Dash);
            // return Ok(CharBreak);
        }
        if value.starts_with(&[Hi, Hi, Hi, Lo]) && value.len() == 4 {
            // return Ok(WordBreak);
            return Ok(LineBreak);
        }
        if value.starts_with(&[Hi, Hi, Hi, Hi, Lo]) && value.len() == 5 {
            return Ok(WordBreak);
            // return Ok(LineBreak);
        }

        Err(MorseError::UnknownBitSequence)
    }
}

pub const MORSE_TABLE: [&[MorseBit]; 128] = {
    let mut table = [&[] as &[MorseBit]; 128];
    use MorseBit::*;

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
    table[b'0' as usize] = &[Dash, Dash, Dash, Dash, Dash];
    table[b'1' as usize] = &[Dot, Dash, Dash, Dash, Dash];
    table[b'2' as usize] = &[Dot, Dot, Dash, Dash, Dash];
    table[b'3' as usize] = &[Dot, Dot, Dot, Dash, Dash];
    table[b'4' as usize] = &[Dot, Dot, Dot, Dot, Dash];
    table[b'5' as usize] = &[Dot, Dot, Dot, Dot, Dot];
    table[b'6' as usize] = &[Dash, Dot, Dot, Dot, Dot];
    table[b'7' as usize] = &[Dash, Dash, Dot, Dot, Dot];
    table[b'8' as usize] = &[Dash, Dash, Dash, Dot, Dot];
    table[b'9' as usize] = &[Dash, Dash, Dash, Dash, Dot];

    table[b'.' as usize] = &[Dot, Dash, Dot, Dash, Dot, Dash];

    table
};

use MorseBit::*;
pub const INVERSE_MORSE_TABLE: &[(&[MorseBit], char)] = &[
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
    (&[Dot, Dash, Dot, Dash, Dot, Dash], '.'),
    (&[CharBreak], '\0'),
    (&[LineBreak], '\n'),
    (&[WordBreak], ' '),
];
