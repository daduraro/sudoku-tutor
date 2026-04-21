use itertools::Itertools;

use crate::{board::SudokuFlags};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct DigitIndex(u8);

impl DigitIndex {
    pub const COUNT: usize = 9;

    pub fn new(v: usize) -> Self {
        assert!(v < Self::COUNT);
        DigitIndex(v as u8)
    }

    pub const fn domain() -> &'static [DigitIndex; Self::COUNT] {
        &[
            DigitIndex(0), DigitIndex(1), DigitIndex(2),
            DigitIndex(3), DigitIndex(4), DigitIndex(5),
            DigitIndex(6), DigitIndex(7), DigitIndex(8),
        ]
    }

    pub fn value(&self) -> usize {
        self.0 as usize
    }
}

impl core::fmt::Display for DigitIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "d{}", self.0)
    }
}

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

pub trait HouseIndexer {
    fn cell_index(&self, idx: usize) -> CellIndex;

    fn contains(&self, cell_idx: CellIndex) -> bool {
        (0..9).any(|i| self.cell_index(i) == cell_idx)
    }

    fn cell_indices(&self) -> [CellIndex; 9] {
        core::array::from_fn(|idx| self.cell_index(idx))
    }
    fn flat_indices(&self) -> [usize; 9] {
        self.cell_indices().map(|idx| idx.flat())
    }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct RowIndex(u8);

impl RowIndex {
    pub const COUNT: usize = 9;

    pub fn new(v: usize) -> Self {
        assert!(v < Self::COUNT);
        RowIndex(v as u8)
    }

    pub const fn domain() -> &'static [RowIndex; Self::COUNT] {
        &[
            RowIndex(0), RowIndex(1), RowIndex(2),
            RowIndex(3), RowIndex(4), RowIndex(5),
            RowIndex(6), RowIndex(7), RowIndex(8),
        ]
    }

    pub fn value(&self) -> usize {
        self.0 as usize
    }
}

impl HouseIndexer for RowIndex {
    fn cell_index(&self, idx: usize) -> CellIndex {
        CellIndex::new(*self, ColumnIndex::new(idx))
    }
}

impl core::fmt::Display for RowIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.0)
    }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ColumnIndex(u8);

impl ColumnIndex {
    pub const COUNT: usize = 9;

    pub fn new(v: usize) -> Self {
        assert!(v < Self::COUNT);
        ColumnIndex(v as u8)
    }

    pub const fn domain() -> &'static [ColumnIndex; Self::COUNT] {
        &[
            ColumnIndex(0), ColumnIndex(1), ColumnIndex(2),
            ColumnIndex(3), ColumnIndex(4), ColumnIndex(5),
            ColumnIndex(6), ColumnIndex(7), ColumnIndex(8),
        ]
    }

    pub fn value(&self) -> usize {
        self.0 as usize
    }
}

impl HouseIndexer for ColumnIndex {
    fn cell_index(&self, idx: usize) -> CellIndex {
        CellIndex::new(RowIndex::new(idx), *self)
    }
}

impl core::fmt::Display for ColumnIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c{}", self.0)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct BlockIndex(u8);
impl BlockIndex {
    pub const COUNT: usize = 9;

    pub fn new(v: usize) -> Self {
        assert!(v < Self::COUNT);
        BlockIndex(v as u8)
    }

    pub const fn domain() -> &'static [BlockIndex; Self::COUNT] {
        &[
            BlockIndex(0), BlockIndex(1), BlockIndex(2),
            BlockIndex(3), BlockIndex(4), BlockIndex(5),
            BlockIndex(6), BlockIndex(7), BlockIndex(8),
        ]
    }

    pub fn value(&self) -> usize {
        self.0 as usize
    }
}

impl HouseIndexer for BlockIndex {
    fn cell_index(&self, idx: usize) -> CellIndex {
        assert!(idx < 9);
        let row = RowIndex::new((self.value() / 3) * 3 + idx /3);
        let col = ColumnIndex::new((self.value() % 3) * 3 + idx % 3);
        CellIndex::new(row, col)
    }
}

impl core::fmt::Display for BlockIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "b{}", self.0)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum HouseIndex {
    Row(RowIndex),
    Column(ColumnIndex),
    Block(BlockIndex),
}

