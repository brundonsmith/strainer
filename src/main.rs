use std::collections::HashSet;
#[cfg(feature = "syntax-highlighting")]
use std::{collections::HashMap, ffi::OsStr};
use std::io::{self, prelude::*};
use std::{fs::File, path::{PathBuf, Path}, time::SystemTime, sync::Arc};

extern crate clap;
extern crate crossbeam;

#[cfg(feature = "syntax-highlighting")]
extern crate syntect;

mod options;
mod gather_paths;
mod counting;
mod pattern;
mod printing;

use clap::ArgMatches;
use gather_paths::list_files_in_dir;
use counting::{LineOccurrences, count_lines, merge_records, strip_lines};
use options::Mode;
use pattern::parse_pattern;
use printing::print_occurences;

#[cfg(feature = "syntax-highlighting")]
use syntect::{easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet};

use crate::options::{Options, SearchResult};

#[cfg(feature = "syntax-highlighting")]
use crate::printing::Highlighter;
#[cfg(feature = "syntax-highlighting")]
use crate::printing::print_occurences_highlighted;

const MAX_THREADS: usize = 10;

fn mode_from_matches(matches: &ArgMatches) -> Result<Mode, &'static str> {
    if matches.is_present("remove_duplicates") {
        if !matches.is_present("same_file") {
            Err("ERROR: Can't use --remove_duplicates without --same_file")
        } else {
            Ok(Mode::RemoveDuplicates)
        }
    } else {
        if !matches.is_present("same_file") {
            Ok(Mode::AllFiles)
        } else {
            Ok(Mode::SameFile)
        }
    }
}

