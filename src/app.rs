use std::{env, fs, path::{Path, PathBuf}, usize};

use ratatui::widgets::ListState;
use rust_search::SearchBuilder;
use syn::{Item, spanned::Spanned};

#[derive(Hash, Default, Eq, PartialEq)]
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
            Item::Fn(item) => {
                Ref {
                    line: item.span().start().line,
                    column: item.span().start().column,
                    file: self.1,
                    sig,
                }
            }
            Item::Mod(item) => {
                Ref {
                    line: item.span().start().line,
                    column: item.span().start().column,
                    file: self.1,
                    sig,
                }
            }
            Item::Enum(item) => {
                Ref {
                    line: item.span().start().line,
                    column: item.span().start().column,
                    file: self.1,
                    sig,
                }
            }
            Item::Trait(item) => {
                Ref {
                    line: item.span().start().line,
                    column: item.span().start().column,
                    file: self.1,
                    sig,
                }
            }
            Item::Struct(item) => {
                Ref {
                    line: item.span().start().line,
                    column: item.span().start().column,
                    file: self.1,
                    sig,
                }
            }
            _ => Ref::default()
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
                        .expect("Getting source text of signature")
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

pub struct App {
    pub current_screen: Screen,
    pub refs: Vec<Ref>,
    pub input: String,
    pub search_result_state: ListState,
}

impl App {
    pub fn new() -> Self {
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
            let src = fs::read_to_string(&file).expect("Reading file contents to string");
            let syntax = syn::parse_file(&src).expect("Parsing file contents");
            // Append refs with each item in the file that is relevant
            let rel_items: Vec<_> = syntax.items.iter().filter(|e| e.is_relevant()).collect();
            for item in rel_items {
                refs.push((item.to_owned(), file.to_owned()).into());
            }
        }

        Self {
            current_screen: Screen::Main,
            refs,
            input: String::new(),
            search_result_state: ListState::default(),
        }
    }
}
