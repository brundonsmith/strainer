
/// A pattern against which strings can be matched
pub type Pattern<'a> = Vec<&'a str>;

pub fn parse_pattern(pattern_str: &str) -> Pattern {
    pattern_str.split('*').collect()
}

pub fn matches<'a>(s: &str, pattern: &Pattern) -> bool {
    let mut remainder = Some(s);
  
    for segment in pattern {
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