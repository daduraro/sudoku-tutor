pub mod basic;
pub mod region;
pub mod chute_index;
pub mod cell_index;
pub mod intersect;

pub use basic::{DigitIndex, BlockIndex, RowIndex, ColumnIndex, HouseIndex, SudokuIndex};
pub use cell_index::CellIndex;
pub use region::{SudokuRegion, HouseRegion};
pub use intersect::RegionIntersection;
pub use chute_index::ChuteIndex;

use strum::{EnumCount, EnumIter};

use crate::error::SudokuError;

#[derive(PartialEq, Eq, Clone, Copy, Debug, EnumIter, EnumCount)]
pub enum LineDirection {
    Vertical,
    Horizontal,
}

impl LineDirection {
    pub const fn const_eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }

    pub const fn other(self) -> Self {
        match self {
            LineDirection::Horizontal => LineDirection::Vertical,
            LineDirection::Vertical => LineDirection::Horizontal,
        }
    }

    pub fn line(&self, index: usize) -> Result<HouseIndex, SudokuError> {
        match self {
            LineDirection::Horizontal => Ok(HouseIndex::Row(RowIndex::new(index)?)),
            LineDirection::Vertical => Ok(HouseIndex::Column(ColumnIndex::new(index)?)),
        }
    }

    pub fn lines(&self) -> [HouseIndex; 9] {
        match self {
            // LineDirection::Horizontal => RowIndex::domain().map(HouseIndex::Row),
            // LineDirection::Vertical => ColumnIndex::domain().map(HouseIndex::Column),
            LineDirection::Horizontal => 
                core::array::from_fn(|i| HouseIndex::Row(RowIndex::new(i).unwrap())),
            LineDirection::Vertical => 
                core::array::from_fn(|i| HouseIndex::Column(ColumnIndex::new(i).unwrap())),
        }
    }
}

pub type SudokuSubCellIndex = (CellIndex, DigitIndex);

