use fuzzy_matcher::clangd::fuzzy_match;
use priority_queue::PriorityQueue;
use ratatui::{
    layout::{Constraint, Direction, Layout}, style::{Color, Style}, widgets::{Block, Borders, List, ListItem, Paragraph}, Frame
};

use crate::app::App;

pub fn ui(frame: &mut Frame, app: &mut App) {
    // Break up the frame into chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(frame.area());

    // Create the top search block
    let search_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    let search = Paragraph::new(app.input.clone()).block(search_block);
    frame.render_widget(search, chunks[0]);

    // Create the search results
    let mut list_items: Vec<ListItem> = Vec::new();
    let mut search_results = PriorityQueue::new();
    for item in &app.refs {
        if let Some(weight) = fuzzy_match(item.sig.as_str(), app.input.as_str()) {
            search_results.push(item, weight);
        }
    }
    for item in search_results.into_sorted_iter() {
        list_items.push(ListItem::from(item.0.sig.to_owned()));
    }
    let search_results_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    let search_results_list = List::new(list_items).block(search_results_block).highlight_style(Style::default().bg(Color::LightCyan));
    frame.render_stateful_widget(search_results_list, chunks[1], &mut app.search_result_state);
}
