use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::{Frame};
use ratatui::layout::Rect;

use crate::board::{SudokuBoard, SudokuSubCellIndex};

#[derive(Clone, Copy, Debug)]
pub enum Highlight {
    Digit(SudokuSubCellIndex),
    Row(u8),
    Column(u8),
    Block(u8),
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
    // "╔═══╤═══╤═══╦═══╤═══╤═══╦═══╤═══╤═══╗"
    // "║123|123|123║123|123|123║123|123|123║"
    // "║456|456|456║456|456|456║456|456|456║"
    // "║789|789|789║789|789|789║789|789|789║"
    // "╟───┼───┼───╫───┼───┼───╫───┼───┼───╢"
    // "║123|123|123║123|123|123║123|123|123║"
    // "║456|456|456║456|456|456║456|456|456║"
    // "║789|789|789║789|789|789║789|789|789║"
    // "╟───┼───┼───╫───┼───┼───╫───┼───┼───╢"
    // "║123|123|123║123|123|123║123|123|123║"
    // "║456|456|456║456|456|456║456|456|456║"
    // "║789|789|789║789|789|789║789|789|789║"
    // "╠═══╪═══╪═══╬═══╪═══╪═══╬═══╪═══╪═══╣"
    // "║123|123|123║123|123|123║123|123|123║"
    // "║456|456|456║456|456|456║456|456|456║"
    // "║789|789|789║789|789|789║789|789|789║"
    // "╟───┼───┼───╫───┼───┼───╫───┼───┼───╢"
    // "║123|123|123║123|123|123║123|123|123║"
    // "║456|456|456║456|456|456║456|456|456║"
    // "║789|789|789║789|789|789║789|789|789║"
    // "╟───┼───┼───╫───┼───┼───╫───┼───┼───╢"
    // "║123|123|123║123|123|123║123|123|123║"
    // "║456|456|456║456|456|456║456|456|456║"
    // "║789|789|789║789|789|789║789|789|789║"
    // "╠═══╪═══╪═══╬═══╪═══╪═══╬═══╪═══╪═══╣"
    // "║123|123|123║123|123|123║123|123|123║"
    // "║456|456|456║456|456|456║456|456|456║"
    // "║789|789|789║789|789|789║789|789|789║"
    // "╟───┼───┼───╫───┼───┼───╫───┼───┼───╢"
    // "║123|123|123║123|123|123║123|123|123║"
    // "║456|456|456║456|456|456║456|456|456║"
    // "║789|789|789║789|789|789║789|789|789║"
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

    lines.push(Line::from("╔═══════╤═══════╤═══════╦═══════╤═══════╤═══════╦═══════╤═══════╤═══════╗"));
    for r in 0..9 {
        let cells = board.row(r);

        for r_inner in 0..3 {
            let mut line = Vec::<Span>::new();
            line.push(Span::from("║"));
            for (c, cell) in cells.iter().enumerate() {

                for d in r_inner*3..r_inner*3+3 {
                    let d = d as usize;
                    let idx = (r, c, d as u8);

                    line.push(Span::from(" "));
                    if striked.contains(&idx) {
                        let ch = char::from_digit(d as u32 + 1, 10).unwrap_or('�');
                        let ch =  Span::styled(String::from(ch), Style::default().red().crossed_out());
                        line.push(ch);
                    }
                    else if circled.contains(&idx) {
                        let ch = char::from_u32(('①' as u32) + (d as u32)).unwrap_or('�');
                        let ch = Span::styled(String::from(ch), Style::default().blue());
                        line.push(ch);
                    }
                    else if cell[d] {
                        let ch = char::from_digit(d as u32 + 1, 10).unwrap_or('�').to_string();
                        line.push(Span::from(ch));
                    }
                    else { line.push(Span::from(" ")); }
                }

                let col_sep = 
                    if c % 3 == 2 { " ║" }
                    else { " │" };
                line.push(Span::from(col_sep));
            }
            lines.push(Line::from(line));
        }

        let row_sep =
            if r == 8 { "╚═══════╧═══════╧═══════╩═══════╧═══════╧═══════╩═══════╧═══════╧═══════╝" }
            else if r % 3 == 2 { "╠═══════╪═══════╪═══════╬═══════╪═══════╪═══════╬═══════╪═══════╪═══════╣" }
            else { "╟───────┼───────┼───────╫───────┼───────┼───────╫───────┼───────┼───────╢" };
        lines.push(Line::from(row_sep));
    }

    frame.render_widget(Text::from(lines), area);
}
