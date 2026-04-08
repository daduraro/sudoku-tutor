use std::io::Read; // take method

use xml::reader::{EventReader, XmlEvent};

use crate::board::{SudokuBoard, SudokuBoardTrait};

pub fn load_games(r: Box<dyn std::io::Read>) -> Vec<SudokuBoard> {
    let parser = EventReader::new(r.take(10_000_000));

    let mut games = Vec::new();
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {name, attributes, .. }) if name.local_name == "game" => {
                if let Some(v) = attributes.iter().find(|p| p.name.local_name == "data") {
                    match SudokuBoard::decode_board(&v.value) {
                        Ok(board) => games.push(board),
                        Err(e) => log::error!("Failed to load game `{}`: {}", &v.value, e),
                    }
                }
            }
            Err(e) => log::error!("Failed to parse XML content: {}", e),
            _ => {}
        }
    }

    games
}