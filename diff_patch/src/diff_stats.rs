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

use std::fmt::{self, Display, Formatter};
use std::slice::Iter;

use regex::Regex;

use crate::lines::{Line, Lines};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum DiffStatsCategory {
    Inserted = 0,
    Deleted = 1,
    Modified = 2,
    Unchanged = 3,
}

impl Display for DiffStatsCategory {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            DiffStatsCategory::Inserted => write!(f, "inserted"),
            DiffStatsCategory::Deleted => write!(f, "deleted"),
            DiffStatsCategory::Modified => write!(f, "modified"),
            DiffStatsCategory::Unchanged => write!(f, "unchanged"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct DiffStats {
    stats: [u64; 4],
}

impl DiffStats {
    pub fn count(&self, category: DiffStatsCategory) -> u64 {
        self.stats[category as usize]
    }

    pub fn incr_count(&mut self, category: DiffStatsCategory, by: u64) {
        self.stats[category as usize] += by
    }
}

pub struct DiffStatsLines {
    lines: Lines,
    stats: DiffStats,
}

impl DiffStatsLines {
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn iter(&self) -> Iter<Line> {
        self.lines.iter()
    }

    pub fn stats(&self) -> &DiffStats {
        &self.stats
    }
}

pub struct DiffStatParser {
    empty_cre: Regex,
    end_cre: Regex,
    file_stats_cre: Regex,
    blank_line_cre: Regex,
    divider_line_cre: Regex,
}

impl DiffStatParser {
    pub fn new() -> Self {
        let end_cre_str = format!(
            "{}{}{}{}",
            r"^#? (\d+) files? changed",
            r"(, (\d+) insertions?\(\+\))?",
            r"(, (\d+) deletions?\(-\))?",
            r"(, (\d+) modifications?\(!\))?(\n)?$",
        );
        DiffStatParser {
            empty_cre: Regex::new(r"^#? 0 files changed(\n)?$").unwrap(),
            end_cre: Regex::new(&end_cre_str).unwrap(),
            file_stats_cre: Regex::new(r"^#? (\S+)\s*\|((binary)|(\s*(\d+)(\s+\+*-*!*)?))(\n)$")
                .unwrap(),
            blank_line_cre: Regex::new(r"^\s*(\n)$").unwrap(),
            divider_line_cre: Regex::new(r"^---(\n)$").unwrap(),
        }
    }

    pub fn get_summary_line_range_at(
        &self,
        lines: &[Line],
        start_index: usize,
    ) -> Option<(usize, usize)> {
        let mut index = start_index;

        if self.divider_line_cre.is_match(&lines[index]) {
            index += 1;
        }
        while index < lines.len() && self.blank_line_cre.is_match(&lines[index]) {
            index += 1;
        }
        if index >= lines.len() {
            return None;
        }
        if self.empty_cre.is_match(&lines[index]) {
            return Some((start_index, index));
        }
        while index < lines.len() && self.file_stats_cre.is_match(&lines[index]) {
            index += 1;
        }
        if index < lines.len() && self.end_cre.is_match(&lines[index]) {
            return Some((start_index, index));
        }
        // TODO: worry about malformed summary
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
