

#[cfg(feature = "syntax-highlighting")]
use syntect::{easy::HighlightLines, highlighting::Style, parsing::{SyntaxReference, SyntaxSet}, util::{as_24_bit_terminal_escaped}};

use crate::counting::FileLocation;

#[cfg(feature = "syntax-highlighting")]
pub fn print_occurences_highlighted(line: &str, occurences: &Vec<FileLocation>, mut write: impl FnMut(&str) -> (), ps: &SyntaxSet, mut h: HighlightLines) {
    
    // Syntax-color if possible
    let ranges: Vec<(Style, &str)> = h.highlight(line, ps);

    write(&as_24_bit_terminal_escaped(&ranges[..], false));

    for loc in occurences {
        write(&format!("\n\t{}", &loc));
    }
}

#[cfg(feature = "syntax-highlighting")]
pub struct Highlighter<'a> {
    pub syntax: &'a SyntaxReference,
    pub h: HighlightLines<'a>
}

fn print_occurences(line: &str, occurences: &Vec<FileLocation>, mut write: impl FnMut(&str) -> ()) {
    write(line);

    for loc in occurences {
        write(&format!("\n\t{}", &loc));
    }
}


#[cfg(not(feature = "syntax-highlighting"))]
pub fn print_all_unhighlighted<'a>(duplicates: impl Iterator<Item=(&'a String, &'a Vec<FileLocation>)>) -> (String, usize) {
    let mut output_buffer = String::new();
    let mut duplicate_count = 0;

    for (line, occurences) in duplicates {
        duplicate_count += 1;
        output_buffer.push_str("\n\n"); // spacing

        print_occurences(line, occurences, |str| output_buffer.push_str(str));
    }

    (output_buffer, duplicate_count)
}

#[cfg(feature = "syntax-highlighting")]
pub fn print_all_highlighted<'a>(duplicates: impl Iterator<Item=(&'a String, &'a Vec<FileLocation>)>) -> (String, usize) {
    let mut output_buffer = String::new();
    let mut duplicate_count = 0;

    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let mut highlighters_by_ext: HashMap<String, Highlighter> = HashMap::new();

    for (line, occurences) in duplicates {
        duplicate_count += 1;
        output_buffer.push_str("\n\n"); // spacing

        // Get the first file extension we can from the files where this line
        // was found (assumes the file extensions are the same)
        let extension: Option<&str> = occurences.iter()
            .map(|o| o.path.extension().and_then(OsStr::to_str))
            .filter_map(|ext| ext)
            .next();


        if let Some(ext) = extension {
            if let Some(highlighter) = highlighters_by_ext.get_mut(ext) {
                // found in hashmap
                print_occurences_highlighted(line, occurences, |str| output_buffer.push_str(str), &ps, HighlightLines::new(highlighter.syntax, &ts.themes["base16-ocean.dark"]))
            } else if let Some(syntax) = ps.find_syntax_by_extension(ext) {
                // valid in library
                let h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
                let highlighter = Highlighter { syntax, h };

                print_occurences_highlighted(line, occurences, |str| output_buffer.push_str(str), &ps, HighlightLines::new(highlighter.syntax, &ts.themes["base16-ocean.dark"]));

                highlighters_by_ext.insert(String::from(ext), highlighter);
            } else {
                // can't highlight
                print_occurences(line, occurences, |str| output_buffer.push_str(str));
            }
        } else {
            // no extension
            print_occurences(line, occurences, |str| output_buffer.push_str(str));
        }
    }

    (output_buffer, duplicate_count)
}