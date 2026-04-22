use std::io::{BufRead, BufReader, Read, Seek}; // take method
use std::path::PathBuf;

use xml::reader::{EventReader, XmlEvent};

use crate::board::{SudokuBoard, SudokuStringDecoding};
use crate::error::SudokuError;

fn load_xml_games(r: BufReader<impl Read>) -> Result<Vec<SudokuBoard>, xml::reader::Error> {
    let parser = EventReader::new(r.take(10_000_000));

    let mut games = Vec::new();
    for e in parser {
        if let XmlEvent::StartElement {name, attributes, .. } = e? 
            && name.local_name == "game" 
            && let Some(v) = attributes.iter().find(|p| p.name.local_name == "data")
        {
            match SudokuBoard::decode_sudoku_string(&v.value) {
                Ok(board) => games.push(board),
                Err(e) => log::error!("Failed to load game `{}`: {}", &v.value, e),
            }
        }
    }
    Ok(games)
}

fn load_sdm_games(r: BufReader<impl Read>) -> Result<Vec<SudokuBoard>, SudokuError> {
    let mut games = Vec::new();
    for line in r.lines() {
        let line = line?;
        let game = SudokuBoard::decode_sudoku_string(line.trim())?;
        games.push(game);
    }
    Ok(games)
}

pub fn load_games(path: &PathBuf) -> Vec<SudokuBoard> {
    std::fs::File::open(path).map(|mut reader|{
        if let Ok(games) = load_xml_games(BufReader::new(&reader)) {
            games
        } else {
            reader.rewind().unwrap();
            load_sdm_games(BufReader::new(reader)).unwrap_or_default()
        }
    }).unwrap_or_default()
}