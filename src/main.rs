use std::io::{self, prelude::*};
use std::{collections::HashMap, fs::File, path::{PathBuf, Path}, time::SystemTime, sync::{Mutex, Arc}};

extern crate crossbeam;

mod gather_paths;
mod counting;
mod common;

use gather_paths::list_files_in_dir;
use counting::{count_lines, CountingOptions, append_records, LineRecords};
use common::parse_pattern;

fn main() {

    let options = CountingOptions {
        line_delimiter: '\n',
        line_pattern: parse_pattern("*{"),
        squash_chars: vec![' ', '\t'],
        ignore_delimiters: vec![],
        trim_whitespace: true
    };


    // walk directory recursively and find all target files
    println!("Walking...");
    let start_walk = SystemTime::now();
    let files = list_files_in_dir(
        Path::new("/Users/brundolf/git/sovereignty"), 
        &parse_pattern("*")
    ).unwrap();

    let files_arc = Arc::new(&files);
    let options_arc = Arc::new(options);
    let end_walk = SystemTime::now();

    for path in files.iter() {
        println!("{:?}", path);
    }


    // comb each file in list
    let results = Mutex::new(HashMap::new());
    let results_arc = Arc::new(&results);

    let start_search = SystemTime::now();
    let files_count = files.len();
    crossbeam::scope(move |scope| {

        for chunk in files_arc.clone().chunks(100) {
            let local_options_arc = options_arc.clone();
            let local_results_arc = results_arc.clone();

            scope.spawn(move |_| {
                for file_path in chunk {
                    match search_file(&local_options_arc.clone(), &file_path) {
                        Ok(mut file_results) => {
                            let mut results_lock = local_results_arc.lock().unwrap();
                            append_records(&mut results_lock, &mut file_results);
                            std::mem::drop(results_lock);
                        },
                        Err(e) => println!("Failed to read file {:?}: '{}'", &file_path, &e)
                    }
                }
            });
        }
    }).unwrap();
    let end_search = SystemTime::now();

    // filter out lines with no duplication
    let results_lock = results.lock().unwrap();
    let duplicates = results_lock.iter().filter(|entry| entry.1.len() > 1);
    for dupe in duplicates {
        println!();
        println!("{}", dupe.0);

        for loc in dupe.1 {
            println!("\t{}", &loc);
        }
    }
    
    println!("{} files found", files_count);
    println!(
        "Walking took {:?}ms",
        end_walk.duration_since(start_walk).unwrap().as_millis()
    );

    println!(
        "Searching took {:?}ms",
        end_search.duration_since(start_search).unwrap().as_millis()
    );

}


fn search_file(options: &CountingOptions, file_path: &PathBuf) -> Result<LineRecords, io::Error> {
    let mut contents = String::new();

    let mut file = File::open(file_path)?;
    file.read_to_string(&mut contents)?;

    return Ok(count_lines(
        file_path,
        &contents,
        options,
    ));
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
