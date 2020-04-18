
use std::{path::{PathBuf, Path}, fs, io};

use crate::common::matches;

pub fn list_files_in_dir(path: &Path, pattern: &str) -> Result<Vec<PathBuf>, io::Error> {
  if path.is_dir() {
      let mut all_children = vec![];

      for entry in fs::read_dir(path)? {
          let child_path = entry?.path();
          let mut child_contents = list_files_in_dir(&path.join(&child_path), pattern)?;

          all_children.append(&mut child_contents);
      }

      return Ok(all_children);
  } else if matches(path.to_str().unwrap(), pattern) {
      return Ok(vec![ PathBuf::from(path) ]);
  } else {
      return Ok(vec![]);
  }
}
