use crate::error::SudokuError;
use super::{HouseRegion, SudokuRegion, ChuteIndex, CellIndex, LineDirection};

macro_rules! basic_index_impl {
    ($($t:ty)*) => ($(
        impl $t {
            pub const COUNT: usize = 9;

            pub const fn new(index: usize) -> Result<Self, SudokuError> {
                if index < Self::COUNT {
                    Ok(Self(index as u8))
                } else {
                    Err(SudokuError::InvalidValue(index))
                }
            }

            pub const fn value(&self) -> usize {
                self.0 as usize
            }

            pub const fn const_eq(&self, other: &Self) -> bool {
                self.value() == other.value()
            }

            pub fn iter() -> impl Iterator<Item=Self> {
                (0..Self::COUNT).map(|i| Self::new(i).unwrap())
            }
        }
    )*);
}

macro_rules! into_iter_impl {
    ($($t:ty => $iter:tt),*) => ($(
        #[derive(Clone, Copy, Debug)]
        pub struct $iter($t, u8);
        impl Iterator for $iter {
            type Item = CellIndex;
            fn next(&mut self) -> Option<Self::Item> {
                ((self.1 as usize) < <$t>::COUNT).then(move || {
                    let next = self.0.cell_index(self.1 as usize).unwrap();
                    self.1 += 1;
                    next
                })
            }
        }

        impl core::iter::IntoIterator for $t {
            type Item = CellIndex;
            type IntoIter = $iter;
            fn into_iter(self) -> Self::IntoIter {
                $iter(self, 0)
            }
        }
    )*);
}

basic_index_impl! (SudokuIndex DigitIndex RowIndex ColumnIndex BlockIndex);
into_iter_impl! (RowIndex => RowIter, ColumnIndex => ColumnIter, BlockIndex => BlockIter);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct SudokuIndex(u8);

impl core::convert::From<DigitIndex> for SudokuIndex {
    fn from(value: DigitIndex) -> Self {
        Self(value.value() as u8)
    }
}

impl core::convert::From<ColumnIndex> for SudokuIndex {
    fn from(value: ColumnIndex) -> Self {
        Self(value.value() as u8)
    }
}

impl core::convert::From<RowIndex> for SudokuIndex {
    fn from(value: RowIndex) -> Self {
        Self(value.value() as u8)
    }
}

impl core::convert::From<BlockIndex> for SudokuIndex {
    fn from(value: BlockIndex) -> Self {
        Self(value.value() as u8)
    }
}

impl core::convert::From<HouseIndex> for SudokuIndex {
    fn from(value: HouseIndex) -> Self {
        match value {
            HouseIndex::Block(i) => i.into(),
            HouseIndex::Row(i) => i.into(),
            HouseIndex::Column(i) => i.into(),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct DigitIndex(u8);

impl core::convert::From<DigitIndex> for char {
    fn from(value: DigitIndex) -> Self {
        char::from(&value)
    }
}

impl core::convert::From<&DigitIndex> for char {
    fn from(digit: &DigitIndex) -> Self {
        char::from_digit((digit.value() + 1) as u32, 10).unwrap()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct RowIndex(u8);

impl RowIndex {
    pub fn chute(&self) -> ChuteIndex {
        ChuteIndex::new(LineDirection::Horizontal, self.value() / 3).unwrap()
    }
}

impl SudokuRegion for RowIndex {
    const CELL_COUNT: usize = RowIndex::COUNT;
    fn cell_index(&self, idx: usize) -> Result<CellIndex, SudokuError> {
        Ok(CellIndex::new(*self, ColumnIndex::new(idx)?))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ColumnIndex(u8);

impl ColumnIndex {  
    pub fn chute(&self) -> ChuteIndex {
        ChuteIndex::new(LineDirection::Vertical, self.value() / 3).unwrap()
    }
}

impl SudokuRegion for ColumnIndex {
    const CELL_COUNT: usize = 9;
    fn cell_index(&self, idx: usize) -> Result<CellIndex, SudokuError> {
        Ok(CellIndex::new(RowIndex::new(idx)?, *self))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct BlockIndex(u8);
impl BlockIndex {
    pub const fn index(&self) -> (usize, usize) {
        (self.chute_row(), self.chute_column())
    }

    pub const fn from_index(row: usize, column: usize) -> Result<Self, SudokuError> {
        Self::new(row*3 + column)
    }

    pub const fn chute_row(&self) -> usize {
        self.value() / 3
    }

    pub const fn chute_column(&self) -> usize {
        self.value() % 3
    }
}

impl SudokuRegion for BlockIndex {
    const CELL_COUNT: usize = 9;
    fn cell_index(&self, idx: usize) -> Result<CellIndex, SudokuError> {
        let row = RowIndex::new(self.chute_row() * 3 + idx /3)?;
        let col = ColumnIndex::new(self.chute_column() * 3 + idx % 3)?;
        Ok(CellIndex::new(row, col))
    }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum HouseIndex {
    Row(RowIndex),
    Column(ColumnIndex),
    Block(BlockIndex),
}

impl HouseIndex {
    pub fn iter() -> impl Iterator<Item=HouseIndex> {
        RowIndex::iter().map(HouseIndex::Row)
            .chain(ColumnIndex::iter().map(HouseIndex::Column))
            .chain(BlockIndex::iter().map(HouseIndex::Block))
    }

    pub fn rows_and_columns() -> impl Iterator<Item=HouseIndex> {
        RowIndex::iter().map(HouseIndex::Row)
            .chain(ColumnIndex::iter().map(HouseIndex::Column))
    }
}

impl core::convert::From<RowIndex> for HouseIndex {
    fn from(value: RowIndex) -> Self { HouseIndex::Row(value) }
}

impl core::convert::From<ColumnIndex> for HouseIndex {
    fn from(value: ColumnIndex) -> Self { HouseIndex::Column(value) }
}

impl core::convert::From<BlockIndex> for HouseIndex {
    fn from(value: BlockIndex) -> Self { HouseIndex::Block(value) }
}


impl SudokuRegion for HouseIndex {
    const CELL_COUNT: usize = 9;
    fn cell_index(&self, idx: usize) -> Result<CellIndex, SudokuError> {
        match self {
            HouseIndex::Block(inner) => inner.cell_index(idx),
            HouseIndex::Row(inner) => inner.cell_index(idx),
            HouseIndex::Column(inner) => inner.cell_index(idx),
        }
    }
}

impl HouseRegion for ColumnIndex {}
impl HouseRegion for RowIndex {}
impl HouseRegion for BlockIndex {}
impl HouseRegion for HouseIndex {}
