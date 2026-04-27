use crate::index::{BlockIndex, ColumnIndex, DigitIndex, RowIndex, SudokuIndex};

macro_rules! sudoku_flags {
    ($t:tt $idx:tt) => {
        impl $t {
            pub const ZERO: Self = Self(0);
            pub const ALL: Self = Self(0b111_111_111);

            pub const fn new(flags: u16) -> Self {
                Self(flags & 0b111_111_111)
            }

            pub const fn only(idx: $idx) -> Self {
                Self(1 << idx.value())
            }

            pub fn all_but(idx: $idx) -> Self {
                Self::new(!(1 << idx.value()))
            }

            fn select_unchecked(&self, idx: usize) -> bool {
                (self.0 & (1 << idx)) != 0
            }

            pub fn iter(&self) -> impl Iterator<Item=$idx> {
                (0..9).filter(|i| self.select_unchecked(*i)).map(|i| $idx::new(i).unwrap())
            }

            pub const fn any(&self) -> bool {
                self.0 != 0
            }

            pub const fn count(&self) -> usize {
                self.0.count_ones() as usize
            }

            pub fn first(&self) -> Option<$idx> {
                if self.any() {
                    Some($idx::new(self.0.trailing_zeros() as usize).unwrap())
                } else {
                    None
                }
            }
        }

        impl core::ops::Index<$idx> for $t {
            type Output = bool;
            fn index(&self, index: $idx) -> &Self::Output {
                if self.select_unchecked(index.value()) {
                    &true
                } else {
                    &false
                }
            }
        }

        impl core::ops::Add<$idx> for $t {
            type Output = Self;
            fn add(self, index: $idx) -> Self::Output {
                self | Self::only(index)
            }
        }

        impl core::ops::AddAssign<$idx> for $t {
            fn add_assign(&mut self, rhs: $idx) {
                *self |= Self::only(rhs);
            }
        }

        impl core::ops::Sub<$idx> for $t {
            type Output = Self;
            fn sub(self, index: $idx) -> Self::Output {
                self & Self::all_but(index)
            }
        }

        impl core::ops::SubAssign<$idx> for $t {
            fn sub_assign(&mut self, rhs: $idx) {
                *self &= Self::all_but(rhs);
            }
        }

        impl core::ops::BitOr for $t {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl core::ops::BitOrAssign for $t {
            fn bitor_assign(&mut self, other: Self) {
                self.0 |= other.0
            }
        }

        impl core::ops::BitAnd for $t {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }

        impl core::ops::BitAndAssign for $t {
            fn bitand_assign(&mut self, other: Self) {
                self.0 &= other.0
            }
        }

        impl core::ops::Not for $t {
            type Output = Self;
            fn not(self) -> Self::Output {
                Self::new(!self.0)
            }
        }
    };
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct DigitFlags(u16);
sudoku_flags!(DigitFlags DigitIndex);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct RowFlags(u16);
sudoku_flags!(RowFlags RowIndex);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct ColumnFlags(u16);
sudoku_flags!(ColumnFlags ColumnIndex);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct BlockFlags(u16);
sudoku_flags!(BlockFlags BlockIndex);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct SudokuFlags(u16);
sudoku_flags!(SudokuFlags SudokuIndex);

