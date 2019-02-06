//Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::slice::Iter;

use crate::diff_stats::DiffStatParser;
use crate::lines::*;

pub struct PatchHeader {
    lines: Lines,
    comment: (usize, usize),
    description: (usize, usize),
    diff_stats_lines: (usize, usize),
}

impl PatchHeader {
    pub fn new(lines: &[Line]) -> PatchHeader {
        let mut descr_starts_at = 0;
        for line in lines {
            if !line.starts_with("#") {
                break;
            }
            descr_starts_at += 1;
        }
        let mut diff_stats_range = None;
        let mut index = descr_starts_at;
        let parser = DiffStatParser::new();
        while index < lines.len() {
            diff_stats_range = parser.get_summary_line_range_at(&lines, index);
            if diff_stats_range.is_some() {
                break;
            }
            index += 1;
        }
        if let Some(diff_stats_range) = diff_stats_range {
            PatchHeader {
                lines: lines.to_vec(),
                comment: (0, descr_starts_at),
                description: (descr_starts_at, diff_stats_range.0),
                diff_stats_lines: (diff_stats_range.0, diff_stats_range.1),
            }
        } else {
            PatchHeader {
                lines: lines.to_vec(),
                comment: (0, descr_starts_at),
                description: (descr_starts_at, lines.len()),
                diff_stats_lines: (0, 0),
            }
        }
    }

    pub fn iter(&self) -> Iter<Line> {
        self.lines.iter()
    }

    pub fn iter_comment(&self) -> Iter<Line> {
        self.lines[self.comment.0..self.comment.1].iter()
    }

    pub fn iter_description(&self) -> Iter<Line> {
        self.lines[self.description.0..self.description.1].iter()
    }

    pub fn iter_diff_stats_lnes(&self) -> Iter<Line> {
        self.lines[self.diff_stats_lines.0..self.diff_stats_lines.1].iter()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
