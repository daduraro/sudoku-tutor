use itertools::Itertools;
use strum::{EnumCount, EnumIter, FromRepr, IntoEnumIterator};

use crate::{board::SudokuFlags};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, EnumIter, EnumCount, FromRepr)]
pub enum DigitIndex {
    D1, D2, D3, D4, D5, D6, D7, D8, D9,
}

impl DigitIndex {
    pub const fn new(index: usize) -> Self {
        DigitIndex::from_repr(index).unwrap()
    }

    pub const fn index(&self) -> usize {
        *self as usize
    }
}

impl core::convert::From<DigitIndex> for char {
    fn from(value: DigitIndex) -> Self {
        char::from(&value)
    }
}

impl core::convert::From<&DigitIndex> for char {
    fn from(digit: &DigitIndex) -> Self {
        char::from_digit((digit.index() + 1) as u32, 10).unwrap()
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


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, EnumCount, FromRepr, EnumIter)]
pub enum RowIndex {
    R1, R2, R3, R4, R5, R6, R7, R8, R9,
}

impl RowIndex {
    pub const fn new(index: usize) -> Self {
        RowIndex::from_repr(index).unwrap()
    }

    pub const fn index(&self) -> usize {
        *self as usize
    }
}

impl HouseIndexer for RowIndex {
    fn cell_index(&self, idx: usize) -> CellIndex {
        CellIndex::new(*self, ColumnIndex::new(idx))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, EnumCount, FromRepr, EnumIter)]
pub enum ColumnIndex {
    C1, C2, C3, C4, C5, C6, C7, C8, C9,
}

impl ColumnIndex {
    pub const fn new(index: usize) -> Self {
        ColumnIndex::from_repr(index).unwrap()
    }

    pub const fn index(&self) -> usize {
        *self as usize
    }
}

impl HouseIndexer for ColumnIndex {
    fn cell_index(&self, idx: usize) -> CellIndex {
        CellIndex::new(RowIndex::new(idx), *self)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, EnumCount, FromRepr, EnumIter)]
pub enum BlockIndex {
    R1C1, R1C2, R1C3, R2C1, R2C2, R2C3, R3C1, R3C2, R3C3,
}
impl BlockIndex {
    pub const fn new(row: usize, column: usize) -> Self {
        assert!(column < 3 && row < 3);
        BlockIndex::from_repr(row*3 + column).unwrap()
    }

    pub const fn from_flat_index(index: usize) -> Self {
        BlockIndex::from_repr(index).unwrap()
    }

    pub const fn chute_row(&self) -> usize {
        match self {
            BlockIndex::R1C1 | BlockIndex::R1C2 | BlockIndex::R1C3 => 0,
            BlockIndex::R2C1 | BlockIndex::R2C2 | BlockIndex::R2C3 => 1,
            BlockIndex::R3C1 | BlockIndex::R3C2 | BlockIndex::R3C3 => 2,
        }
    }

    pub const fn chute_column(&self) -> usize {
        match self {
            BlockIndex::R1C1 | BlockIndex::R2C1 | BlockIndex::R3C1 => 0,
            BlockIndex::R1C2 | BlockIndex::R2C2 | BlockIndex::R3C2 => 1,
            BlockIndex::R1C3 | BlockIndex::R2C3 | BlockIndex::R3C3 => 2,
        }
    }

    pub const fn index(&self) -> (usize, usize) {
        (self.chute_row(), self.chute_column())
    }

    pub const fn flat_index(&self) -> usize {
        *self as usize
    }
}

impl HouseIndexer for BlockIndex {
    fn cell_index(&self, idx: usize) -> CellIndex {
        assert!(idx < 9);
        let row = RowIndex::new(self.chute_row() * 3 + idx /3);
        let col = ColumnIndex::new(self.chute_column() * 3 + idx % 3);
        CellIndex::new(row, col)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChuteDirection {
    Vertical,
    Horizontal,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, EnumIter, EnumCount)]
pub enum ChuteIndex {
    V1, V2, V3,
    H1, H2, H3,
}
impl ChuteIndex {
    pub const fn new(direction: ChuteDirection, index: usize) -> Self {
        assert!(index < 3);
        match (direction, index) {
            (ChuteDirection::Vertical, 0) => ChuteIndex::V1,
            (ChuteDirection::Vertical, 1) => ChuteIndex::V2,
            (ChuteDirection::Vertical, 2) => ChuteIndex::V3,
            (ChuteDirection::Horizontal, 0) => ChuteIndex::H1,
            (ChuteDirection::Horizontal, 1) => ChuteIndex::H2,
            (ChuteDirection::Horizontal, 2) => ChuteIndex::H3,
            _ => panic!()
        }
    }

