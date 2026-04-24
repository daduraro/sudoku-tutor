use std::ops::{Add, BitAnd};

use itertools::Itertools;
use ratatui::widgets::Block;
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

    pub const fn const_eq(&self, other: &Self) -> bool {
        self.index() == other.index()
    }

    pub const fn domain() -> [DigitIndex; 9] {
        [
            DigitIndex::D1, DigitIndex::D2, DigitIndex::D3,
            DigitIndex::D4, DigitIndex::D5, DigitIndex::D6,
            DigitIndex::D7, DigitIndex::D8, DigitIndex::D9,
        ]
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

pub trait SudokuRegion {
    const CELL_COUNT: usize;

    fn cell_index(&self, flat_index: usize) -> CellIndex;

    fn cell_indices(&self) -> impl Iterator<Item=CellIndex> {
        (0..Self::CELL_COUNT).map(|i| self.cell_index(i))
    }

    fn contains(&self, cell_idx: CellIndex) -> bool {
        self.cell_indices().any(|idx| idx == cell_idx)
    }

    fn flat_indices(&self) -> impl Iterator<Item=usize> {
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

    pub const fn chute(&self) -> ChuteIndex {
        ChuteIndex::new(LineDirection::Horizontal, self.index() / 3)
    }

    pub const fn const_eq(&self, other: &Self) -> bool {
        self.index() == other.index()
    }

    pub const fn domain() -> [RowIndex; 9] {
        [
            RowIndex::R1, RowIndex::R2, RowIndex::R3,
            RowIndex::R4, RowIndex::R5, RowIndex::R6,
            RowIndex::R7, RowIndex::R8, RowIndex::R9,
        ]
    }
}

impl SudokuRegion for RowIndex {
    const CELL_COUNT: usize = 9;
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

    pub const fn chute(&self) -> ChuteIndex {
        ChuteIndex::new(LineDirection::Vertical, self.index() / 3)
    }

    pub const fn const_eq(&self, other: &Self) -> bool {
        self.index() == other.index()
    }

    pub const fn domain() -> [ColumnIndex; 9] {
        [
            ColumnIndex::C1, ColumnIndex::C2, ColumnIndex::C3,
            ColumnIndex::C4, ColumnIndex::C5, ColumnIndex::C6,
            ColumnIndex::C7, ColumnIndex::C8, ColumnIndex::C9,
        ]
    }
}

impl SudokuRegion for ColumnIndex {
    const CELL_COUNT: usize = 9;
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

    pub const fn const_eq(&self, other: &Self) -> bool {
        self.flat_index() == other.flat_index()
    }
}

impl SudokuRegion for BlockIndex {
    const CELL_COUNT: usize = 9;
    fn cell_index(&self, idx: usize) -> CellIndex {
        assert!(idx < 9);
        let row = RowIndex::new(self.chute_row() * 3 + idx /3);
        let col = ColumnIndex::new(self.chute_column() * 3 + idx % 3);
        CellIndex::new(row, col)
    }
}

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

    pub const fn line(&self, index: usize) -> HouseIndex {
        match self {
            LineDirection::Horizontal => HouseIndex::Row(RowIndex::new(index)),
            LineDirection::Vertical => HouseIndex::Column(ColumnIndex::new(index)),
        }
    }

    pub const fn lines(&self) -> [HouseIndex; 9] {
        match self {
            // LineDirection::Horizontal => RowIndex::domain().map(HouseIndex::Row),
            // LineDirection::Vertical => ColumnIndex::domain().map(HouseIndex::Column),
            LineDirection::Horizontal => [
                HouseIndex::Row(RowIndex::R1), HouseIndex::Row(RowIndex::R2), HouseIndex::Row(RowIndex::R3),
                HouseIndex::Row(RowIndex::R4), HouseIndex::Row(RowIndex::R5), HouseIndex::Row(RowIndex::R6),
                HouseIndex::Row(RowIndex::R7), HouseIndex::Row(RowIndex::R8), HouseIndex::Row(RowIndex::R9),
            ],
            LineDirection::Vertical => [
                HouseIndex::Column(ColumnIndex::C1), HouseIndex::Column(ColumnIndex::C2), HouseIndex::Column(ColumnIndex::C3),
                HouseIndex::Column(ColumnIndex::C4), HouseIndex::Column(ColumnIndex::C5), HouseIndex::Column(ColumnIndex::C6),
                HouseIndex::Column(ColumnIndex::C7), HouseIndex::Column(ColumnIndex::C8), HouseIndex::Column(ColumnIndex::C9),
            ],
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, EnumIter, EnumCount)]
pub enum ChuteIndex {
    V1, V2, V3,
    H1, H2, H3,
}
impl ChuteIndex {
    pub const fn new(direction: LineDirection, index: usize) -> Self {
        assert!(index < 3);
        match (direction, index) {
            (LineDirection::Vertical, 0) => ChuteIndex::V1,
            (LineDirection::Vertical, 1) => ChuteIndex::V2,
            (LineDirection::Vertical, 2) => ChuteIndex::V3,
            (LineDirection::Horizontal, 0) => ChuteIndex::H1,
            (LineDirection::Horizontal, 1) => ChuteIndex::H2,
            (LineDirection::Horizontal, 2) => ChuteIndex::H3,
            _ => panic!()
        }
    }

    pub const fn const_eq(&self, other: &ChuteIndex) -> bool {
        self.index_value() == other.index_value() &&
            self.direction().const_eq(&other.direction())
    }

    pub const fn line(&self, index: usize) -> HouseIndex {
        assert!(index < 3);
        self.lines()[index]
    }

    pub const fn lines(&self) -> [HouseIndex; 3] {
        match self.index() {
            (LineDirection::Vertical, index_value) => [
                HouseIndex::Column(ColumnIndex::new(index_value*3   )),
                HouseIndex::Column(ColumnIndex::new(index_value*3 + 1)),
                HouseIndex::Column(ColumnIndex::new(index_value*3 + 2)),
            ],
            (LineDirection::Horizontal, index_value) => [
                HouseIndex::Row(RowIndex::new(index_value*3   )),
                HouseIndex::Row(RowIndex::new(index_value*3 + 1)),
                HouseIndex::Row(RowIndex::new(index_value*3 + 2)),
            ],
        }
    }

    pub const fn block(&self, index: usize) -> BlockIndex {
        self.blocks()[index]
    }

    pub const fn blocks(&self) -> [BlockIndex; 3] {
        match self.index() {
            (LineDirection::Horizontal, block_row) => [
                BlockIndex::new(block_row, 0),
                BlockIndex::new(block_row, 1),
                BlockIndex::new(block_row, 2),
            ],
            (LineDirection::Vertical, block_column) => [
                BlockIndex::new(0, block_column),
                BlockIndex::new(1, block_column),
                BlockIndex::new(2, block_column),
            ],
        }
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

impl SudokuRegion for ChuteIndex {
    const CELL_COUNT: usize = 9*3;
    fn cell_index(&self, flat_index: usize) -> CellIndex {
        self.block(flat_index/9).cell_index(flat_index % 9)
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

    pub fn rows_and_columns() ->  impl Iterator<Item=HouseIndex> {
        RowIndex::iter().map(HouseIndex::Row)
            .chain(ColumnIndex::iter().map(HouseIndex::Column))
    }

    // pub fn crossed_by(&self, idx: usize) -> Option<HouseIndex> {
    //     match self {
    //         HouseIndex::Column(h) => Some(h.cell_index(idx).row().into()),
    //         HouseIndex::Row(h) => Some(h.cell_index(idx).column().into()),
    //         HouseIndex::Block(_) => None,
    //     }
    // }
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

    pub const fn new(r: RowIndex, c: ColumnIndex) -> Self {
        CellIndex(r, c)
    }

    pub const fn row(&self) -> RowIndex {
        self.0
    }

    pub const fn column(&self) -> ColumnIndex {
        self.1
    }

    pub const fn block(&self) -> BlockIndex {
        let r = self.0.index();
        let c = self.1.index();

        BlockIndex::new(r/3, c/3)
    }

    pub const fn flat(&self) -> usize {
        self.0.index() * 9 + self.1.index()
    }

    pub const fn houses(&self) -> [HouseIndex; 3] {
        [
            HouseIndex::Row(self.row()),
            HouseIndex::Column(self.column()),
            HouseIndex::Block(self.block()),
        ]
    }

    pub const fn share_house(&self, other: &CellIndex) -> bool {
        self.row().index() == other.row().index() ||
            self.column().index() == other.column().index() ||
            self.block().flat_index() == other.block().flat_index()
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
            cells.extend(house.cell_indices().filter(|idx|{
                (idx != self) && (idx != other) && !others.iter().any(|h| h.contains(*idx))
            }));
        }

        if cells.is_empty() { // they do not share houses so add the crossovers
            cells.push(CellIndex::new(self.row(), other.column()));
            cells.push(CellIndex::new(other.row(), self.column()));
        }

        cells
    }

    pub const fn from_flat(i: usize) -> CellIndex{
        assert!(i < Self::COUNT);
        unsafe { Self::from_flat_unchecked(i) }
    }

    const unsafe fn from_flat_unchecked(i: usize) -> CellIndex {
        let r = i / ColumnIndex::COUNT;
        let c = i % ColumnIndex::COUNT;
        CellIndex(RowIndex::new(r), ColumnIndex::new(c))
    }

    pub fn iter() -> impl Iterator<Item = CellIndex> {
        (0..Self::COUNT).map(|i| unsafe { Self::from_flat_unchecked(i) })
    }

    pub const fn line(&self, direction: LineDirection) -> HouseIndex {
        match direction {
            LineDirection::Horizontal => HouseIndex::Row(self.row()),
            LineDirection::Vertical => HouseIndex::Column(self.column()),
        }
    }
}

// pub trait CellIndexIter {
//     fn share_row(self) -> bool;
//     fn share_column(self) -> bool;
//     fn share_block(self) -> bool;
//     fn share_house(self) -> bool;
// }

// impl<T> CellIndexIter for T
// where 
//     T: Iterator<Item=CellIndex>
// {
//     fn share_row(self) -> bool {
//         self.map(|idx| idx.row()).all_equal()
//     }

//     fn share_column(self) -> bool {
//         self.map(|idx| idx.column()).all_equal()
//     }

//     fn share_block(self) -> bool {
//         self.map(|idx| idx.block()).all_equal()
//     }

//     fn share_house(self) -> bool {
//         let (rows, columns, blocks) = self.fold((SudokuFlags::ZERO, SudokuFlags::ZERO, SudokuFlags::ZERO), 
//             |(mut rows, mut columns, mut blocks), idx|{
//                 rows.set(idx.row().index(), true);
//                 columns.set(idx.column().index(), true);
//                 blocks.set(idx.block().flat_index(), true);
//                 (rows, columns, blocks)
//             });
//         rows.count_ones() == 1 || columns.count_ones() == 1 || blocks.count_ones() == 1
//     }
// }

pub type SudokuSubCellIndex = (CellIndex, DigitIndex);

pub trait RegionIntersection<Rhs> {
    fn intersect(&self, rhs: &Rhs) -> Vec<CellIndex>;
}

macro_rules! impl_symmetrical_intersect {
    ($t:ty => $s:ty) => {
        impl RegionIntersection<$t> for $s {
            fn intersect(&self, other: &$t) -> Vec<CellIndex> {
                RegionIntersection::<$s>::intersect(other, self)
            }
        }
    }
}

impl RegionIntersection<RowIndex> for ColumnIndex {
    fn intersect(&self, row: &RowIndex) -> Vec<CellIndex> {
        vec![CellIndex::new(*row, *self)]
    }
}
impl_symmetrical_intersect!(ColumnIndex => RowIndex);

impl RegionIntersection<ColumnIndex> for BlockIndex {
    fn intersect(&self, column: &ColumnIndex) -> Vec<CellIndex> {
        column.cell_indices().filter(|idx| idx.block() == *self).collect()
    }
}
impl_symmetrical_intersect!(BlockIndex => ColumnIndex);

impl RegionIntersection<RowIndex> for BlockIndex {
    fn intersect(&self, row: &RowIndex) -> Vec<CellIndex> {
        row.cell_indices().filter(|idx| idx.block() == *self).collect()
    }
}
impl_symmetrical_intersect!(BlockIndex => RowIndex);

impl RegionIntersection<RowIndex> for RowIndex {
    fn intersect(&self, other: &RowIndex) -> Vec<CellIndex> {
        if self == other { self.cell_indices().collect() } else { Vec::new() }
    }
}

impl RegionIntersection<ColumnIndex> for ColumnIndex {
    fn intersect(&self, other: &ColumnIndex) -> Vec<CellIndex> {
        if self == other { self.cell_indices().collect() } else { Vec::new() }
    }
}

impl RegionIntersection<BlockIndex> for BlockIndex {
    fn intersect(&self, other: &BlockIndex) -> Vec<CellIndex> {
        if self == other { self.cell_indices().collect() } else { Vec::new() }
    }
}

macro_rules! impl_house_intersect {
    ($($t:ty)*) => ($(
        impl RegionIntersection<$t> for HouseIndex {
            fn intersect(&self, rhs: &$t) -> Vec<CellIndex> {
                match self {
                    HouseIndex::Block(b) => b.intersect(rhs),
                    HouseIndex::Column(c) => c.intersect(rhs),
                    HouseIndex::Row(r) => r.intersect(rhs),
                }
            }
        }
    )*);
}

impl_house_intersect!(RowIndex ColumnIndex BlockIndex HouseIndex);
impl_symmetrical_intersect!(HouseIndex => RowIndex);
impl_symmetrical_intersect!(HouseIndex => ColumnIndex);
impl_symmetrical_intersect!(HouseIndex => BlockIndex);
