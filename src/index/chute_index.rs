use strum::{EnumCount, EnumIter};

use crate::error::SudokuError;
use super::{LineDirection, HouseIndex, ColumnIndex, RowIndex, BlockIndex, SudokuRegion, CellIndex};

#[derive(PartialEq, Eq, Clone, Copy, Debug, EnumCount, EnumIter)]
pub enum ChuteIndex {
    V1, V2, V3,
    H1, H2, H3,
}

impl ChuteIndex {
    pub const fn new(direction: LineDirection, index: usize) -> Result<Self, SudokuError> {
        match (direction, index) {
            (LineDirection::Vertical, 0) => Ok(ChuteIndex::V1),
            (LineDirection::Vertical, 1) => Ok(ChuteIndex::V2),
            (LineDirection::Vertical, 2) => Ok(ChuteIndex::V3),
            (LineDirection::Horizontal, 0) => Ok(ChuteIndex::H1),
            (LineDirection::Horizontal, 1) => Ok(ChuteIndex::H2),
            (LineDirection::Horizontal, 2) => Ok(ChuteIndex::H3),
            _ => Err(SudokuError::InvalidValue(index))
        }
    }

    pub const fn const_eq(&self, other: &ChuteIndex) -> bool {
        self.index_value() == other.index_value() &&
            self.direction().const_eq(&other.direction())
    }

    pub fn line(&self, index: usize) -> Result<HouseIndex, SudokuError> {
        match self.index() {
            (LineDirection::Vertical, index_value) => ColumnIndex::new(index_value*3 + index).map(Into::into),
            (LineDirection::Horizontal, index_value) => RowIndex::new(index_value*3 + index).map(Into::into),
        }
    }

    pub fn lines(&self) -> impl Iterator<Item=HouseIndex> {
        (0..3).map(|i| self.line(i).unwrap())
    }

    pub fn block(&self, index: usize) -> Result<BlockIndex, SudokuError> {
        match self.index() {
            (LineDirection::Vertical, block_column) => BlockIndex::from_index(index, block_column),
            (LineDirection::Horizontal, block_row) => BlockIndex::from_index(block_row, index),
        }
    }

    pub fn blocks(&self) -> impl Iterator<Item=BlockIndex> {
        (0..3).map(|i| self.block(i).unwrap())
    }

    pub const fn direction(&self) -> LineDirection {
        match self {
            ChuteIndex::V1 | ChuteIndex::V2 | ChuteIndex::V3 => LineDirection::Vertical,
            ChuteIndex::H1 | ChuteIndex::H2 | ChuteIndex::H3 => LineDirection::Horizontal,
        }
    }

    pub const fn index_value(&self) -> usize {
        match self {
            ChuteIndex::V1 | ChuteIndex::H1 => 0,
            ChuteIndex::V2 | ChuteIndex::H2 => 1,
            ChuteIndex::V3 | ChuteIndex::H3 => 2,
        }
    }

    pub const fn index(&self) -> (LineDirection, usize) {
        (self.direction(), self.index_value())
    }

    pub const fn contains_block(&self, block: BlockIndex) -> bool {
        matches!((self, block.chute_row(), block.chute_column()),
            (ChuteIndex::V1, _, 0) |
            (ChuteIndex::V2, _, 1) |
            (ChuteIndex::V3, _, 2) |
            (ChuteIndex::H1, 0, _) |
            (ChuteIndex::H2, 1, _) |
            (ChuteIndex::H3, 2, _)
        )
    }

    pub const fn contains_row(&self, row: RowIndex) -> bool {
        matches!((self, row.value() / 3),
            (ChuteIndex::H1, 0) |
            (ChuteIndex::H2, 1) |
            (ChuteIndex::H3, 2)
        )
    }

    pub const fn contains_column(&self, column: ColumnIndex) -> bool {
        matches!((self, column.value() / 3),
            (ChuteIndex::V1, 0) |
            (ChuteIndex::V2, 1) |
            (ChuteIndex::V3, 2)
        )
    }

    pub const fn contains(&self, house: HouseIndex) -> bool {
        match house {
            HouseIndex::Row(row) => self.contains_row(row),
            HouseIndex::Column(column) => self.contains_column(column),
            HouseIndex::Block(block) => self.contains_block(block),
        }
    }

}

impl SudokuRegion for ChuteIndex {
    const CELL_COUNT: usize = 9*3;
    fn cell_index(&self, flat_index: usize) -> Result<CellIndex, SudokuError> {
        self.block(flat_index/9)?.cell_index(flat_index % 9)
    }
}
