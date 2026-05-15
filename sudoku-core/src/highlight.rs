use crate::index::{CellIndex, HouseIndex, SudokuSubCellIndex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Highlight {
    Digit(SudokuSubCellIndex),
    House(HouseIndex),
    Cell(CellIndex),
}

impl<Idx> core::convert::From<Idx> for Highlight
where Idx: core::convert::Into<HouseIndex> {
    fn from(value: Idx) -> Self {
        Highlight::House(value.into())
    }
}

impl core::convert::From<SudokuSubCellIndex> for Highlight {
    fn from(value: SudokuSubCellIndex) -> Self {
        Highlight::Digit(value)
    }
}

impl core::convert::From<CellIndex> for Highlight {
    fn from(value: CellIndex) -> Self {
        Highlight::Cell(value)
    }
}
