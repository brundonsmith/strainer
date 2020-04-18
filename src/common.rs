
pub fn matches(s: &str, pattern: &str) -> bool {
    let mut remainder = Some(s);
  
    for segment in pattern.split('*') {
        match remainder {
            Some(slice) => {
                match slice.find(segment) {
                    Some(index) => remainder = slice.get(index..),
                    None => return false
                }
            },
            None => return false
        }
    }
  
    return true;
  }