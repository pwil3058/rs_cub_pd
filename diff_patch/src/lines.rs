// Copyright 2019 Peter Williams <pwil3058@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::Path;
pub use std::sync::Arc;

pub type Line = Arc<String>;
pub type Lines = Vec<Line>;

pub trait LineIfce {
    fn new(s: &str) -> Line {
        Arc::new(String::from(s))
    }

    fn conflict_start_marker() -> Line {
        Arc::new(String::from("<<<<<<<"))
    }

    fn conflict_separation_marker() -> Line {
        Arc::new(String::from("======="))
    }

    fn conflict_end_marker() -> Line {
        Arc::new(String::from(">>>>>>>"))
    }
}

impl LineIfce for Line {}

pub trait LinesIfce {
    fn read(path: &Path) -> io::Result<Lines> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut lines = vec![];
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 {
                break;
            } else {
                lines.push(Arc::new(line))
            }
        }
        Ok(lines)
    }

    fn from_string(string: &str) -> Lines {
        let mut lines: Lines = vec![];
        let mut start_index = 0;
        for (end_index, _) in string.match_indices("\n") {
            lines.push(Arc::new(string[start_index..end_index + 1].to_string()));
            start_index = end_index + 1;
        }
        if start_index < string.len() {
            lines.push(Arc::new(string[start_index..].to_string()));
        }
        lines
    }

    // Does we contain "sub_lines" starting at "index"?
    fn contains_sub_lines_at(&self, sub_lines: &[Line], index: usize) -> bool;

    // Find index of the first instance of "sub_lines" at or after "start_index"
    fn find_first_sub_lines(&self, sub_lines: &[Line], start_index: usize) -> Option<usize>;
}

impl LinesIfce for Lines {
    fn contains_sub_lines_at(&self, sub_lines: &[Line], index: usize) -> bool {
        if sub_lines.len() + index > self.len() {
            return false;
        }
        for (line, sub_line) in self[index..index + sub_lines.len()].iter().zip(sub_lines) {
            if line != sub_line {
                return false;
            }
        }
        true
    }

    fn find_first_sub_lines(&self, sub_lines: &[Line], start_index: usize) -> Option<usize> {
        for index in start_index..start_index + self.len() - sub_lines.len() + 1 {
            if self.contains_sub_lines_at(sub_lines, index) {
                return Some(index);
            }
        }
        None
    }
}

pub fn first_inequality_fm_head(lines1: &Lines, lines2: &Lines) -> Option<usize> {
    if let Some(index) = lines1.iter().zip(lines2.iter()).position(|(a, b)| a != b) {
        Some(index)
    } else {
        if lines1.len() == lines2.len() {
            None
        } else {
            Some(lines1.len().min(lines2.len()))
        }
    }
}

pub fn first_inequality_fm_tail(lines1: &Lines, lines2: &Lines) -> Option<usize> {
    if let Some(index) = lines1
        .iter()
        .rev()
        .zip(lines2.iter().rev())
        .position(|(a, b)| a != b)
    {
        Some(index)
    } else {
        if lines1.len() > lines2.len() {
            Some(lines1.len() - lines2.len())
        } else if lines2.len() > lines1.len() {
            Some(lines2.len() - lines1.len())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_from_strings_works() {
        let test_string = " aaa\nbbb \nccc ddd\njjj";
        let lines = Lines::from_string(test_string);
        assert!(lines.len() == 4);
        let lines = Lines::from_string(&test_string.to_string());
        assert!(lines.len() == 4);
        assert!(*lines[0] == " aaa\n");
        assert!(*lines[3] == "jjj");
        let test_string = " aaa\nbbb \nccc ddd\njjj\n";
        let lines = Lines::from_string(test_string);
        assert!(lines.len() == 4);
        let lines = Lines::from_string(&test_string.to_string());
        assert!(lines.len() == 4);
        assert!(*lines[0] == " aaa\n");
        assert!(*lines[3] == "jjj\n");
    }
}
