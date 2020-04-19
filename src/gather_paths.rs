use std::io::{self};
use std::{
    fs,
    path::{Path, PathBuf}, collections::VecDeque,
};

use crate::common::{matches,Pattern};

pub fn list_files_in_dir(root_dir: &Path, pattern: &Pattern) -> Result<Vec<PathBuf>, io::Error> {
    let mut dir_queue: VecDeque<PathBuf> = VecDeque::new();
    dir_queue.push_back(PathBuf::from(root_dir));

    let mut file_paths = Vec::new();

    while !dir_queue.is_empty() {
        let next_dir = dir_queue.pop_front().unwrap();
        
        for child_path in fs::read_dir(next_dir)?.filter_map(|entry| entry.ok()).map(|entry| entry.path()) {
            if child_path.is_dir() {
                dir_queue.push_back(PathBuf::from(child_path));
            } else if matches(child_path.to_str().unwrap(), pattern) {
                file_paths.push(PathBuf::from(child_path));
            }
        }
    }

    return Ok(file_paths);
}
