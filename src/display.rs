use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Block};
use ratatui::{Frame};
use ratatui::layout::{Offset, Rect, Size};
use tui_big_text::{BigText, PixelSize};

use crate::board::{SudokuBoard};
use crate::index::{CellIndex, DigitIndex, HouseIndex, HouseIndexer, SudokuSubCellIndex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

fn render_cell(
    board: &SudokuBoard,
    cell_idx: CellIndex,
    frame: &mut Frame,
    area: Rect,
    circled: &[SudokuSubCellIndex],
    striked: &[SudokuSubCellIndex],
)
{    
    if let Some(digit) = board[cell_idx].digit_value() 
        && striked.iter().all(|(c, _)| *c != cell_idx)
    {
        let style = if circled.contains(&(cell_idx, digit)) {
            Style::default().fg(Color::Blue)
        } else {
            Style::default()
        };

        let big_text = BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .style(style)
            .centered()
            .lines(vec![
                Line::from(String::from(char::from(digit)))
            ])
            .build();
        frame.render_widget(big_text, area.offset(Offset::new(1, 0)));
    } else {
        let mut digits: Vec<_> = DigitIndex::domain().iter().map(|d| {
            if striked.contains(&(cell_idx, *d)) {
                Span::styled(String::from(char::from(*d)), Style::default().red().crossed_out())
            } else if circled.contains(&(cell_idx, *d)) {
                let ch = char::from_u32(('①' as u32) + (**d as u32)).unwrap_or('�');
                Span::styled(String::from(ch), Style::default().blue())
            } else if board[cell_idx][**d] {
                Span::from(String::from(char::from(*d)))
            } else {
                Span::from(" ")
            }
        }).rev().collect();
        frame.render_widget(Paragraph::new(vec![
            Line::from(vec![digits.pop().unwrap(), Span::from(" "), digits.pop().unwrap(), Span::from(" "), digits.pop().unwrap()]),
            Line::from(vec![digits.pop().unwrap(), Span::from(" "), digits.pop().unwrap(), Span::from(" "), digits.pop().unwrap()]),
            Line::from(vec![digits.pop().unwrap(), Span::from(" "), digits.pop().unwrap(), Span::from(" "), digits.pop().unwrap()]),
        ]), area.offset(Offset::new(1, 0)).intersection(area));
    }
}

fn render_sudoku_frame(
    frame: &mut Frame,
    area: Rect,
)
{
    let mut lines = Vec::<Line>::new();

    let thin_line_style = Style::default().fg(Color::Rgb(120, 120, 120));
    let row_sep = |r: usize| {
            if r == 8 { Line::from("╚═══════════════════════╩═══════════════════════╩═══════════════════════╝") }
            else if r % 3 == 2 { Line::from("╠═══════════════════════╬═══════════════════════╬═══════════════════════╣") }
            else { 
                Line::from(vec![
                    Span::from("║"), Span::styled("───────┼───────┼───────", thin_line_style),
                    Span::from("║"), Span::styled("───────┼───────┼───────", thin_line_style),
                    Span::from("║"), Span::styled("───────┼───────┼───────", thin_line_style),
                    Span::from("║"),
                ])
            }
        };

    let empty_line = Line::from(vec![
            Span::from("║"),
            Span::from("       "), Span::styled("│", thin_line_style),
            Span::from("       "), Span::styled("│", thin_line_style),
            Span::from("       "), Span::from("║"),
            Span::from("       "), Span::styled("│", thin_line_style),
            Span::from("       "), Span::styled("│", thin_line_style),
            Span::from("       "), Span::from("║"),
            Span::from("       "), Span::styled("│", thin_line_style),
            Span::from("       "), Span::styled("│", thin_line_style),
            Span::from("       "), Span::from("║"),
        ]);

    lines.push(Line::from("╔═══════════════════════╦═══════════════════════╦═══════════════════════╗"));
    for r in 0..9 {
        lines.push(empty_line.clone());
        lines.push(empty_line.clone());
        lines.push(empty_line.clone());
        lines.push(row_sep(r));
    }

    // render skeleton
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_sudoku_highlights(
    frame: &mut Frame,
    highlights: &[Highlight],
    area: Rect,
)
{
    let highlight_style = Style::default().bg(Color::Rgb(40, 40, 40));
    for h in highlights.iter() {
        if let Highlight::House(house) = h {
            let highlight_area = match house {
                HouseIndex::Block(b) => {
                    let topleft = b.index(0);
                    area.resize(Size::new(7*3 + 2, 3*3 + 2))
                        .offset(Offset::new(1, 1))
                        .offset(Offset::new(8 * (*topleft.column() as i32), 4 * (*topleft.row() as i32)))
                        .intersection(area)
                },
                HouseIndex::Row(r) => {
                    area.resize(Size::new(7*9 + 8, 3))
                        .offset(Offset::new(1, 1))
                        .offset(Offset::new(0, 4 * (**r as i32)))
                        .intersection(area)
                },
                HouseIndex::Column(c) => {
                    area.resize(Size::new(7, 3*9 + 8))
                        .offset(Offset::new(1, 1))
                        .offset(Offset::new(8 * (**c as i32), 0))
                        .intersection(area)
                },
            };
            frame.render_widget(Block::default().style(highlight_style), highlight_area);
        }
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

    render_sudoku_frame(frame, area);
    render_sudoku_highlights(frame, highlights, area);

    let circled: Vec<_> = highlights.iter().filter_map(|h| {
        if let Highlight::Digit(d) = h {
            Some(d)
        } else {
            None
        }
    }).cloned().collect();
    for cell_idx in 0..81 {
        let cell_idx = CellIndex::from(cell_idx);

        let cell_area = area.resize(Size::new(7, 3))
            .offset(Offset::new(1, 1))
            .offset(Offset::new(8 * (*cell_idx.column() as i32), 4 * (*cell_idx.row() as i32)))
            .intersection(area)
            ;
        render_cell(board, cell_idx, frame, cell_area, &circled, striked);
    }
}
