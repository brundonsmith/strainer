use std::{collections::HashMap, fmt::Display, path::PathBuf};

use crate::common::matches;

#[derive(Debug)]
pub struct CountingOptions<'a> {
    pub line_delimiter: char,
    pub line_pattern: &'a str,
    pub squash_chars: Vec<char>,
    pub ignore_delimiters: Vec<char>,
    pub trim_whitespace: bool,
}

pub fn count_lines(
    file_path: &str,
    text: &str,
    options: CountingOptions,
) -> HashMap<String, Vec<FileLocation>> {
    let mut records: HashMap<String, Vec<FileLocation>> = HashMap::new();

    let mut current_line_number = 0;
    let mut current_line = String::new();
    let mut prev_char: Option<char> = None;

    for c in text.chars() {
        if prev_char
            .map(|prev| prev == c && options.squash_chars.contains(&prev))
            .unwrap_or(false)
        {
            // do nothing
        } else if c == options.line_delimiter {
            if options.trim_whitespace {
                current_line = String::from(current_line.trim());
            }
            
            let completed_line = current_line;
            current_line = String::new();
            
            if matches(&completed_line, &options.line_pattern) {
                record_line(&mut records, file_path, completed_line, current_line_number);
            }

            current_line_number += 1;
        } else {
            current_line.push(c);
        }

        prev_char = Some(c);
    }

    if options.trim_whitespace {
        current_line = String::from(current_line.trim());
    }

    if matches(&current_line, &options.line_pattern) {
        record_line(&mut records, file_path, current_line, current_line_number);
    }

    return records;
}

pub fn append_records(
    target: &mut HashMap<String, Vec<FileLocation>>,
    source: &mut HashMap<String, Vec<FileLocation>>,
) {
    for key in source.keys().map(|k| k.clone()).collect::<Vec<String>>() {
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
    records: &mut HashMap<String, Vec<FileLocation>>,
    file_path: &str,
    line: String,
    line_number: u32,
) {
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