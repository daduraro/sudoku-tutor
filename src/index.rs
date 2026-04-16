#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct DigitIndex(usize);

impl DigitIndex {
    pub fn new(v: usize) -> Self {
        assert!(v < 9);
        DigitIndex(v)
    }

    pub const fn domain() -> &'static [DigitIndex] {
        &[
            DigitIndex(0), DigitIndex(1), DigitIndex(2),
            DigitIndex(3), DigitIndex(4), DigitIndex(5),
            DigitIndex(6), DigitIndex(7), DigitIndex(8),
        ]
    }

}

impl core::fmt::Display for DigitIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "d{}", self.0)
    }
}

impl core::convert::From<usize> for DigitIndex {
    fn from(value: usize) -> Self {
        DigitIndex::new(value)
    }
}

impl core::ops::Deref for DigitIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0   
    }
}

pub trait HouseIndexer {
    fn index(&self, idx: usize) -> CellIndex;

    fn contains(&self, cell_idx: CellIndex) -> bool {
        (0..9).any(|i| self.index(i) == cell_idx)
    }

    fn indices(&self) -> [CellIndex; 9] {
        core::array::from_fn(|idx| self.index(idx))
    }
    fn flat_indices(&self) -> [usize; 9] {
        self.indices().map(|idx| idx.flat())
    }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct RowIndex(usize);

impl RowIndex {
    pub fn new(v: usize) -> Self {
        assert!(v < 9);
        RowIndex(v)
    }

    pub const fn domain() -> &'static [RowIndex] {
        &[
            RowIndex(0), RowIndex(1), RowIndex(2),
            RowIndex(3), RowIndex(4), RowIndex(5),
            RowIndex(6), RowIndex(7), RowIndex(8),
        ]
    }
}

impl HouseIndexer for RowIndex {
    fn index(&self, idx: usize) -> CellIndex {
        CellIndex::new(*self, ColumnIndex::new(idx))
    }
}

impl core::fmt::Display for RowIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.0)
    }
}

impl core::convert::From<usize> for RowIndex {
    fn from(value: usize) -> Self {
        RowIndex::new(value)
    }
}

impl core::ops::Deref for RowIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0   
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ColumnIndex(usize);

impl ColumnIndex {
    pub fn new(v: usize) -> Self {
        assert!(v < 9);
        ColumnIndex(v)
    }

    pub const fn domain() -> &'static [ColumnIndex] {
        &[
            ColumnIndex(0), ColumnIndex(1), ColumnIndex(2),
            ColumnIndex(3), ColumnIndex(4), ColumnIndex(5),
            ColumnIndex(6), ColumnIndex(7), ColumnIndex(8),
        ]
    }
}

impl HouseIndexer for ColumnIndex {
    fn index(&self, idx: usize) -> CellIndex {
        CellIndex::new(RowIndex::new(idx), *self)
    }
}

impl core::fmt::Display for ColumnIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c{}", self.0)
    }
}

impl core::convert::From<usize> for ColumnIndex {
    fn from(value: usize) -> Self {
        ColumnIndex::new(value)
    }
}

impl core::ops::Deref for ColumnIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0   
    }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct BlockIndex(usize);
impl BlockIndex {
    pub fn new(v: usize) -> Self {
        assert!(v < 9);
        BlockIndex(v)
    }

    pub const fn domain() -> &'static [BlockIndex] {
        &[
            BlockIndex(0), BlockIndex(1), BlockIndex(2),
            BlockIndex(3), BlockIndex(4), BlockIndex(5),
            BlockIndex(6), BlockIndex(7), BlockIndex(8),
        ]
    }
}

impl HouseIndexer for BlockIndex {
    fn index(&self, idx: usize) -> CellIndex {
        assert!(idx < 9);
        let row = RowIndex::new((**self / 3) * 3 + idx /3);
        let col = ColumnIndex::new((**self % 3) * 3 + idx % 3);
        CellIndex::new(row, col)
    }
}

impl core::fmt::Display for BlockIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "b{}", self.0)
    }
}

impl core::convert::From<usize> for BlockIndex {
    fn from(value: usize) -> Self {
        BlockIndex::new(value)
    }
}

impl core::convert::From<&CellIndex> for BlockIndex {
    fn from(value: &CellIndex) -> Self {
        let (r, c) = **value;
        BlockIndex::new((r / 3) * 3 + c/3)
    }
}

impl core::ops::Deref for BlockIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0   
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum HouseIndex {
    Row(RowIndex),
    Column(ColumnIndex),
    Block(BlockIndex),
}

impl HouseIndex {
    pub const fn domain() -> &'static [HouseIndex] {
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
    fn index(&self, idx: usize) -> CellIndex {
        match self {
            HouseIndex::Block(inner) => inner.index(idx),
            HouseIndex::Row(inner) => inner.index(idx),
            HouseIndex::Column(inner) => inner.index(idx),
        }
    }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct CellIndex((usize, usize));

impl CellIndex {
    pub fn new(r: RowIndex, c: ColumnIndex) -> Self {
        CellIndex((*r, *c))
    }

    pub fn row(&self) -> RowIndex {
        RowIndex(self.0.0)
    }

    pub fn column(&self) -> ColumnIndex {
        ColumnIndex(self.0.1)
    }

    pub fn block(&self) -> BlockIndex {
        self.into()
    }

    pub fn flat(&self) -> usize {
        let (r, c) = self.0;
        r * 9 + c
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
}

impl core::fmt::Display for CellIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0.0, self.0.1)
    }
}

impl core::convert::From<(usize, usize)> for CellIndex {
    fn from((r, c): (usize, usize)) -> Self {
        CellIndex::new(RowIndex::from(r), ColumnIndex::from(c))
    }
}

impl core::convert::From<usize> for CellIndex {
    fn from(value: usize) -> Self {
        CellIndex::new(RowIndex::from(value / 9), ColumnIndex(value % 9))
    }
}

impl core::ops::Deref for CellIndex {
    type Target = (usize, usize);

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}



pub type SudokuSubCellIndex = (CellIndex, DigitIndex);
