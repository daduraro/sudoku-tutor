use itertools::Itertools;

use crate::error::SudokuError;

use super::{RowIndex, ColumnIndex, BlockIndex, HouseIndex, LineDirection, RegionIntersection};

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

    pub fn block(&self) -> BlockIndex {
        let r = self.0.value();
        let c = self.1.value();

        BlockIndex::from_index(r/3, c/3).unwrap()
    }

    pub const fn flat(&self) -> usize {
        self.0.value() * 9 + self.1.value()
    }

    pub fn houses(&self) -> [HouseIndex; 3] {
        [
            HouseIndex::Row(self.row()),
            HouseIndex::Column(self.column()),
            HouseIndex::Block(self.block()),
        ]
    }

    pub fn visible(&self, other: &CellIndex) -> bool {
        self.row().value() == other.row().value() ||
            self.column().value() == other.column().value() ||
            self.block().value() == other.block().value()
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

    pub fn cells_visible_with(&self, other: &CellIndex) -> Vec<CellIndex> {
        let mut candidates: Vec<_> = self.houses().iter().cartesian_product(other.houses())
            .flat_map(|(a, b)| a.intersect(&b))
            .filter(|idx| idx != self && idx != other)
            .collect()
        ;
        candidates.sort();

        let mut unique = Vec::new();
        while let Some(idx) = candidates.pop() {
            if unique.last() == Some(&idx) { continue }
            else { unique.push(idx) }
        }

        unique
    }

    pub fn from_flat(i: usize) -> Result<CellIndex, SudokuError> {
        let r = i / ColumnIndex::COUNT;
        let c = i % ColumnIndex::COUNT;
        Ok(CellIndex(RowIndex::new(r)?, ColumnIndex::new(c)?))
    }

    pub fn iter() -> impl Iterator<Item = CellIndex> {
        (0..Self::COUNT).map(|i| {
            let r = i / ColumnIndex::COUNT;
            let c = i % ColumnIndex::COUNT;
            CellIndex(RowIndex::new(r).unwrap(), ColumnIndex::new(c).unwrap())
        })
    }

    pub const fn line(&self, direction: LineDirection) -> HouseIndex {
        match direction {
            LineDirection::Horizontal => HouseIndex::Row(self.row()),
            LineDirection::Vertical => HouseIndex::Column(self.column()),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_with() {
        for pairs in CellIndex::iter().combinations_with_replacement(2) {
            let c0 = pairs[0];
            let c1 = pairs[1];
            let visible: Vec<_> = c0.cells_visible_with(&c1).into_iter().sorted().collect();

            let ground_truth: Vec<_> = CellIndex::iter()
                .filter(|idx| idx != &c0 && idx != &c1 && idx.visible(&c0) && idx.visible(&c1)).sorted().collect();

            assert_eq!(visible, ground_truth, "c0: {:?}; c1: {:?}", c0, c1);
        }
    }
}