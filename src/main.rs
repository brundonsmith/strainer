use std::io::prelude::*;
use std::{collections::HashMap, fs::File, path::Path, time::SystemTime};

mod gather_paths;
mod counting;
mod common;

use gather_paths::list_files_in_dir;
use counting::{append_records, count_lines, CountingOptions};

fn main() {
    println!("Walking...");
    let start_walk = SystemTime::now();
    let files = list_files_in_dir(Path::new("/Users/brundolf/git/sovereignty"), "*.css").unwrap();
    let end_walk = SystemTime::now();

    for path in files.iter() {
        println!("{:?}", path);
    }

    println!("{} files found", files.len());
    println!(
        "Walking took {:?}ms",
        end_walk.duration_since(start_walk).unwrap().as_millis()
    );

    let start_search = SystemTime::now();
    let mut results = HashMap::new();

    for file_path in files {
        let mut file = File::open(file_path.clone()).unwrap();

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {
                let mut file_results = count_lines(
                    file_path.to_str().unwrap(),
                    &contents,
                    CountingOptions {
                        line_delimiter: '\n',
                        line_pattern: "*{",
                        squash_chars: vec![' ', '\t'],
                        ignore_delimiters: vec![],
                        trim_whitespace: true
                    },
                );

                append_records(&mut results, &mut file_results);
            },
            Err(e) => {
                println!("Failed to read file {:?}: '{}'", &file_path, &e);
            }
        }
    }

    let end_search = SystemTime::now();

    let duplicates = results.iter().filter(|entry| entry.1.len() > 1);

    for dupe in duplicates {
        println!();
        println!("{}", dupe.0);

        for loc in dupe.1 {
            println!("\t{}", &loc);
        }
    }

    println!(
        "Searching took {:?}ms",
        end_search.duration_since(start_search).unwrap().as_millis()
    );
}

/*
Start process:
    - path pattern
    - options
        - segment delimiter
        - character groups to "squash"
        - ignore strings (string delimiters?)

1) list out all files (single thread?)
2) delegate fractions of the file set to different threads
3) data structure for line values and locations
*/
