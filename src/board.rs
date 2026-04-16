use bitvec::{bitarr, array::BitArray, order::Lsb0};

use crate::error::SudokuError;
use crate::index::{CellIndex, HouseIndex, HouseIndexer, DigitIndex};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SudokuBoard(Vec<SudokuCell>);

impl SudokuBoard {
    pub fn new(data: Vec<SudokuCell>) -> Result<Self, SudokuError> {
        if data.len() == 9*9 {
            Ok(SudokuBoard(data))
        } else {
            Err(SudokuError::InvalidBoardSize(data.len()))
        }
    }

    pub fn house<Idx: HouseIndexer>(&self, idx: Idx) -> impl Iterator<Item=&SudokuCell> {
        idx.indices().into_iter()
            .map(|cell_idx: CellIndex| &self[cell_idx])
    }
    pub fn house_mut<Idx: HouseIndexer>(&mut self, idx: Idx) -> impl Iterator<Item=&mut SudokuCell> {
        let ptr = self.0.as_mut_ptr();
        idx.indices().into_iter()
            .map(move |cell_idx| unsafe { &mut *ptr.add(cell_idx.flat()) })
    }
    pub fn indexed_house<Idx: HouseIndexer>(&self, idx: Idx) -> impl Iterator<Item=(CellIndex, &SudokuCell)> {
        idx.indices().into_iter()
            .map(move |cell_idx| (cell_idx, &self[cell_idx]))
    }
    pub fn indexed_house_mut<Idx: HouseIndexer>(&mut self, idx: Idx) -> impl Iterator<Item=(CellIndex, &mut SudokuCell)> {
        let ptr = self.0.as_mut_ptr();
        idx.indices().into_iter()
            .map(move |cell_idx| (cell_idx, unsafe { &mut *ptr.add(cell_idx.flat()) }))
    }

    pub fn iter(&self) -> impl Iterator<Item = &SudokuCell> { self.0.iter() }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SudokuCell> { self.0.iter_mut() }
    pub fn indexed_iter(&self) -> impl Iterator<Item = (CellIndex, &SudokuCell)> {
        self.0.iter().enumerate().map(|(i, cell)| (CellIndex::from(i), cell))
    }
    pub fn indexed_iter_mut(&mut self) -> impl Iterator<Item = (CellIndex, &mut SudokuCell)> {
        self.0.iter_mut().enumerate().map(|(i, cell)| (CellIndex::from(i), cell))
    }

    pub fn is_valid(&self) -> bool {
        self.iter().all(SudokuCell::is_valid)
    }
    pub fn is_solved(&self) -> bool {
        HouseIndex::domain().iter().all(|h|{
            self.house(*h).fold(DigitFlag::default(), |acc, c| {
                let mut acc = acc;
                if let Some(d) = c.digit_value() {
                    acc.set(*d, true)
                }
                acc
            }).count_ones() == 9
        })
    }

}

impl SudokuStringDecoding for SudokuBoard {
    fn decode_sudoku_string(data: &str) -> Result<Self, SudokuError> {
        SudokuBoard::new(Vec::<SudokuCell>::decode_sudoku_string(data)?)
    }
}

impl IntoIterator for SudokuBoard {
    type Item = SudokuCell;
    type IntoIter = <std::vec::Vec<SudokuCell> as std::iter::IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'board> IntoIterator for &'board SudokuBoard {
    type Item = &'board SudokuCell;
    type IntoIter = <&'board std::vec::Vec<SudokuCell> as std::iter::IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

impl<'board> IntoIterator for &'board mut SudokuBoard {
    type Item = &'board mut SudokuCell;
    type IntoIter = <&'board mut std::vec::Vec<SudokuCell> as std::iter::IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter { self.0.iter_mut() }
}

impl core::ops::Index<CellIndex> for SudokuBoard {
    type Output = SudokuCell;
    fn index(&self, index: CellIndex) -> &Self::Output {
        &self.0[index.flat()]
    }
}

impl core::ops::IndexMut<CellIndex> for SudokuBoard {
    fn index_mut(&mut self, index: CellIndex) -> &mut Self::Output {
        &mut self.0[index.flat()]
    }
}

impl core::ops::Index<&CellIndex> for SudokuBoard {
    type Output = SudokuCell;
    fn index(&self, index: &CellIndex) -> &Self::Output {
        &self.0[index.flat()]
    }
}
impl core::ops::IndexMut<&CellIndex> for SudokuBoard {
    fn index_mut(&mut self, index: &CellIndex) -> &mut Self::Output {
        &mut self.0[index.flat()]
    }
}

pub type DigitFlag = BitArray<[u16; 1]>;
pub const ALL_DIGITS_FLAG: DigitFlag = bitarr![const u16, Lsb0; 1, 1, 1, 1, 1, 1, 1, 1, 1];

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct SudokuCell(DigitFlag);

impl core::default::Default for SudokuCell {
    fn default() -> Self {
        SudokuCell(ALL_DIGITS_FLAG)
    }
}

impl SudokuCell {
    pub fn digit(d: DigitIndex) -> Self {
        let mut v = DigitFlag::ZERO;
        v.set(*d, true);
        SudokuCell(v)
    }

    pub fn is_valid(&self) -> bool {
        self.0.any()
    }

    pub fn is_digit(&self) -> bool {
        self.0.count_ones() == 1
    }

    pub fn digit_value(&self) -> Option<DigitIndex> {
        if self.is_digit() {
            self.0.first_one().map(DigitIndex::new)
        } else {
            None
        }
    }

    pub fn apply_mask(&mut self, mask: &DigitFlag) -> bool {
        if self.0 & *mask != self.0 {
            self.0 &= *mask;
            true
        } else {
            false
        }
    }
}

impl core::ops::Index<DigitIndex> for &SudokuCell {
    type Output = bool;
    fn index(&self, index: DigitIndex) -> &Self::Output {
        &self.0[*index]
    }
}

impl core::convert::TryFrom<char> for SudokuCell {
    type Error = SudokuError;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        if c == '.' { return Ok(Self::default()) }

        if let Some(v) = c.to_digit(10) && v < 10 {
            if v == 0 { Ok(Self::default()) } 
            else { Ok(Self::digit(DigitIndex::new((v - 1) as usize))) }
        } else {
            Err(SudokuError::InvalidDigit(c))
        }
    }
}

impl core::convert::From<&SudokuCell> for char {
    fn from(value: &SudokuCell) -> Self {
        if let Some(d) = value.digit_value() {
            char::from(d)
        } else {
            '0'
        }
    }
}

impl core::ops::BitAndAssign<DigitFlag> for &mut SudokuCell {
    fn bitand_assign(&mut self, rhs: DigitFlag) {
        self.0 &= rhs
    }
}

impl core::convert::From<DigitIndex> for char {
    fn from(value: DigitIndex) -> Self {
        char::from(&value)
    }
}

impl core::convert::From<&DigitIndex> for char {
    fn from(value: &DigitIndex) -> Self {
        char::from_digit((**value + 1) as u32, 10).unwrap()
    }
}

impl core::ops::Deref for SudokuCell {
    type Target = DigitFlag;

