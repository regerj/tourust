use std::path::Path;

use ansi_to_tui::IntoText;
use bat::{
    PrettyPrinter,
    line_range::{LineRange, LineRanges},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::{app::App, error::Result};

fn highlight_syntax(file: &Path, line: usize) -> Result<String> {
    let mut x = String::new();
    PrettyPrinter::new()
        .input_file(file)
        .header(true)
        .line_numbers(true)
        .grid(true)
        .highlight(line)
        .line_ranges(LineRanges::from(vec![LineRange::new(line, usize::MAX)]))
        .print_with_writer(Some(&mut x))?;

    Ok(x)
}

pub fn ui(frame: &mut Frame, app: &mut App) {
    // Break up the frame into chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(frame.area());

    // Our subchunks is the search results and code preview
    let subchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Min(1)])
        .split(chunks[1]);

    // Create the top search block
    let search_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    let search = Paragraph::new(app.input.clone()).block(search_block);
    frame.render_widget(search, chunks[0]);

    // Create the search results
    let mut list_items: Vec<ListItem> = Vec::new();
    for item in app.search_results.clone().into_sorted_iter() {
        list_items.push(ListItem::from(item.0.sig.to_owned()));
    }
    let search_results_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    let search_results_list = List::new(list_items)
        .block(search_results_block)
        .highlight_style(Style::default().bg(Color::LightCyan));
    frame.render_stateful_widget(
        search_results_list,
        subchunks[0],
        &mut app.search_result_state,
    );

    // Create the code render
    frame.render_widget(Clear, subchunks[1]);
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    if let Some(selected_ref) = app.get_selected_ref() {
        let highlighted_text = highlight_syntax(&selected_ref.file, selected_ref.line)
            .expect("Failed to highlight file")
            .into_text()
            .expect("Failed to translate from ANSI to TUI");
        let file_preview = Paragraph::new(highlighted_text).block(preview_block);
        frame.render_widget(file_preview, subchunks[1]);
    }
}
