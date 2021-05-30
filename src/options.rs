use std::{collections::HashMap, sync::Mutex};

use crate::{counting::FileLocation, pattern::Pattern};

#[derive(Debug)]
pub struct Options<'a> {
    pub line_delimiter: char,
    pub line_pattern: Pattern<'a>,
    pub squash_chars: Vec<char>,
    pub ignore_delimiters: Vec<char>,
    pub trim_whitespace: bool,
    pub mode: Mode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    AllFiles,
    SameFile,
    RemoveDuplicates,
}

pub enum SearchResult {
    AllFiles(Mutex<HashMap<String, Vec<FileLocation>>>),
    SameFile(Mutex<Vec<HashMap<String, Vec<FileLocation>>>>),
    RemoveDuplicates,
}

impl SearchResult {
    pub fn from_mode(mode: Mode) -> Self {
        match mode {
            Mode::AllFiles => Self::AllFiles(Mutex::new(HashMap::new())),
            Mode::SameFile => Self::SameFile(Mutex::new(Vec::new())),
            Mode::RemoveDuplicates => Self::RemoveDuplicates,
        }
    }
}