fn main() {
    let matches = clap::App::new("Strainer")
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
            .help("The character that delimits 'lines'. Can be used, for example, to search a natural-language file by passing '.' to split on sentences. [default: \\n]")
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
        .arg(clap::Arg::with_name("same_file")
            .short("sf")
            .long("same_file")
            .help("Only check for duplicate lines within the same file."))
        .arg(clap::Arg::with_name("remove_duplicates")
            .short("rm")
            .long("remove_duplicates")
            .help("Remove duplicate lines (keep the first occurrence). Requires --same_file. DANGER: Overwrites source files, use with caution!"))
        .arg(clap::Arg::with_name("squash_chars")
            .short("s")
            .long("squash_chars")
            .help("Characters that should be 'squashed' when processing a line. When a character is 'squashed', any continuous sequence of that character will be treated as a single instance. This cen be used to, for example, normalize indentation.")
            .default_value("false")
            .multiple(true))
        .get_matches();

    let directory = matches.value_of("DIRECTORY").unwrap();
    let path_pattern = matches.value_of("path_pattern").unwrap();
    let mode = match mode_from_matches(&matches) {
        Ok(mode) => mode,
        Err(e) => panic!("{}", e),
    };

    let options = Options {
        line_delimiter:    matches.value_of("line_delimiter").map(|s| s.chars().next().unwrap()).unwrap_or('\n'),
        line_pattern:      parse_pattern(matches.value_of("line_pattern").unwrap()),
        trim_whitespace:   matches.is_present("trim_whitespace"),
        mode,
        squash_chars:      matches.values_of("squash_chars")
                            .map(|iter| 
                                iter.map(|s| s.chars().next().unwrap()).collect())
                            .unwrap_or(vec![]),
        ignore_delimiters: vec![], // TOTO: Implement
    };


    // walk directory recursively and find all target files
    println!("Searching...");
    let start_listing = SystemTime::now();
    let files = list_files_in_dir(
        Path::new(&directory), 
        &parse_pattern(path_pattern)
    ).unwrap();

    let files_ref = &files;
    let options_ref = &options;
    let end_walk = SystemTime::now();

    let results = SearchResult::from_mode(options_ref.mode);
    let results_arc = Arc::new(&results);

    let start_processing = SystemTime::now();
    let files_count = files.len();
    let files_per_thread = std::cmp::max(files_count / MAX_THREADS, 1);
    crossbeam::scope(move |scope| {
        for chunk in files_ref.chunks(files_per_thread) {
            let local_results_arc = results_arc.clone();

            scope.spawn(move |_| {
                for file_path in chunk {
                    match local_results_arc.as_ref() {
                        SearchResult::RemoveDuplicates => {
                            dedupe_file(options_ref, &file_path).unwrap();
                        },
                        SearchResult::SameFile(results) => {
                            if let Ok(file_results) = search_file(options_ref, &file_path) {
                                results.lock().unwrap().push(file_results);
                            }
                        },
                        SearchResult::AllFiles(results) => {
                            if let Ok(file_results) = search_file(options_ref, &file_path) {
                                merge_records(&mut results.lock().unwrap(), file_results);
                            }
                        },
                    }
                }
            });
        }
    }).unwrap();
    let end_search = SystemTime::now();


    // filter out lines with no duplication
    match results {
        SearchResult::RemoveDuplicates => {
            println!("Searched {} files and removed any duplicate lines", files_count);
        },
        SearchResult::SameFile(results) => {
            let results_lock = results.lock().unwrap();

            let mut output_buffer = String::new();
            let mut duplicate_count = 0;

            #[cfg(feature = "syntax-highlighting")]
            let ps = SyntaxSet::load_defaults_newlines();
            #[cfg(feature = "syntax-highlighting")]
            let ts = ThemeSet::load_defaults();
            #[cfg(feature = "syntax-highlighting")]
            let mut highlighters_by_ext: HashMap<String, Highlighter> = HashMap::new();

            for file_results in results_lock.iter() {
                for (line, occurences) in file_results.iter().filter(|entry| entry.1.len() > 1) {
                    duplicate_count += 1;
                    output_buffer.push_str("\n\n"); // spacing

                    #[cfg(feature = "syntax-highlighting")]
                    {
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
                        }
                    }

                    #[cfg(not(feature = "syntax-highlighting"))]
                    {
                        print_occurences(line, occurences, |str| output_buffer.push_str(str));
                    }
                }
            }

            let files_with_duplicates = {
                let mut files_set = HashSet::new();
                for file_results in results_lock.iter() {
                    for dupe in file_results.iter().filter(|entry| entry.1.len() > 1) {
                        for fl in dupe.1 {
                            files_set.insert(&fl.path);
                        }
                    }
                }

                files_set.len()
            };

            println!("{}", &output_buffer);
            println!();
            println!("Searched {} files", files_count);
            println!("Found {} duplicated lines in {} files", duplicate_count, files_with_duplicates);
        },
        SearchResult::AllFiles(results) => {
            let results_lock = results.lock().unwrap();

            let mut output_buffer = String::new();
            let mut duplicate_count = 0;

            #[cfg(feature = "syntax-highlighting")]
            let ps = SyntaxSet::load_defaults_newlines();
            #[cfg(feature = "syntax-highlighting")]
            let ts = ThemeSet::load_defaults();
            #[cfg(feature = "syntax-highlighting")]
            let mut highlighters_by_ext: HashMap<String, Highlighter> = HashMap::new();

            let duplicates = results_lock.iter().filter(|entry| entry.1.len() > 1);
            for (line, occurences) in duplicates {
                duplicate_count += 1;
                output_buffer.push_str("\n\n"); // spacing

                #[cfg(feature = "syntax-highlighting")]
                {
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
                    }
                }

                #[cfg(not(feature = "syntax-highlighting"))]
                {
                    print_occurences(line, occurences, |str| output_buffer.push_str(str));
                }
            }

            let files_with_duplicates = {
                let duplicates = results_lock.iter().filter(|entry| entry.1.len() > 1);

                let mut files_set = HashSet::new();
                for dupe in duplicates {
                    for fl in dupe.1 {
                        files_set.insert(&fl.path);
                    }
                }

                files_set.len()
            };

            println!("{}", &output_buffer);
            println!();
            println!("Searched {} files", files_count);
            println!("Found {} duplicated lines in {} files", duplicate_count, files_with_duplicates);
        },
    };

    println!(
        "Determining file list took {:?}ms",
        end_walk.duration_since(start_listing).unwrap().as_millis()
    );

    println!(
        "Processing files took {:?}ms",
        end_search.duration_since(start_processing).unwrap().as_millis()
    );
}


fn search_file(options: &Options, file_path: &PathBuf) -> Result<LineOccurrences, io::Error> {
    let mut contents = String::new();

    let mut file = File::open(file_path)?;
    file.read_to_string(&mut contents)?;

    return Ok(count_lines(
        file_path,
        &contents,
        options,
    ));
}

fn dedupe_file(options: &Options, file_path: &PathBuf) -> Result<(), io::Error> {
    let mut contents = String::new();

    let mut file = File::open(file_path)?;
    file.read_to_string(&mut contents)?;

    let new_contents = strip_lines(
        &contents,
        options,
    );

    let mut file = File::create(file_path)?;
    file.write(new_contents.as_bytes())?;

    Ok(())
}
