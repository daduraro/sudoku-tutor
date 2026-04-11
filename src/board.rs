use ndarray::{Array2, ArrayView, ArrayView2, ArrayViewMut2, Axis, s};
use bitvec::{bitarr, array::BitArray, order::Lsb0};

use crate::error::SudokuError;

pub type SudokuSubCellIndex = (usize, usize, u8);

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub type SudokuCell = BitArray<[u16; 1]>;

pub trait SudokuCellTrait where 
    Self: std::marker::Sized
{
    fn empty_cell() -> Self;
    fn digit(d: u8) -> Self;
    fn is_digit(&self) -> bool;
    fn digit_value(&self) -> Option<u8>;
    fn is_valid(&self) -> bool;
    fn encode_sudoku_cell(&self) -> char;
    fn decode_sudoku_cell(c: char) -> Result<Self, SudokuError>;
}

impl SudokuCellTrait for SudokuCell {
    fn empty_cell() -> Self {
        bitarr![u16, Lsb0; 1, 1, 1, 1, 1, 1, 1, 1, 1]
    }

    fn digit(d: u8) -> Self {
        assert!(0 < d && d < 10);
        let mut v = SudokuCell::ZERO;
        v.set((d-1) as usize, true);
        v
    }

    fn is_valid(&self) -> bool {
        self.any()
    }

    fn is_digit(&self) -> bool {
        self.count_ones() == 1
    }

    fn digit_value(&self) -> Option<u8> {
        if self.is_digit() {
            self.first_one().map(|v| v as u8)
        } else {
            None
        }
    }

    fn encode_sudoku_cell(&self) -> char {
        if let Some(d) = self.digit_value() {
            char::from_digit((d + 1) as u32, 10).unwrap()
        } else {
            '0'
        }
    }

    fn decode_sudoku_cell(c: char) -> Result<Self, SudokuError> {
        if c == '.' { return Ok(Self::empty_cell()) }

        if let Some(v) = c.to_digit(10) && v < 10 {
            if v == 0 { Ok(Self::empty_cell()) } 
            else { Ok(Self::digit(v as u8)) }
        } else {
            Err(SudokuError::InvalidDigit(c))
        }
    }

}

pub trait SudokuStringEncoding {
    fn encode_sudoku_string(self) -> String;
}

pub trait SudokuStringDecoding where
    Self: std::marker::Sized
{
    fn decode_sudoku_string(game: &str) -> Result<Self, SudokuError>;
}

impl<'cell, T> SudokuStringEncoding for T
where
    T: IntoIterator<Item = &'cell SudokuCell>,
{
    fn encode_sudoku_string(self) -> String {
        self.into_iter()
            .map(SudokuCell::encode_sudoku_cell)
            .collect()
    }

}

impl<T> SudokuStringDecoding for T
where
    T: FromIterator<SudokuCell>
{
    fn decode_sudoku_string(game: &str) -> Result<Self, SudokuError> {
        game.chars()
            .map(SudokuCell::decode_sudoku_cell)
            .collect::<Result<T, SudokuError>>()
    }
}

pub type SudokuBoard = Array2<SudokuCell>;

pub trait SudokuBoardTrait 
where 
    Self: std::marker::Sized
{
    type BoardCell;
    
    fn is_valid(&self) -> bool;
    fn is_solved(&self) -> bool;

    fn encode_board(&self) -> String;
    fn decode_board(s: &str) -> Result<Self, SudokuError>;

    fn row_collapse(&self, index: usize) -> ArrayView2<'_, SudokuCell>;
    fn row_collapse_mut(&mut self, index: usize) -> ArrayViewMut2<'_, SudokuCell>;

    fn column_collapse(&self, index: usize) -> ArrayView2<'_, SudokuCell>;
    fn column_collapse_mut(&mut self, index: usize) -> ArrayViewMut2<'_, SudokuCell>;

    fn block(&self, index: usize) -> ArrayView2<'_, Self::BoardCell>;
    fn block_mut(&mut self, index: usize) -> ArrayViewMut2<'_, Self::BoardCell>;

    fn block_index(row: usize, col: usize) -> usize {
        (row / 3) * 3 + (col / 3)
    }

    fn index_from_block(block: usize, row: usize, col: usize) -> [usize; 2] {
        let row = (block / 3) * 3 + row;
        let col = (block % 3) * 3 + col;
        [row, col]
    }
}

impl SudokuBoardTrait for SudokuBoard {
    type BoardCell = SudokuCell;
    fn is_valid(&self) -> bool {
        self.iter().all(SudokuCell::is_valid)
    }

