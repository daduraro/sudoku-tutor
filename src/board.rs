use ndarray::{Array2};
use bitvec::{bitarr, array::BitArray, order::Lsb0};

use crate::error::SudokuError;

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
type SudokuCell = BitArray<[u16; 1]>;

pub trait SudokuCellTrait where 
    Self: std::marker::Sized
{
    fn empty_cell() -> Self;
    fn digit(d: u8) -> Self;
    fn is_valid(&self) -> bool;
    fn encode_sudoku_cell(&self) -> char;
    fn decode_sudoku_cell(c: char) -> Result<Self, SudokuError>;
}

impl SudokuCellTrait for SudokuCell {
    fn empty_cell() -> Self {
        bitarr![u16, Lsb0; 1; 9]
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

    fn encode_sudoku_cell(&self) -> char {
        if self.count_ones() == 1 {
            char::from_digit((self.first_one().unwrap() + 1) as u32, 10).unwrap()
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

pub trait SudokuStringDecoding where
    Self: std::marker::Sized
{
    fn decode_sudoku_string(game: &str) -> Result<Self, SudokuError>;
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SudokuBoard {
    state: Array2<SudokuCell>,
}

impl SudokuBoard {
    pub fn is_valid(&self) -> bool {
        self.state.iter().all(SudokuCell::is_valid)
    }

    pub fn is_solved(&self) -> bool {
        self.state.iter().all(|c| c.count_ones() == 1)
    }

    pub fn row(&self, r: usize) -> [SudokuCell; 9] {
        core::array::from_fn(|c| self.state[[r, c]])
    }

    pub fn col(&self, c: usize) -> [SudokuCell; 9] {
        core::array::from_fn(|r| self.state[[r, c]])
    }

    pub fn block(&self, b: usize) -> [SudokuCell; 9] {
        let br = 3*(b/3);
        let bc = 3*(b%3);
        core::array::from_fn(|i| self.state[[br + i/3, bc + i%3]])
    }
}

impl SudokuStringEncoding for &SudokuBoard {
    fn encode_sudoku_string(self) -> String {
        self.state.encode_sudoku_string()
    }
}

impl SudokuStringDecoding for SudokuBoard {
    fn decode_sudoku_string(state: &str) -> Result<Self, SudokuError> {
        let state  = Vec::<SudokuCell>::decode_sudoku_string(state)?;
        let ndigits = state.len();

        match Array2::from_shape_vec((9, 9), state) {
            Ok(state) => Ok(SudokuBoard { state }),
            Err(_) => Err(SudokuError::InvalidBoardSize(ndigits)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding() -> Result<(), SudokuError> {
        let g = SudokuBoard::decode_sudoku_string("501740008000000050098600400040961580050000010016854070005006730070000000900072805")?;
        assert_eq!(g, SudokuBoard::decode_sudoku_string(&g.encode_sudoku_string())?);
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
        let g = SudokuBoard::decode_sudoku_string("501740008000000050098600400040961580050000010016854070005006730070000000900072805").unwrap();
        assert_eq!(g.block(5).encode_sudoku_string(), "580010070");
        assert_eq!(g.col(8).encode_sudoku_string(), "800000005");
        assert_eq!(g.row(3).encode_sudoku_string(), "040961580");
    }

    #[test]
    #[should_panic]
    fn test_read_oob() {
        let g = SudokuBoard::decode_sudoku_string("501740008000000050098600400040961580050000010016854070005006730070000000900072805").unwrap();
        g.block(9);
    }
}