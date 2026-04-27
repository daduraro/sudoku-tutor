use super::{CellIndex, ColumnIndex, RowIndex, BlockIndex, HouseIndex, SudokuRegion};

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
