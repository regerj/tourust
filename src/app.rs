use std::{env, fs, io, path::PathBuf, thread::sleep, time::Duration};

use futures::future::BoxFuture;
use fuzzy_matcher::clangd::fuzzy_match;
use log::debug;
use priority_queue::PriorityQueue;
use ratatui::{
    Terminal,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
    widgets::ListState,
};
use rust_search::SearchBuilder;
use syn::{Item, spanned::Spanned};

use crate::{error::Result, tui};

#[derive(Hash, Default, Eq, PartialEq, Clone, Debug)]
pub struct Ref {
    pub line: usize,
    pub column: usize,
    pub file: PathBuf,
    pub sig: String,
}

impl Into<Ref> for (Item, PathBuf) {
    fn into(self) -> Ref {
        let sig = self.0.display();
        match self.0 {
            Item::Fn(item) => Ref {
                line: item.span().start().line,
                column: item.span().start().column,
                file: self.1,
                sig,
            },
            Item::Mod(item) => Ref {
                line: item.span().start().line,
                column: item.span().start().column,
                file: self.1,
                sig,
            },
            Item::Enum(item) => Ref {
                line: item.span().start().line,
                column: item.span().start().column,
                file: self.1,
                sig,
            },
            Item::Trait(item) => Ref {
                line: item.span().start().line,
                column: item.span().start().column,
                file: self.1,
                sig,
            },
            Item::Struct(item) => Ref {
                line: item.span().start().line,
                column: item.span().start().column,
                file: self.1,
                sig,
            },
            _ => Ref::default(),
        }
    }
}

pub enum Screen {
    Main,
}

pub trait IsRelevant {
    fn is_relevant(&self) -> bool;
}

impl IsRelevant for Item {
    fn is_relevant(&self) -> bool {
        match self {
            Item::Use(_) => false,
            Item::Impl(_) => false,
            Item::Type(_) => false,
            Item::Macro(_) => false,
            Item::TraitAlias(_) => false,
            Item::Verbatim(_) => false,
            Item::ForeignMod(_) => false,
            Item::Static(_) => false,
            Item::Const(_) => false,
            Item::Union(_) => false,
            Item::ExternCrate(_) => false,
            _ => true,
        }
    }
}

pub trait ItemDisplay {
    fn display(&self) -> String;
}

impl ItemDisplay for Item {
    fn display(&self) -> String {
        match self {
            Item::Fn(item) => {
                format!(
                    "{}{}",
                    item.vis
                        .span()
                        .source_text()
                        .map_or(String::new(), |e| e + " "),
                    item.sig
                        .span()
                        .source_text()
                        .unwrap_or("MISSING SOURCE TEXT".to_string())
                )
            }
            Item::Mod(item) => {
                format!(
                    "{}mod {}",
                    item.vis
                        .span()
                        .source_text()
                        .map_or(String::new(), |e| e + " "),
                    item.ident
                )
            }
            Item::Enum(item) => {
                format!(
                    "{}enum {}",
                    item.vis
                        .span()
                        .source_text()
                        .map_or(String::new(), |e| e + " "),
                    item.ident
                )
            }
            Item::Trait(item) => {
                format!(
                    "{}trait {}",
                    item.vis
                        .span()
                        .source_text()
                        .map_or(String::new(), |e| e + " "),
                    item.ident
                )
            }
            Item::Struct(item) => {
                format!(
                    "{}struct {}",
                    item.vis
                        .span()
                        .source_text()
                        .map_or(String::new(), |e| e + " "),
                    item.ident
                )
            }
            _ => String::new(),
        }
    }
}

pub trait SelectCallback {
    fn call(&self, selection: Ref) -> BoxFuture<'static, Result<()>>;
}

impl<T, F> SelectCallback for T
where
    T: Fn(Ref) -> F,
    F: Future<Output = Result<()>> + 'static + Send,
{
    fn call(&self, selection: Ref) -> BoxFuture<'static, Result<()>> {
        Box::pin(self(selection))
    }
}

pub struct App {
    pub current_screen: Screen,
    pub refs: Vec<Ref>,
    pub search_results: PriorityQueue<Ref, i64>,
    pub input: String,
    pub search_result_state: ListState,
    pub select_callback: Option<Box<dyn SelectCallback>>,
}

impl App {
    pub fn new() -> Result<Self> {
        // Parse all of our rust files
        let files: Vec<PathBuf> = SearchBuilder::default()
            .location(env::current_dir().unwrap())
            .ext("rs")
            .hidden()
            .build()
            .map(|e| PathBuf::from(e))
            .collect();

        let mut refs = Vec::<Ref>::new();
        for file in files {
            let src = fs::read_to_string(&file)?;
            let syntax = syn::parse_file(&src)?;
            // Append refs with each item in the file that is relevant
            let rel_items: Vec<_> = syntax.items.iter().filter(|e| e.is_relevant()).collect();
            for item in rel_items {
                refs.push((item.to_owned(), file.to_owned()).into());
            }
        }

        let search_results = refs.iter().map(|elem| (elem.to_owned(), 0)).collect();

        debug!("refs: {:#?}", refs);

        Ok(Self {
            current_screen: Screen::Main,
            refs,
            search_results,
            input: String::new(),
            search_result_state: ListState::default(),
            select_callback: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stderr = io::stderr();
        execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stderr);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| tui::ui(f, self))?;
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Char(ch) => {
                        // Every time the user types a character, add it to the input, drain the refs,
                        // map them to assign new prio, then collect and reassign them to refs.
                        self.input.push(ch);
                        self.search_results = self
                            .refs
                            .iter()
                            .filter_map(|elem| {
                                fuzzy_match(&elem.sig, &self.input)
                                    .map(|prio| (elem.to_owned(), prio))
                            })
                            .collect()
                    }
                    KeyCode::Up => self.search_result_state.select_previous(),
                    KeyCode::Down => self.search_result_state.select_next(),
                    KeyCode::Backspace => {
                        self.input.pop();
                        self.search_results = self
                            .refs
                            .iter()
                            .map(|elem| {
                                (
                                    elem.to_owned(),
                                    fuzzy_match(&elem.sig, &self.input).unwrap_or_default(),
                                )
                            })
                            .collect();
                    }
                    KeyCode::Enter => {
                        // Continue if nothing is selected
                        if let Some(r) = self.get_selected_ref() {
                            if let Some(callback) = &self.select_callback {
                                callback.call(r.clone()).await?;
                            }
                            break;
                        } else {
                            continue;
                        }
                    }
                    _ => {}
                }
            }
            sleep(Duration::from_millis(25));
        }
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    pub fn get_selected_ref(&self) -> Option<&Ref> {
        let i = self.search_result_state.selected()?;
        debug!("i: {i}");
        debug!("search results: {:#?}", self.search_results);
        self.search_results.iter().rev().nth(i).map(|x| x.0)
    }
}
