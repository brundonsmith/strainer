

#[cfg(feature = "syntax-highlighting")]
use syntect::{util::{as_24_bit_terminal_escaped}, highlighting::{ThemeSet, Style}, easy::HighlightLines, parsing::SyntaxSet};

#[cfg(feature = "syntax-highlighting")]
use std::ffi::OsStr;

use crate::counting::FileLocation;

pub fn print_occurences(line: &str, occurences: &Vec<FileLocation>) {
    
    #[cfg(feature = "syntax-highlighting")]
    {
        let extension: Option<&str> = occurences.iter()
            .map(|o| o.path.extension().and_then(OsStr::to_str))
            .filter_map(|ext| ext)
            .next();

        let output = match extension {
            Some(ext) => {
            
                // Load these once at the start of your program
                let ps = SyntaxSet::load_defaults_newlines();
                let ts = ThemeSet::load_defaults();
            
                let syntax = ps.find_syntax_by_extension(ext);

                match syntax {
                    Some(s) => {
                        let mut h = HighlightLines::new(s, &ts.themes["base16-ocean.dark"]);

                        let ranges: Vec<(Style, &str)> = h.highlight(line, &ps);
                        
                        as_24_bit_terminal_escaped(&ranges[..], false)
                    },
                    None => String::from(line)
                }
            },
            None => String::from(line)
        };

        println!("{}", &output);
    }


    #[cfg(not(feature = "syntax-highlighting"))]
    {
        println!("{}", line);
    }

    for loc in occurences {
        println!("\t{}", &loc);
    }
}
