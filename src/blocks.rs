use std::{collections::{HashMap, HashSet}, hash::BuildHasherDefault, path::Path};

use rustc_hash::FxHasher;

use crate::{counting::{Occurrences, record_line}, options::Options};

type OccurrencesMap<TSequence, TLocation> = HashMap<TSequence, HashSet<TLocation, BuildHasherDefault<FxHasher>>, BuildHasherDefault<FxHasher>>;

pub fn count_chunks(
    file_path: &Path,
    text: &str,
    options: &Options,
) -> Occurrences {
    let mut records = HashMap::new();
    
    let lines = text.lines()
        .map(|line| if options.trim_whitespace { line.trim() } else { line })
        .filter(|line| line.len() > 0)
        .collect::<Vec<&str>>();

    let chunks = all_chunks(&lines);

    for (block, line_numbers) in chunks {
        let block_string = block.join("\n");
        
        for line_number in line_numbers {
            record_line(
                &options,
                &mut records,
                file_path,
                &block_string,
                line_number,
            );
        }
    }

    return records;
}

fn all_chunks<'a>(items: &'a [&'a str]) -> OccurrencesMap<&'a [&'a str], usize> {
    let mut chunk_occurrences: OccurrencesMap<&'a [&'a str], usize> = HashMap::with_hasher(BuildHasherDefault::default());
    // println!("{:?}", items);
    for start in 0..items.len()-1 {
        // println!("start: {}", start);
        for end in start+2..items.len()+1 {
            // println!("  end: {}", end);
            let chunk = &items[start..end];
            
            if chunk_occurrences.contains_key(chunk) {
                chunk_occurrences.get_mut(chunk).unwrap().insert(start);
            } else {
                chunk_occurrences.insert(chunk, [start].iter().map(|x| *x).collect());
            }
        }
    }

    // println!("{:?}", &chunk_occurrences);

    return chunk_occurrences;
}
