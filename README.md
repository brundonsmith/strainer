
# Strainer: Find copypasta in your project

Strainer is a command-line tool that will recursively search the text files in 
a directory, track all duplicate lines across files, and output the matched
lines and where they reside in each file.

## Installation

Strainer is available on crates.io:
```
cargo install strainer
```

It has one compile-time feature flag: `syntax-highlighting`. With this enabled 
the `syntect` library will be used to automatically syntax-highlight code lines 
in the output. This roughly doubles the binary size (it's still small), and the 
coloration doesn't work correctly on the default macOS terminal app (but iTerm2 
works fine).
```
cargo install strainer --features "syntax-highlighting"
```

## Usage

```
USAGE:
    strainer [FLAGS] [OPTIONS] <DIRECTORY>

FLAGS:
    -h, --help                 Prints help information
    -r, --remove_duplicates    Remove duplicate lines (keep the first occurrence). Requires --same_file. DANGER:
                               Overwrites source files, use with caution!
    -s, --same_file            Only check for duplicate lines within the same file.
    -t, --trim_whitespace      Trim whitespace from the start and end of each line before comparing.
    -V, --version              Prints version information

OPTIONS:
    -d, --line_delimiter <CHAR>             The character that delimits 'lines'. Can be used, for example, to search a
                                            natural-language file by passing '.' to split on sentences. [default: \n]
    -l, --line_pattern <PAT>                A basic pattern string to filter which lines will show up in results.
                                            Asterisks ('*') will match any substring. [default: *]
    -p, --path_pattern <PAT>                A basic pattern string to filter which files will be searched. Asterisks
                                            ('*') will match any substring. [default: *]
    -s, --squash_chars <squash_chars>...    Characters that should be 'squashed' when processing a line. When a
                                            character is 'squashed', any continuous sequence of that character will be
                                            treated as a single instance. This cen be used to, for example, normalize
                                            indentation. [default: false]

ARGS:
    <DIRECTORY>    The root directory to search within
```