    fn is_solved(&self) -> bool {
        fn is_complete<D: ndarray::Dimension>(region: ArrayView<SudokuCell, D>) -> bool {
            region.into_iter().fold(SudokuCell::ZERO, |acc: SudokuCell, c: &SudokuCell| {
                let mut acc = acc;
                if let Some(d) = c.digit_value() {
                    acc.set(d as usize, true)
                }
                acc
            }).count_ones() == 9
        }
        
        (0..9).all(|i| 
            is_complete(self.row(i)) &&
            is_complete(self.column(i)) &&
            is_complete(self.block(i))
        )
    }

    fn encode_board(&self) -> String {
        self.encode_sudoku_string()
    }

    fn decode_board(s: &str) -> Result<Self, SudokuError> {
        let state = Vec::<SudokuCell>::decode_sudoku_string(s)?;
        let ndigits = state.len();

        Array2::from_shape_vec((9, 9), state)
            .map_err(|_| SudokuError::InvalidBoardSize(ndigits))
    }

    fn row_collapse(&self, index: usize) -> ArrayView2<'_, SudokuCell> {
        let mut v = self.view();
        v.collapse_axis(Axis(0), index);
        v
    }

    fn row_collapse_mut(&mut self, index: usize) -> ArrayViewMut2<'_, SudokuCell> {
        let mut v = self.view_mut();
        v.collapse_axis(Axis(0), index);
        v
    }

    fn column_collapse(&self, index: usize) -> ArrayView2<'_, SudokuCell> {
        let mut v = self.view();
        v.collapse_axis(Axis(1), index);
        v
    }

    fn column_collapse_mut(&mut self, index: usize) -> ArrayViewMut2<'_, SudokuCell> {
        let mut v = self.view_mut();
        v.collapse_axis(Axis(1), index);
        v
    }

    fn block(&self, index: usize) -> ArrayView2<'_, SudokuCell> {
        assert!(index < 9, "Block index must be between 0 and 8");
        let row_start = (index / 3) * 3;
        let col_start = (index % 3) * 3;
        self.slice(s![row_start..row_start+3, col_start..col_start+3])            
    }

    fn block_mut(&mut self, index: usize) -> ArrayViewMut2<'_, SudokuCell> {
        assert!(index < 9, "Block index must be between 0 and 8");
        let row_start = (index / 3) * 3;
        let col_start = (index % 3) * 3;
        self.slice_mut(s![row_start..row_start+3, col_start..col_start+3])
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding() -> Result<(), SudokuError> {
        let g = SudokuBoard::decode_board("501740008000000050098600400040961580050000010016854070005006730070000000900072805")?;
        assert_eq!(g, SudokuBoard::decode_board(&g.encode_board())?);
        Ok(())
    }

    #[test]
    fn test_read() {
        // +-------+-------+-------+
        // | 5 0 1 | 7 4 0 | 0 0 8 |
        // | 0 0 0 | 0 0 0 | 0 5 0 |
        // | 0 9 8 | 6 0 0 | 4 0 0 |
        // +-------+-------+-------+
        // | 0 4 0 | 9 6 1 | 5 8 0 |
        // | 0 5 0 | 0 0 0 | 0 1 0 |
        // | 0 1 6 | 8 5 4 | 0 7 0 |
        // +-------+-------+-------+
        // | 0 0 5 | 0 0 6 | 7 3 0 |
        // | 0 7 0 | 0 0 0 | 0 0 0 |
        // | 9 0 0 | 0 7 2 | 8 0 5 |
        // +-------+-------+-------+
        let mut g = SudokuBoard::decode_board("501740008000000050098600400040961580050000010016854070005006730070000000900072805").unwrap();
        assert_eq!(g.block(5).encode_sudoku_string(), "580010070");
        assert_eq!(g.column(8).encode_sudoku_string(), "800000005");
        assert_eq!(g.column_collapse(8).encode_sudoku_string(), "800000005");
        assert_eq!(g.column_collapse_mut(8).encode_sudoku_string(), "800000005");
        assert_eq!(g.row(3).encode_sudoku_string(), "040961580");
        assert_eq!(g.row_collapse(3).encode_sudoku_string(), "040961580");
        assert_eq!(g.row_collapse_mut(3).encode_sudoku_string(), "040961580");

        assert_eq!(g.row_collapse(2).indexed_iter().map(|((_,j), _)| j).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(g.column_collapse(2).indexed_iter().map(|((i,_), _)| i).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn modify_block() {
        let mut g = SudokuBoard::decode_board("501740008000000050098600400040961580050000010016854070005006730070000000900072805").unwrap();
        for c in g.block_mut(0) {
            *c = SudokuCell::ZERO;
        }
        assert!(!g.is_valid());
    }

    #[test]
    #[should_panic]
    fn test_read_oob() {
        let g = SudokuBoard::decode_board("501740008000000050098600400040961580050000010016854070005006730070000000900072805").unwrap();
        g.block(9);
    }

}