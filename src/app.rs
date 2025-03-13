use std::{env, fs, path::PathBuf};

use futures::future::BoxFuture;
use priority_queue::PriorityQueue;
use ratatui::widgets::ListState;
use rust_search::SearchBuilder;
use syn::{Item, spanned::Spanned};

use crate::error::Result;

#[derive(Hash, Default, Eq, PartialEq, Clone)]
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

        Ok(Self {
            current_screen: Screen::Main,
            refs,
            search_results,
            input: String::new(),
            search_result_state: ListState::default(),
            select_callback: None,
        })
    }
}