    pub const fn direction(&self) -> ChuteDirection {
        match self {
            ChuteIndex::V1 | ChuteIndex::V2 | ChuteIndex::V3 => ChuteDirection::Vertical,
            ChuteIndex::H1 | ChuteIndex::H2 | ChuteIndex::H3 => ChuteDirection::Horizontal,
        }
    }

    pub const fn index_value(&self) -> usize {
        match self {
            ChuteIndex::V1 | ChuteIndex::H1 => 0,
            ChuteIndex::V2 | ChuteIndex::H2 => 1,
            ChuteIndex::V3 | ChuteIndex::H3 => 2,
        }
    }

    pub const fn index(&self) -> (ChuteDirection, usize) {
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
        matches!((self, row.index() / 3),
            (ChuteIndex::H1, 0) |
            (ChuteIndex::H2, 1) |
            (ChuteIndex::H3, 2)
        )
    }

    pub const fn contains_column(&self, column: ColumnIndex) -> bool {
        matches!((self, column.index() / 3),
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum HouseIndex {
    Row(RowIndex),
    Column(ColumnIndex),
    Block(BlockIndex),
}

impl HouseIndex {
    pub fn iter() -> impl Iterator<Item=HouseIndex> {
        RowIndex::iter().map(|idx| HouseIndex::Row(idx))
            .chain(ColumnIndex::iter().map(|idx| HouseIndex::Column(idx)))
            .chain(BlockIndex::iter().map(|idx| HouseIndex::Block(idx)))
    }

    pub fn rows_and_columns() ->  impl Iterator<Item=HouseIndex> {
        RowIndex::iter().map(|idx| HouseIndex::Row(idx))
            .chain(ColumnIndex::iter().map(|idx| HouseIndex::Column(idx)))
    }

    pub fn crossed_by(&self, idx: usize) -> Option<HouseIndex> {
        match self {
            HouseIndex::Column(h) => Some(h.cell_index(idx).row().into()),
            HouseIndex::Row(h) => Some(h.cell_index(idx).column().into()),
            HouseIndex::Block(_) => None,
        }
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
        let r = self.0.index();
        let c = self.1.index();

        BlockIndex::new(r, c)
    }

    pub fn flat(&self) -> usize {
        self.0.index() * 9 + self.1.index()
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

    pub fn shared_houses(&self, other: &CellIndex) -> Vec<HouseIndex> {
        let mut shared_houses: Vec<HouseIndex> = Vec::new();
        if self.block() == other.block() {
            shared_houses.push(self.block().into());
        }
        if self.row() == other.row() {
            shared_houses.push(self.row().into());
        }
        if self.column() == other.column() {
            shared_houses.push(self.column().into());
        }
        shared_houses
    }

    pub fn visible_with(&self, other: &CellIndex) -> Vec<CellIndex> {
        let shared_houses = self.shared_houses(other);

        let mut cells: Vec<CellIndex> = Vec::new();
        for i in 0..shared_houses.len() {
            let &house = &shared_houses[i];
            let others = &shared_houses[i+1..];
            cells.extend(house.cell_indices().into_iter().filter(|idx|{
                (idx != self) && (idx != other) && !others.iter().any(|h| h.contains(*idx))
            }));
        }

        if cells.is_empty() { // they do not share houses so add the crossovers
            cells.push(CellIndex::new(self.row(), other.column()));
            cells.push(CellIndex::new(other.row(), self.column()));
        }

        cells
    }

    pub fn from_flat(i: usize) -> CellIndex{
        assert!(i < Self::COUNT);
        unsafe { Self::from_flat_unchecked(i) }
    }

    unsafe fn from_flat_unchecked(i: usize) -> CellIndex {
        let r = i / ColumnIndex::COUNT;
        let c = i % ColumnIndex::COUNT;
        CellIndex(RowIndex::new(r), ColumnIndex::new(c))
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
                rows.set(idx.row().index(), true);
                columns.set(idx.column().index(), true);
                blocks.set(idx.block().flat_index(), true);
                (rows, columns, blocks)
            });
        rows.count_ones() == 1 || columns.count_ones() == 1 || blocks.count_ones() == 1
    }
}

pub type SudokuSubCellIndex = (CellIndex, DigitIndex);
