use std::io::{self, prelude::*};
use std::{collections::HashMap, fs::File, path::{PathBuf, Path}, time::SystemTime, sync::{Mutex, Arc}};

extern crate clap;
extern crate crossbeam;

#[cfg(feature = "syntax-highlighting")]
extern crate syntect;

mod gather_paths;
mod counting;
mod common;
mod printing;

use gather_paths::list_files_in_dir;
use counting::{count_lines, CountingOptions, append_records, LineRecords};
use common::parse_pattern;
use printing::print_occurences;

fn main() {
    let matches = clap::App::new("Comb")
        .version("0.1")
        .author("Brandon Smith <mail@brandonsmith.ninja>")
        .about("Find duplicate lines in text files")
        .arg(clap::Arg::with_name("DIRECTORY")
            .help("The root directory to search within")
            .required(true)
            .display_order(0))
        .arg(clap::Arg::with_name("path_pattern")
            .short("p")
            .long("path_pattern")
            .value_name("PAT")
            .help("A basic pattern string to filter which files will be searched. Asterisks ('*') will match any substring.")
            .default_value("*")
            .takes_value(true))
        .arg(clap::Arg::with_name("line_delimiter")
            .short("d")
            .long("line_delimiter")
            .value_name("CHAR")
            .help("The character that delimits 'lines'. Can be used, for example, to comb a natural-language file by passing '.' to split on sentences. [default: \\n]")
            .takes_value(true))
        .arg(clap::Arg::with_name("line_pattern")
            .short("lp")
            .long("line_pattern")
            .value_name("PAT")
            .help("A basic pattern string to filter which lines will show up in results. Asterisks ('*') will match any substring.")
            .default_value("*")
            .takes_value(true))
        .arg(clap::Arg::with_name("trim_whitespace")
            .short("t")
            .long("trim_whitespace")
            .help("Trim whitespace from the start and end of each line before comparing."))
        .arg(clap::Arg::with_name("squash_chars")
            .short("s")
            .long("squash_chars")
            .help("Characters that should be 'squashed' when processing a line. When a character is 'squashed', any continuous sequence of that character will be treated as a single instance. This cen be used to, for example, normalize indentation.")
            .default_value("false")
            .multiple(true))
        .get_matches();

    let directory = matches.value_of("DIRECTORY").unwrap();
    let path_pattern = matches.value_of("path_pattern").unwrap();
    let options = CountingOptions {
        line_delimiter:    matches.value_of("line_delimiter").map(|s| s.chars().next().unwrap()).unwrap_or('\n'),
        line_pattern:      parse_pattern(matches.value_of("line_pattern").unwrap()),
        trim_whitespace:   matches.is_present("trim_whitespace"),
        squash_chars:      matches.values_of("squash_chars")
                            .map(|iter| 
                                iter.map(|s| s.chars().next().unwrap()).collect())
                            .unwrap_or(vec![]),
        ignore_delimiters: vec![], // TOTO: Implement
    };


    // walk directory recursively and find all target files
    println!("Walking...");
    let start_walk = SystemTime::now();
    let files = list_files_in_dir(
        Path::new(&directory), 
        &parse_pattern(path_pattern)
    ).unwrap();

    let files_arc = Arc::new(&files);
    let options_arc = Arc::new(options);
    let end_walk = SystemTime::now();


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
                        Err(_) => ()
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
        print_occurences(dupe.0, dupe.1);
    }
    
    println!();
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
