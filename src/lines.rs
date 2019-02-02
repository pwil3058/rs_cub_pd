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
                break
            } else {
                lines.push(Arc::new(line))
            }
        }
        Ok(lines)
    }
    
    // Does we contain "sub_lines" starting at "index"?
    fn contains_sub_lines_at(&self, sub_lines: &[Line], index: usize) -> bool;

    // Find index of the first instance of "sub_lines" at or after "start_index"
    fn find_first_sub_lines(&self, sub_lines: &[Line], start_index: usize) -> Option<usize>;
}

impl LinesIfce for Lines {
    fn contains_sub_lines_at(&self, sub_lines: &[Line], index: usize) -> bool {
        if sub_lines.len() + index > self.len() {
            return false
        }
        for (line, sub_line) in self[index..index + sub_lines.len()].iter().zip(sub_lines) {
            if line != sub_line {
                return false
            }
        }
        true
    }

    fn find_first_sub_lines(&self, sub_lines: &[Line], start_index: usize) -> Option<usize> {
        for index in start_index..start_index + self.len() - sub_lines.len() + 1 {
            if self.contains_sub_lines_at(sub_lines, index) {
                return Some(index)
            }
        };
        None
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
