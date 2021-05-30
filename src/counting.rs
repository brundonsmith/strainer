use std::{collections::{HashMap, HashSet}, fmt::Display, path::{Path, PathBuf}};

use crate::{options::Options, pattern::matches};

pub type LineOccurrences = HashMap<String, Vec<FileLocation>>;

/// Return a record of all occurrences of every line in `text`
pub fn count_lines(
    file_path: &Path,
    text: &str,
    options: &Options,
) -> LineOccurrences {
    let mut records = HashMap::new();
    let mut current_line_number = 0;

    walk_lines(text, options,
        |next| {
            if let CharOrLine::Line(line) = next {
                current_line_number += 1;

                record_line(
                    &options,
                    &mut records,
                    file_path,
                    line,
                    current_line_number,
                );
            }
        });

    return records;
}

/// Return a copy of `text` with all duplicate lines removed (the first 
/// instance remains)
pub fn strip_lines(
    text: &str,
    options: &Options,
) -> String {
    let mut found_lines = HashSet::new();
    let mut new_text = String::new();

    walk_lines(text, options,
        |next| {
            match next {
                CharOrLine::Char(ch) => new_text.push(ch),
                CharOrLine::Line(line) => {
                    if found_lines.contains(&line) {
                        // do nothing
                    } else {
                        new_text.push_str(&line);
                        found_lines.insert(line);
                    }
                }
            }
        });

    return new_text;
}

enum CharOrLine {
    Char(char),
    Line(String),
}

/// Walk through the lines in `text`, following the specified behavior from
/// `options`, and do something on each line and each character between lines
fn walk_lines(
    text: &str,
    options: &Options,
    mut handle_next: impl FnMut(CharOrLine) -> (),
) {
    let mut current_line = String::new();
    let mut prev_char: Option<char> = None;

    for c in text.chars() {
        let squashing = prev_char
            .map(|prev| prev == c && options.squash_chars.contains(&prev))
            .unwrap_or(false);

        if squashing {
            handle_next(CharOrLine::Char(c));
        } else if c == options.line_delimiter {
            let completed_line = current_line;
            current_line = String::new();

            handle_next(CharOrLine::Line(completed_line));
            handle_next(CharOrLine::Char(c));
        } else {
            current_line.push(c);
            handle_next(CharOrLine::Char(c));
        }

        prev_char = Some(c);
    }

    handle_next(CharOrLine::Line(current_line));
}

// pub fn count_lines(
//     file_path: &Path,
//     text: &str,
//     options: &CountingOptions,
// ) -> LineRecords {
//     let mut records = HashMap::new();

//     let mut current_line_number = 0;
//     let mut current_line = String::new();
//     let mut prev_char: Option<char> = None;

//     for c in text.chars() {
//         let squashing = prev_char
//             .map(|prev| prev == c && options.squash_chars.contains(&prev))
//             .unwrap_or(false);

//         if squashing {
//             // do nothing
//         } else if c == options.line_delimiter {
//             let completed_line = current_line;
//             current_line = String::new();

//             record_line(
//                 &options,
//                 &mut records,
//                 file_path,
//                 completed_line,
//                 current_line_number,
//             );

//             current_line_number += 1;
//         } else {
//             current_line.push(c);
//         }

//         prev_char = Some(c);
//     }

//     record_line(
//         &options,
//         &mut records,
//         file_path,
//         current_line,
//         current_line_number,
//     );

//     return records;
// }

pub fn merge_records(
    target: &mut LineOccurrences,
    source: LineOccurrences,
) {
    let mut source = source;
    
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
    options: &Options,
    records: &mut LineOccurrences,
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

/// A fully-qualified line location within a file (file path + line number)
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_strip_lines_with_no_duplicates() {
        let s = "[Adblock Plus 2.0]
    ||apps.facebook.com^
    ||apps.facebook.com^$popup
    ||apps.facebook.com^$third-party";

        let stripped = strip_lines(s, &(CountingOptions {
            line_delimiter: '\n',
            squash_chars: vec![],

            // aren't used by this function
            line_pattern: vec![],
            ignore_delimiters: vec![],
            trim_whitespace: false,
            same_file: false,
            remove_duplicates: false,
        }));

        assert_eq!(s, stripped);
    }

}
