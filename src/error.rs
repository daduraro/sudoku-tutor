use thiserror::Error;
#[derive(Error, Debug)]
pub enum SudokuError {
    #[error("character `{0}` is not a valid digit")]
    InvalidDigit(char),
    #[error("invalid board size, expected 81 digits, found `{0}`")]
    InvalidBoardSize(usize),
    #[error("sudoku is unsolvable")]
    UnsolvableSudoku,
    #[error("io error")]
    IOError(#[from] std::io::Error),
    #[error("xml error")]
    XmlError(#[from] xml::reader::Error),
}
