use core::ops::{Add, BitOr};
use std::iter::zip;

use itertools::Itertools;

use crate::error::SudokuError;
use crate::flags::{DigitFlags, SudokuFlags};
use crate::index::{
    CellIndex, DigitIndex, HouseIndex, HouseRegion, SudokuIndex, SudokuRegion, SudokuSubCellIndex,
};

use super::{SudokuCell, SudokuStringDecoding};

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

    pub fn region<Idx: SudokuRegion>(&self, idx: &Idx) -> impl Iterator<Item = &SudokuCell> {
        idx.cell_indices()
            .map(|cell_idx: CellIndex| &self[cell_idx])
    }
    pub fn region_mut<Idx: SudokuRegion>(
        &mut self,
        idx: &Idx,
    ) -> impl Iterator<Item = &mut SudokuCell> {
        debug_assert!(idx.flat_indices().all_unique());
        let ptr = self.0.as_mut_ptr();
        idx.flat_indices()
            .map(move |offset| unsafe { &mut *ptr.add(offset) })
    }
    pub fn indexed_region<Idx: SudokuRegion>(
        &self,
        idx: &Idx,
    ) -> impl Iterator<Item = (CellIndex, &SudokuCell)> {
        idx.cell_indices()
            .map(move |cell_idx| (cell_idx, &self[cell_idx]))
    }
    pub fn indexed_region_mut<Idx: SudokuRegion>(
        &mut self,
        idx: &Idx,
    ) -> impl Iterator<Item = (CellIndex, &mut SudokuCell)> {
        debug_assert!(
            idx.cell_indices()
                .map(|cell_idx| cell_idx.flat())
                .all_unique()
        );
        let ptr = self.0.as_mut_ptr();
        idx.cell_indices()
            .map(move |cell_idx| (cell_idx, unsafe { &mut *ptr.add(cell_idx.flat()) }))
    }

    pub fn enumerate_house<Idx: HouseRegion>(
        &self,
        idx: &Idx,
    ) -> impl Iterator<Item = (SudokuIndex, &SudokuCell)> {
        idx.cell_indices()
            .enumerate()
            .map(|(i, cell_idx)| (SudokuIndex::new(i).unwrap(), &self[cell_idx]))
    }

    pub fn cells_with<Idx: HouseRegion>(&self, house: &Idx, digit: DigitIndex) -> SudokuFlags {
        self.enumerate_house(house)
            .filter(move |(_, cell)| cell.contains(digit))
            .map(|(i, _)| i)
            .fold(SudokuFlags::ZERO, SudokuFlags::add)
    }

    pub fn primaries(&self) -> impl Iterator<Item = (CellIndex, DigitIndex)> {
        CellIndex::iter().primaries(self)
    }

    pub fn primaries_in<Idx: SudokuRegion>(
        &self,
        region: &Idx,
    ) -> impl Iterator<Item = (CellIndex, DigitIndex)> {
        region.cell_indices().primaries(self)
    }

    pub fn bivalues(&self) -> impl Iterator<Item = (CellIndex, DigitFlags)> {
        CellIndex::iter().bivalues(self)
    }

    pub fn bivalues_in<Region: SudokuRegion>(
        &self,
        region: &Region,
    ) -> impl Iterator<Item = (CellIndex, DigitFlags)> {
        region.cell_indices().bivalues(self)
    }

    pub fn iter(&self) -> impl Iterator<Item = &SudokuCell> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut SudokuCell> {
        self.0.iter_mut()
    }
    pub fn indexed_iter(&self) -> impl Iterator<Item = (CellIndex, &SudokuCell)> {
        self.0
            .iter()
            .enumerate()
            .map(|(i, cell)| (CellIndex::from_flat(i).unwrap(), cell))
    }
    pub fn indexed_iter_mut(&mut self) -> impl Iterator<Item = (CellIndex, &mut SudokuCell)> {
        self.0
            .iter_mut()
            .enumerate()
            .map(|(i, cell)| (CellIndex::from_flat(i).unwrap(), cell))
    }

    pub fn is_valid(&self) -> bool {
        self.iter().all(SudokuCell::is_valid)
    }
    pub fn is_solved(&self) -> bool {
        HouseIndex::iter().all(|h| {
            let set_digits = self
                .region(&h)
                .filter_map(|cell| cell.digit_value())
                .fold(DigitFlags::ZERO, DigitFlags::add)
                .count();
            set_digits == 9
        })
    }

    pub fn is_finished(&self) -> bool {
        self.iter().all(|cell| cell.num_digits() <= 1)
    }

    pub fn diff(&self, prev: &SudokuBoard) -> Vec<SudokuSubCellIndex> {
        zip(self.indexed_iter(), prev)
            .flat_map(|((cell_idx, curr), prev)| {
                DigitIndex::iter().filter_map(move |d| {
                    let has_diff = curr.contains(d) ^ prev.contains(d);
                    has_diff.then_some((cell_idx, d))
                })
            })
            .collect()
    }

    pub fn is_bilocation_link(
        &self,
        cell_a: CellIndex,
        cell_b: CellIndex,
        digit: DigitIndex,
    ) -> bool {
        // bilocation link is when two visible cells are the only two cells
        // with a specific digit in a house, that means either one or the other must
        // contain the digit.
        cell_a
            .shared_houses(&cell_b)
            .into_iter()
            .any(|house| self.cells_with(&house, digit).count() == 2)
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
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'board> IntoIterator for &'board SudokuBoard {
    type Item = &'board SudokuCell;
    type IntoIter = <&'board std::vec::Vec<SudokuCell> as std::iter::IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'board> IntoIterator for &'board mut SudokuBoard {
    type Item = &'board mut SudokuCell;
    type IntoIter = <&'board mut std::vec::Vec<SudokuCell> as std::iter::IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
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

pub trait SudokuBoardIter {
    fn primaries(self, board: &SudokuBoard) -> impl Iterator<Item = (CellIndex, DigitIndex)>;
    fn bivalues(self, board: &SudokuBoard) -> impl Iterator<Item = (CellIndex, DigitFlags)>;
    fn digits(self, board: &SudokuBoard) -> DigitFlags;
}

impl<Iter> SudokuBoardIter for Iter
where
    Iter: Iterator<Item = CellIndex>,
{
    fn primaries(self, board: &SudokuBoard) -> impl Iterator<Item = (CellIndex, DigitIndex)> {
        self.filter_map(|idx| board[idx].digit_value().map(move |d| (idx, d)))
    }

    fn bivalues(self, board: &SudokuBoard) -> impl Iterator<Item = (CellIndex, DigitFlags)> {
        self.filter(|idx| board[idx].num_digits() == 2)
            .map(|idx| (idx, board[idx].digit_flags()))
    }

    fn digits(self, board: &SudokuBoard) -> DigitFlags {
        self.map(|idx| board[idx].digit_flags())
            .fold(DigitFlags::ZERO, DigitFlags::bitor)
    }
}

// impl<'board, Iter> SudokuBoardIter for Iter
// where
//     Iter: Iterator<Item=&'board CellIndex>
// {
//     fn primaries(self, board: &SudokuBoard) -> impl Iterator<Item = (CellIndex, DigitIndex)> {
//         self.filter_map(|idx| board[idx].digit_value().map(move |d| (*idx, d)))
//     }

//     fn bivalues(self, board: &SudokuBoard) -> impl Iterator<Item = (CellIndex, DigitFlags)> {
//         self.filter(|idx| board[*idx].num_digits() == 2).map(|idx| (*idx, board[idx].digit_flags()))
//     }

//     fn digits(self, board: &SudokuBoard) -> DigitFlags {
//         self.map(|idx| board[idx].digit_flags()).fold(DigitFlags::ZERO, DigitFlags::bitor)
//     }
// }
