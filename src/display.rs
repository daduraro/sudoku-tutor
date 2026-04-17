use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::{Frame};
use ratatui::layout::Rect;

use crate::board::{SudokuBoard};
use crate::index::{CellIndex, DigitIndex, HouseIndex, HouseIndexer, RowIndex, SudokuSubCellIndex};

#[derive(Clone, Copy, Debug)]
pub enum Highlight {
    Digit(SudokuSubCellIndex),
    House(HouseIndex),
}

impl<Idx> core::convert::From<Idx> for Highlight 
where Idx: core::convert::Into<HouseIndex> {
    fn from(value: Idx) -> Self {
        Highlight::House(value.into())
    }
}

impl core::convert::From<SudokuSubCellIndex> for Highlight {
    fn from(value: SudokuSubCellIndex) -> Self {
        Highlight::Digit(value)
    }
}


pub fn render_sudoku_board(
    frame: &mut Frame,
    area: Rect,
    board: &SudokuBoard,
    highlights: &[Highlight],
    striked: &[SudokuSubCellIndex],
)
{
    // The string of an empty sudoku board should be: 
    // "╔═══════╤═══════╤═══════╦═══════╤═══════╤═══════╦═══════╤═══════╤═══════╗"
    // "║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║"
    // "║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║"
    // "║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║"
    // "╟───────┼───────┼───────╫───────┼───────┼───────╫───────┼───────┼───────╢"
    // "║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║"
    // "║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║"
    // "║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║"
    // "╟───────┼───────┼───────╫───────┼───────┼───────╫───────┼───────┼───────╢"
    // "║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║"
    // "║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║"
    // "║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║"
    // "╠═══════╪═══════╪═══════╬═══════╪═══════╪═══════╬═══════╪═══════╪═══════╣"
    // "║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║"
    // "║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║"
    // "║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║"
    // "╟───────┼───────┼───────╫───────┼───────┼───────╫───────┼───────┼───────╢"
    // "║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║"
    // "║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║"
    // "║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║"
    // "╟───────┼───────┼───────╫───────┼───────┼───────╫───────┼───────┼───────╢"
    // "║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║"
    // "║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║"
    // "║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║"
    // "╠═══════╪═══════╪═══════╬═══════╪═══════╪═══════╬═══════╪═══════╪═══════╣"
    // "║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║"
    // "║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║"
    // "║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║"
    // "╟───────┼───────┼───────╫───────┼───────┼───────╫───────┼───────┼───────╢"
    // "║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║"
    // "║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║"
    // "║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║"
    // "╟───────┼───────┼───────╫───────┼───────┼───────╫───────┼───────┼───────╢"
    // "║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║ 1 2 3 | 1 2 3 | 1 2 3 ║"
    // "║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║ 4 5 6 | 4 5 6 | 4 5 6 ║"
    // "║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║ 7 8 9 | 7 8 9 | 7 8 9 ║"
    // "╚═══════╧═══════╧═══════╩═══════╧═══════╧═══════╩═══════╧═══════╧═══════╝">

    let mut lines = Vec::<Line>::new();

    let circled: Vec<_> = highlights.iter()
        .filter_map(|h| if let Highlight::Digit(idx) = h { Some(idx) } else { None })
        .cloned()
        .collect();

    lines.push(Line::from("╔═══════════════════════╦═══════════════════════╦═══════════════════════╗"));

    let default_style = Style::default();
    let highlight_style = Style::default().bg(Color::Rgb(40, 40, 40));
    let should_highlight = |cell_idx: CellIndex| -> bool {
        highlights.iter().any(|h|  {
            if let Highlight::House(idx) = h {
                idx.contains(cell_idx)
            } else {
                false
            }
        })
    };

    for &r in RowIndex::domain() {
        for r_inner in 0..3 {
            let mut line = Vec::<Span>::new();
            line.push(Span::from("║"));
            for (cell_idx, cell) in board.indexed_house(r) {

                let style = if should_highlight(cell_idx) { &highlight_style } else { &default_style };

                for d in r_inner*3..r_inner*3+3 {
                    let d = DigitIndex::new(d);
                    let idx = (cell_idx, d);

                    line.push(Span::styled(" ", *style));
                    if striked.contains(&idx) {
                        let ch = char::from(d);
                        let ch =  Span::styled(String::from(ch), style.patch(Style::default().red().crossed_out()));
                        line.push(ch);
                    }
                    else if circled.contains(&idx) {
                        let ch = char::from_u32(('①' as u32) + (*d as u32)).unwrap_or('�');
                        let ch = Span::styled(String::from(ch), style.patch(Style::default().blue()));
                        line.push(ch);
                    }
                    else if cell[d] {
                        let ch = char::from_digit(*d as u32 + 1, 10).unwrap_or('�').to_string();
                        line.push(Span::styled(ch, *style));
                    }
                    else { line.push(Span::styled(" ", *style)); }
                }

                line.push(Span::styled(" ", *style));

                let col_sep = 
                    if *cell_idx.column() % 3 == 2 { Span::from("║") }
                    else { Span::styled("│", Style::default().fg(Color::Rgb(120, 120, 120))) };
                line.push(col_sep);
            }
            
            lines.push(Line::from(line));
        }

        let row_sep =
            if *r == 8 { Line::from("╚═══════════════════════╩═══════════════════════╩═══════════════════════╝") }
            else if *r % 3 == 2 { Line::from("╠═══════════════════════╬═══════════════════════╬═══════════════════════╣") }
            else { Line::from(vec![
                Span::from("║"),
                Span::styled("───────┼───────┼───────", Style::default().fg(Color::Rgb(120, 120, 120))),
                Span::from("║"),
                Span::styled("───────┼───────┼───────", Style::default().fg(Color::Rgb(120, 120, 120))),
                Span::from("║"),
                Span::styled("───────┼───────┼───────", Style::default().fg(Color::Rgb(120, 120, 120))),
                Span::from("║"),
            ]) };
        lines.push(row_sep);
    }

    frame.render_widget(Text::from(lines), area);
}
