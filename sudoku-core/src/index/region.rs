use crate::error::SudokuError;
use super::{CellIndex, SudokuIndex};

pub trait SudokuRegion {
    const CELL_COUNT: usize;

    fn cell_index(&self, flat_index: usize) -> Result<CellIndex, SudokuError>;

    fn cell_indices(&self) -> impl Iterator<Item=CellIndex> {
        (0..Self::CELL_COUNT).map(|i| self.cell_index(i).unwrap())
    }

    fn contains(&self, cell_idx: CellIndex) -> bool {
        self.cell_indices().any(|idx| idx == cell_idx)
    }

    fn flat_indices(&self) -> impl Iterator<Item=usize> {
        self.cell_indices().map(|idx| idx.flat())
    }
}

pub trait HouseRegion: SudokuRegion {
    fn get(&self, index: SudokuIndex) -> CellIndex {
        self.cell_index(index.value()).unwrap()
    }
}