impl HouseIndex {
    pub const fn domain() -> &'static [HouseIndex; 9*3] {
        &[
            HouseIndex::Row(RowIndex(0)), HouseIndex::Row(RowIndex(1)), HouseIndex::Row(RowIndex(2)),
            HouseIndex::Row(RowIndex(3)), HouseIndex::Row(RowIndex(4)), HouseIndex::Row(RowIndex(5)),
            HouseIndex::Row(RowIndex(6)), HouseIndex::Row(RowIndex(7)), HouseIndex::Row(RowIndex(8)),

            HouseIndex::Column(ColumnIndex(0)), HouseIndex::Column(ColumnIndex(1)), HouseIndex::Column(ColumnIndex(2)),
            HouseIndex::Column(ColumnIndex(3)), HouseIndex::Column(ColumnIndex(4)), HouseIndex::Column(ColumnIndex(5)),
            HouseIndex::Column(ColumnIndex(6)), HouseIndex::Column(ColumnIndex(7)), HouseIndex::Column(ColumnIndex(8)),

            HouseIndex::Block(BlockIndex(0)), HouseIndex::Block(BlockIndex(1)), HouseIndex::Block(BlockIndex(2)),
            HouseIndex::Block(BlockIndex(3)), HouseIndex::Block(BlockIndex(4)), HouseIndex::Block(BlockIndex(5)),
            HouseIndex::Block(BlockIndex(6)), HouseIndex::Block(BlockIndex(7)), HouseIndex::Block(BlockIndex(8)),
        ]
    }

    pub const fn rows_and_columns() ->  &'static [HouseIndex; 9*2] {
        &[
            HouseIndex::Row(RowIndex(0)), HouseIndex::Row(RowIndex(1)), HouseIndex::Row(RowIndex(2)),
            HouseIndex::Row(RowIndex(3)), HouseIndex::Row(RowIndex(4)), HouseIndex::Row(RowIndex(5)),
            HouseIndex::Row(RowIndex(6)), HouseIndex::Row(RowIndex(7)), HouseIndex::Row(RowIndex(8)),

            HouseIndex::Column(ColumnIndex(0)), HouseIndex::Column(ColumnIndex(1)), HouseIndex::Column(ColumnIndex(2)),
            HouseIndex::Column(ColumnIndex(3)), HouseIndex::Column(ColumnIndex(4)), HouseIndex::Column(ColumnIndex(5)),
            HouseIndex::Column(ColumnIndex(6)), HouseIndex::Column(ColumnIndex(7)), HouseIndex::Column(ColumnIndex(8)),
        ]
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

impl HouseIndexer for HouseIndex {
    fn cell_index(&self, idx: usize) -> CellIndex {
        match self {
            HouseIndex::Block(inner) => inner.cell_index(idx),
            HouseIndex::Row(inner) => inner.cell_index(idx),
            HouseIndex::Column(inner) => inner.cell_index(idx),
        }
    }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct CellIndex(RowIndex, ColumnIndex);

impl CellIndex {
    pub const COUNT: usize = ColumnIndex::COUNT * RowIndex::COUNT;

    pub fn new(r: RowIndex, c: ColumnIndex) -> Self {
        CellIndex(r, c)
    }

    pub fn row(&self) -> RowIndex {
        self.0
    }

    pub fn column(&self) -> ColumnIndex {
        self.1
    }

    pub fn block(&self) -> BlockIndex {
        let r = self.0.value();
        let c = self.1.value();
        let b = (r / 3) * 3 + c/3;

        BlockIndex(b as u8)
    }

    pub fn flat(&self) -> usize {
        self.0.value() * 9 + self.1.value()
    }

    pub fn houses(&self) -> [HouseIndex; 3] {
        [
            HouseIndex::from(self.row()),
            HouseIndex::from(self.column()),
            HouseIndex::from(self.block()),
        ]
    }

    pub fn share_house(&self, other: &CellIndex) -> bool {
        self.row() == other.row() ||
        self.column() == other.column() ||
        self.block() == other.block()
    }

    pub fn from_flat(i: usize) -> CellIndex{
        assert!(i < Self::COUNT);
        unsafe { Self::from_flat_unchecked(i) }
    }

    unsafe fn from_flat_unchecked(i: usize) -> CellIndex {
        let r = i / ColumnIndex::COUNT;
        let c = i % ColumnIndex::COUNT;
        CellIndex(RowIndex(r as u8), ColumnIndex(c as u8))
    }

    pub fn domain() -> impl Iterator<Item = CellIndex> {
        (0..Self::COUNT).map(|i| unsafe { Self::from_flat_unchecked(i) })
    }
}

pub trait CellIndexIter {
    fn share_row(self) -> bool;
    fn share_column(self) -> bool;
    fn share_block(self) -> bool;
    fn share_house(self) -> bool;
}

impl<T> CellIndexIter for T
where 
    T: Iterator<Item=CellIndex>
{
    fn share_row(self) -> bool {
        self.map(|idx| idx.row()).all_equal()
    }

    fn share_column(self) -> bool {
        self.map(|idx| idx.column()).all_equal()
    }

    fn share_block(self) -> bool {
        self.map(|idx| idx.block()).all_equal()
    }

    fn share_house(self) -> bool {
        let (rows, columns, blocks) = self.fold((SudokuFlags::ZERO, SudokuFlags::ZERO, SudokuFlags::ZERO), 
            |(mut rows, mut columns, mut blocks), idx|{
                rows.set(idx.row().value(), true);
                columns.set(idx.column().value(), true);
                blocks.set(idx.block().value(), true);
                (rows, columns, blocks)
            });
        rows.count_ones() == 1 || columns.count_ones() == 1 || blocks.count_ones() == 1
    }
}

impl core::fmt::Display for CellIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

pub type SudokuSubCellIndex = (CellIndex, DigitIndex);
