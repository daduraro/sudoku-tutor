
use crate::flags::DigitFlags;
use crate::index::DigitIndex;
use crate::error::SudokuError;

#[derive(PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord, Hash)]
pub struct SudokuCell(DigitFlags);

impl core::default::Default for SudokuCell {
    fn default() -> Self {
        SudokuCell(DigitFlags::ALL)
    }
}

impl SudokuCell {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn digit(d: DigitIndex) -> Self {
        SudokuCell(DigitFlags::only(d))
    }

    pub fn is_valid(&self) -> bool {
        self.0.any()
    }

    pub fn is_digit(&self) -> bool {
        self.0.count() == 1
    }

    pub fn digit_value(&self) -> Option<DigitIndex> {
        self.is_digit().then(|| self.0.first().unwrap())
    }

    pub fn is_bivalue(&self) -> bool {
        self.0.count() == 2
    }

    pub fn num_digits(&self) -> usize {
        self.0.count()
    }

    pub fn apply_mask(&mut self, mask: &DigitFlags) -> bool {
        if self.would_change(mask) {
            self.0 &= *mask;
            true
        } else {
            false
        }
    }

    pub fn would_change(&self, mask: &DigitFlags) -> bool {
        (self.0 & *mask) != self.0
    }

    pub fn contains(&self, d: DigitIndex) -> bool {
        self.0[d]
    }

    pub fn digit_flags(&self) -> DigitFlags {
        self.0
    }

    pub fn digits(&self) -> impl Iterator<Item=DigitIndex> {
        self.0.iter()
    }
}

impl core::convert::TryFrom<char> for SudokuCell {
    type Error = SudokuError;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        if c == '.' { return Ok(Self::default()) }

        if let Some(v) = c.to_digit(10) && v < 10 {
            if v == 0 { Ok(Self::default()) } 
            else { Ok(Self::digit(DigitIndex::new((v - 1) as usize)?)) }
        } else {
            Err(SudokuError::InvalidDigit(c))
        }
    }
}

impl core::convert::From<&SudokuCell> for char {
    fn from(value: &SudokuCell) -> Self {
        if let Some(d) = value.digit_value() {
            char::from_digit((d.value() + 1) as u32, 10).unwrap()
        } else {
            '0'
        }
    }
}

impl core::ops::BitAndAssign<DigitFlags> for &mut SudokuCell {
    fn bitand_assign(&mut self, rhs: DigitFlags) {
        self.0 &= rhs
    }
}