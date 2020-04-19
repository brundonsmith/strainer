use std::{collections::HashMap, fmt::Display, path::{Path, PathBuf}};

use crate::common::{matches, Pattern};

pub type LineRecords = HashMap<String, Vec<FileLocation>>;

#[derive(Debug)]
pub struct CountingOptions<'a> {
    pub line_delimiter: char,
    pub line_pattern: Pattern<'a>,
    pub squash_chars: Vec<char>,
    pub ignore_delimiters: Vec<char>,
    pub trim_whitespace: bool,
}

pub fn count_lines(
    file_path: &Path,
    text: &str,
    options: &CountingOptions,
) -> LineRecords {
    let mut records = HashMap::new();

    let mut current_line_number = 0;
    let mut current_line = String::new();
    let mut prev_char: Option<char> = None;

    for c in text.chars() {
        let squashing = prev_char
            .map(|prev| prev == c && options.squash_chars.contains(&prev))
            .unwrap_or(false);

        if squashing {
            // do nothing
        } else if c == options.line_delimiter {
            let completed_line = current_line;
            current_line = String::new();

            record_line(
                &options,
                &mut records,
                file_path,
                completed_line,
                current_line_number,
            );

            current_line_number += 1;
        } else {
            current_line.push(c);
        }

        prev_char = Some(c);
    }

    record_line(
        &options,
        &mut records,
        file_path,
        current_line,
        current_line_number,
    );

    return records;
}

pub fn append_records(
    target: &mut LineRecords,
    source: &mut LineRecords,
) {
    let source_keys = source.keys().map(|k| k.clone()).collect::<Vec<String>>();

    for key in source_keys {
        let mut source_val = source.remove(&key).unwrap();

        match target.get_mut(&key) {
            Some(existing_vec) => existing_vec.append(&mut source_val),
            None => {
                target.insert(key, source_val);
            }
        }
    }
}

fn record_line(
    options: &CountingOptions,
    records: &mut LineRecords,
    file_path: &Path,
    line: String,
    line_number: u32,
) {
    let line = if options.trim_whitespace {
        String::from(line.trim())
    } else {
        line
    };

    if matches(&line, &options.line_pattern) {
        if line.len() > 0 {
            let file_location = FileLocation {
                path: PathBuf::from(file_path),
                line_number: line_number,
            };

            match records.get_mut(&line) {
                Some(existing_locations) => existing_locations.push(file_location),
                None => {
                    records.insert(line, vec![file_location]);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct FileLocation {
    pub path: PathBuf,
    pub line_number: u32,
}

impl Display for FileLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.path.to_str().unwrap(), self.line_number)
    }
}
