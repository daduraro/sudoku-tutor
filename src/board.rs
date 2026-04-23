use std::iter::zip;

use bitvec::{bitarr, array::BitArray, order::Lsb0};
use itertools::Itertools;

use crate::error::SudokuError;
use crate::index::{CellIndex, DigitIndex, HouseIndex, HouseIndexer, SudokuSubCellIndex};

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SudokuBoard(Vec<SudokuCell>);

impl SudokuBoard {
    pub fn new(data: Vec<SudokuCell>) -> Result<Self, SudokuError> {
        if data.len() == CellIndex::COUNT {
            Ok(SudokuBoard(data))
        } else {
            Err(SudokuError::InvalidBoardSize(data.len()))
        }
    }

    pub fn house<Idx: HouseIndexer>(&self, idx: Idx) -> impl Iterator<Item=&SudokuCell> {
        idx.cell_indices().into_iter()
            .map(|cell_idx: CellIndex| &self[cell_idx])
    }
    pub fn house_mut<Idx: HouseIndexer>(&mut self, idx: Idx) -> impl Iterator<Item=&mut SudokuCell> {
        debug_assert!(idx.flat_indices().iter().all_unique());
        let ptr = self.0.as_mut_ptr();
        idx.flat_indices().into_iter()
            .map(move |offset| unsafe { &mut *ptr.add(offset) })
    }
    pub fn indexed_house<Idx: HouseIndexer>(&self, idx: Idx) -> impl Iterator<Item=(CellIndex, &SudokuCell)> {
        idx.cell_indices().into_iter()
            .map(move |cell_idx| (cell_idx, &self[cell_idx]))
    }
    pub fn indexed_house_mut<Idx: HouseIndexer>(&mut self, idx: Idx) -> impl Iterator<Item=(CellIndex, &mut SudokuCell)> {
        debug_assert!(idx.cell_indices().iter().map(CellIndex::flat).all_unique());
        let ptr = self.0.as_mut_ptr();
        idx.cell_indices().into_iter()
            .map(move |cell_idx| (cell_idx, unsafe { &mut *ptr.add(cell_idx.flat()) }))
    }

    pub fn iter(&self) -> impl Iterator<Item = &SudokuCell> { self.0.iter() }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SudokuCell> { self.0.iter_mut() }
    pub fn indexed_iter(&self) -> impl Iterator<Item = (CellIndex, &SudokuCell)> {
        self.0.iter().enumerate().map(|(i, cell)| (CellIndex::from_flat(i), cell))
    }
    pub fn indexed_iter_mut(&mut self) -> impl Iterator<Item = (CellIndex, &mut SudokuCell)> {
        self.0.iter_mut().enumerate().map(|(i, cell)| (CellIndex::from_flat(i), cell))
    }

    pub fn is_valid(&self) -> bool {
        self.iter().all(SudokuCell::is_valid)
    }
    pub fn is_solved(&self) -> bool {
        HouseIndex::domain().iter().all(|h|{
            self.house(*h).fold(SudokuFlags::default(), |mut acc, c| {
                if let Some(d) = c.digit_value() {
                    acc.set(d.value(), true)
                }
                acc
            }).count_ones() == 9
        })
    }

    pub fn diff(&self, prev: &SudokuBoard) -> Vec<SudokuSubCellIndex> {
        zip(self.indexed_iter(), prev).flat_map(|((cell_idx, curr), prev)| {
            DigitIndex::domain().iter().cloned().filter_map(move |d| {
                let has_diff = curr.contains(d) ^ prev.contains(d);
                has_diff.then_some((cell_idx, d))
            })
        }).collect()
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

pub type SudokuFlags = BitArray<[u16; 1]>;
pub const SUDOKU_FLAG_ALL: SudokuFlags = bitarr![const u16, Lsb0; 1, 1, 1, 1, 1, 1, 1, 1, 1];

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct DigitMask(SudokuFlags);

impl DigitMask {
    pub fn new(flags: SudokuFlags) -> Self {
        DigitMask(flags & SUDOKU_FLAG_ALL)
    }

    pub fn all_but(digit: DigitIndex) -> Self {
        let mut flags = SUDOKU_FLAG_ALL;
        flags.set(digit.value(), false);
        DigitMask(flags)
    }

    pub fn only(digit: DigitIndex) -> Self {
        let mut flags = SudokuFlags::default();
        flags.set(digit.value(), true);
        DigitMask(flags)
    }

    pub fn add(mut self, digit: DigitIndex) -> Self {
        self.0.set(digit.value(), true);
        self
    }

    pub fn sub(mut self, digit: DigitIndex) -> Self {
        self.0.set(digit.value(), false);
        self
    }

    pub fn value(&self) -> SudokuFlags {
        self.0
    }
}


impl core::default::Default for DigitMask {
    fn default() -> Self {
        DigitMask(SUDOKU_FLAG_ALL)
    }
}

pub trait DigitMaskFromIter {
    fn all_but(self) -> DigitMask;
    fn only(self) -> DigitMask;
}

impl<It> DigitMaskFromIter for It
where 
    It: Iterator<Item=DigitIndex>,
{
    fn all_but(self) -> DigitMask {
        let mut flags = SUDOKU_FLAG_ALL;
        for d in self {
            flags.set(d.value(), false);
        }
        DigitMask(flags)
    }

    fn only(self) -> DigitMask {
        let mut flags = SudokuFlags::default();
        for d in self {
            flags.set(d.value(), true);
        }
        DigitMask(flags)
    }
}

impl core::ops::Index<DigitIndex> for DigitMask {
    type Output = bool;
    fn index(&self, index: DigitIndex) -> &Self::Output {
        &self.0[index.value()]
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, PartialOrd, Ord, Hash)]
pub struct SudokuCell(SudokuFlags);

impl core::default::Default for SudokuCell {
    fn default() -> Self {
        SudokuCell(SUDOKU_FLAG_ALL)
    }
}

impl SudokuCell {
    pub fn digit(d: DigitIndex) -> Self {
        let mut v = SudokuFlags::ZERO;
        v.set(d.value(), true);
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

    pub fn num_digits(&self) -> usize {
        self.0.count_ones()
    }

    pub fn apply_mask(&mut self, mask: &DigitMask) -> bool {
        if self.would_change(mask) {
            self.0 &= mask.value();
            true
        } else {
            false
        }
    }

    pub fn would_change(&self, mask: &DigitMask) -> bool {
        self.0 & mask.value() != self.0
    }

    pub fn contains(&self, d: DigitIndex) -> bool {
        self.0[d.value()]
    }

    pub fn digits(&self) -> SudokuFlags {
        self.0
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
            char::from_digit((d.value() + 1) as u32, 10).unwrap()
        } else {
            '0'
        }
    }
}

impl core::ops::BitAndAssign<DigitMask> for &mut SudokuCell {
    fn bitand_assign(&mut self, rhs: DigitMask) {
        self.0 &= rhs.value()
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