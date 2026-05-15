pub mod cell_value;
pub mod container;
pub mod encoding;

pub use cell_value::SudokuCell;
pub use container::{SudokuBoard, SudokuBoardIter};
pub use encoding::{SudokuStringDecoding, SudokuStringEncoding};


// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::index::{BlockIndex, ColumnIndex, RowIndex, DigitIndex};
//     use crate::error::SudokuError;

//     #[test]
//     fn test_encoding() -> Result<(), SudokuError> {
//         let g = SudokuBoard::decode_sudoku_string("501740008000000050098600400040961580050000010016854070005006730070000000900072805")?;
//         assert_eq!(g, SudokuBoard::decode_sudoku_string(&g.encode_sudoku_string())?);
//         Ok(())
//     }

//     #[test]
//     fn test_read() {
//         // +-------+-------+-------+
//         // | 5 0 1 | 7 4 0 | 0 0 8 |
//         // | 0 0 0 | 0 0 0 | 0 5 0 |
//         // | 0 9 8 | 6 0 0 | 4 0 0 |
//         // +-------+-------+-------+
//         // | 0 4 0 | 9 6 1 | 5 8 0 |
//         // | 0 5 0 | 0 0 0 | 0 1 0 |
//         // | 0 1 6 | 8 5 4 | 0 7 0 |
//         // +-------+-------+-------+
//         // | 0 0 5 | 0 0 6 | 7 3 0 |
//         // | 0 7 0 | 0 0 0 | 0 0 0 |
//         // | 9 0 0 | 0 7 2 | 8 0 5 |
//         // +-------+-------+-------+
//         let g = SudokuBoard::decode_sudoku_string("501740008000000050098600400040961580050000010016854070005006730070000000900072805").unwrap();
//         assert_eq!(g.region(&BlockIndex::new(1, 2)).encode_sudoku_string(), "580010070");
//         assert_eq!(g.region(&ColumnIndex::new(8)).encode_sudoku_string(), "800000005");
//         assert_eq!(g.region(&RowIndex::new(3)).encode_sudoku_string(), "040961580");
//     }

//     #[test]
//     fn modify_block() {
//         let mut g = SudokuBoard::decode_sudoku_string("501740008000000050098600400040961580050000010016854070005006730070000000900072805").unwrap();
//         let idx = ColumnIndex::new(0);
//         for c in g.region_mut(&idx) {
//             *c = SudokuCell::digit(DigitIndex::new(0).unwrap());
//         }
//         assert_eq!(g.region(&idx).encode_sudoku_string(), "111111111");
//     }
// }