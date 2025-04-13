use std::{
    env, fs, io,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

use futures::future::BoxFuture;
use fuzzy_matcher::clangd::fuzzy_match;
use log::debug;
use priority_queue::PriorityQueue;
use ratatui::{
    Terminal,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
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

impl From<(Item, PathBuf)> for Ref {
    fn from(value: (Item, PathBuf)) -> Self {
        let sig = value.0.display();
        match value.0 {
            Item::Fn(item) => Self {
                line: item.sig.span().start().line,
                column: item.sig.span().start().column,
                file: value.1,
                sig,
            },
            Item::Mod(item) => Self {
                line: item.ident.span().start().line,
                column: item.ident.span().start().column,
                file: value.1,
                sig,
            },
            Item::Enum(item) => Self {
                line: item.ident.span().start().line,
                column: item.ident.span().start().column,
                file: value.1,
                sig,
            },
            Item::Trait(item) => Self {
                line: item.ident.span().start().line,
                column: item.ident.span().start().column,
                file: value.1,
                sig,
            },
            Item::Struct(item) => Self {
                line: item.ident.span().start().line,
                column: item.ident.span().start().column,
                file: value.1,
                sig,
            },
            Item::Use(item) => Self {
                line: item.span().start().line,
                column: item.span().start().column,
                file: value.1,
                sig,
            },
            Item::Type(item) => Self {
                line: item.span().start().line,
                column: item.span().start().column,
                file: value.1,
                sig,
            },
            Item::Impl(item) => Self {
                line: item.self_ty.span().start().line,
                column: item.self_ty.span().start().column,
                file: value.1,
                sig,
            },
            Item::Const(item) => Self {
                line: item.span().start().line,
                column: item.span().start().column,
                file: value.1,
                sig,
            },
            Item::Macro(item) => Self {
                line: item.ident.span().start().line,
                column: item.ident.span().start().column,
                file: value.1,
                sig,
            },
            Item::Static(item) => Self {
                line: item.span().start().line,
                column: item.span().start().column,
                file: value.1,
                sig,
            },
            Item::Union(item) => Self {
                line: item.ident.span().start().line,
                column: item.ident.span().start().column,
                file: value.1,
                sig,
            },
            _ => unimplemented!(),
        }
    }
}

pub trait IsRelevant {
    fn is_relevant(&self) -> bool;
}

impl IsRelevant for Item {
    fn is_relevant(&self) -> bool {
        !matches!(
            self,
            Item::TraitAlias(_) | Item::Verbatim(_) | Item::ForeignMod(_) | Item::ExternCrate(_)
        )
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
            Item::Use(item) => item.span().source_text().unwrap_or(String::from("UNKNOWN")),
            Item::Type(item) => item.span().source_text().unwrap_or(String::from("UNKNOWN")),
            Item::Impl(item) => {
                if let Some((_, pth, _)) = &item.trait_ {
                    format!(
                        "impl {} for {}",
                        pth.segments
                            .last()
                            .span()
                            .source_text()
                            .unwrap_or("UNKNOWN".into()),
                        item.self_ty
                            .span()
                            .source_text()
                            .unwrap_or("UNKNOWN".into())
                    )
                } else {
                    format!(
                        "impl {}",
                        item.self_ty
                            .span()
                            .source_text()
                            .unwrap_or("UNKNOWN".into())
                    )
                }
            }
            Item::Const(item) => item.span().source_text().unwrap_or("UNKNOWN".into()),
            Item::Macro(item) => item.ident.span().source_text().unwrap_or("UNKNOWN".into()),
            Item::Static(item) => item.span().source_text().unwrap_or("UNKNOWN".into()),
            Item::Union(item) => {
                format!(
                    "{}union {}",
                    item.vis.span().source_text().unwrap_or("".into()),
                    item.ident.span().source_text().unwrap_or("UNKNOWN".into())
                )
            }
            _ => "IRRELEVANT".into(),
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
    pub refs: Vec<Ref>,
    pub search_results: PriorityQueue<Ref, i64>,
    pub input: String,
    pub search_result_state: ListState,
    pub select_callback: Option<Box<dyn SelectCallback>>,
}

impl App {
    pub fn new() -> Result<Self> {
        // Parse all of our rust files
        let refs = App::find_refs()?;

        let search_results = refs.iter().map(|elem| (elem.to_owned(), 0)).collect();

        debug!("refs: {:#?}", refs);

        Ok(Self {
            refs,
            search_results,
            input: String::new(),
            search_result_state: ListState::default(),
            select_callback: None,
        })
    }

    fn recursive_find_refs(item: Item, refs: &mut Vec<Ref>, file: &Path) -> Result<()> {
        // Push the item itself
        if !item.is_relevant() {
            return Ok(());
        }
        refs.push((item.clone(), file.to_owned()).into());
        match item {
            Item::Mod(md) => {
                // If the module has a body
                if let Some(content) = md.content {
                    // For every item in the module
                    for item in content.1 {
                        Self::recursive_find_refs(item, refs, file)?;
                    }
                }
            }
            // For now, ignore implement items, will require rework of ref struct
            Item::Impl(_im) => {
                //for item in im.items {
                //    match item {
                //        ImplItem::Fn(fun) => {
                //
                //        }
                //    }
                //    Self::recursive_find_refs(item, refs, file);
                //}
            }
            _ => {}
        }
        Ok(())
    }

    fn find_refs() -> Result<Vec<Ref>> {
        let files: Vec<PathBuf> = SearchBuilder::default()
            .location(env::current_dir().unwrap())
            .ext("rs")
            .hidden()
            .build()
            .map(PathBuf::from)
            .collect();

        let mut refs = Vec::<Ref>::new();
        for file in files {
            let src = fs::read_to_string(&file)?;
            let syntax = syn::parse_file(&src)?;
            // Append refs with each item in the file that is relevant
            for item in syntax.items {
                Self::recursive_find_refs(item, &mut refs, &file)?;
            }
        }

        Ok(refs)
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

                if key.modifiers == KeyModifiers::CONTROL {
                    // Allow Ctrl-j/k to move up and down selection
                    if let KeyCode::Char(ch) = key.code {
                        match ch {
                            'j' => self.search_result_state.select_next(),
                            'k' => self.search_result_state.select_previous(),
                            _ => {}
                        }
                    }
                } else if key.modifiers == KeyModifiers::SHIFT {
                    // Allow Shift+Tab to move up selection
                    if let KeyCode::BackTab = key.code {
                        self.search_result_state.select_previous();
                    }
                } else if key.modifiers == KeyModifiers::NONE {
                    // All other normal keybinds
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
                        KeyCode::BackTab => self.search_result_state.select_previous(),
                        KeyCode::Down => self.search_result_state.select_next(),
                        KeyCode::Tab => self.search_result_state.select_next(),
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

    pub fn get_selected_ref(&self) -> Option<Ref> {
        let i = self.search_result_state.selected()?;
        self.search_results
            .clone()
            .into_sorted_iter()
            .nth(i)
            .map(|x| x.0)
    }
}