    fn deref(&self) -> &Self::Target {
        &self.0   
    }
}


#[allow(dead_code)]
pub trait SudokuStringEncoding {
    fn encode_sudoku_string(self) -> String;
}

pub trait SudokuStringDecoding where
    Self: std::marker::Sized
{
    fn decode_sudoku_string(game: &str) -> Result<Self, SudokuError>;
}

impl<'cell, T> SudokuStringEncoding for T
where
    T: IntoIterator<Item = &'cell SudokuCell>,
{
    fn encode_sudoku_string(self) -> String {
        self.into_iter()
            .map(char::from)
            .collect()
    }
}

impl<T> SudokuStringDecoding for T
where
    T: FromIterator<SudokuCell>
{
    fn decode_sudoku_string(game: &str) -> Result<Self, SudokuError> {
        game.chars()
            .map(SudokuCell::try_from)
            .collect::<Result<T, SudokuError>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{BlockIndex, ColumnIndex, RowIndex};

    #[test]
    fn test_encoding() -> Result<(), SudokuError> {
        let g = SudokuBoard::decode_sudoku_string("501740008000000050098600400040961580050000010016854070005006730070000000900072805")?;
        assert_eq!(g, SudokuBoard::decode_sudoku_string(&g.encode_sudoku_string())?);
        Ok(())
    }

    #[test]
    fn test_read() {
        // +-------+-------+-------+
        // | 5 0 1 | 7 4 0 | 0 0 8 |
        // | 0 0 0 | 0 0 0 | 0 5 0 |
        // | 0 9 8 | 6 0 0 | 4 0 0 |
        // +-------+-------+-------+
        // | 0 4 0 | 9 6 1 | 5 8 0 |
        // | 0 5 0 | 0 0 0 | 0 1 0 |
        // | 0 1 6 | 8 5 4 | 0 7 0 |
        // +-------+-------+-------+
        // | 0 0 5 | 0 0 6 | 7 3 0 |
        // | 0 7 0 | 0 0 0 | 0 0 0 |
        // | 9 0 0 | 0 7 2 | 8 0 5 |
        // +-------+-------+-------+
        let g = SudokuBoard::decode_sudoku_string("501740008000000050098600400040961580050000010016854070005006730070000000900072805").unwrap();
        assert_eq!(g.house(BlockIndex::new(5)).encode_sudoku_string(), "580010070");
        assert_eq!(g.house(ColumnIndex::new(8)).encode_sudoku_string(), "800000005");
        assert_eq!(g.house(RowIndex::new(3)).encode_sudoku_string(), "040961580");
    }

    #[test]
    fn modify_block() {
        let mut g = SudokuBoard::decode_sudoku_string("501740008000000050098600400040961580050000010016854070005006730070000000900072805").unwrap();
        let idx = ColumnIndex::new(0);
        for c in g.house_mut(idx) {
            *c = SudokuCell::digit(DigitIndex::new(0));
        }
        assert_eq!(g.house(idx).encode_sudoku_string(), "111111111");
    }

    #[test]
    #[should_panic]
    fn test_read_oob() {
        let _r = RowIndex::new(8);
        let _b = BlockIndex::new(9);
    }

}