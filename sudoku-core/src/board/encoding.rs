use crate::error::SudokuError;
use super::SudokuCell;

#[allow(dead_code)]
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
            .map(char::from)
            .collect()
    }
}

impl<T> SudokuStringDecoding for T
where
    T: FromIterator<SudokuCell>
{
    fn decode_sudoku_string(game: &str) -> Result<Self, SudokuError> {
        game.chars()
            .map(SudokuCell::try_from)
            .collect::<Result<T, SudokuError>>()
    }
}