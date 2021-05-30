

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

pub fn print_occurences(line: &str, occurences: &Vec<FileLocation>, mut write: impl FnMut(&str) -> ()) {
    write(line);

    for loc in occurences {
        write(&format!("\n\t{}", &loc));
    }
}
