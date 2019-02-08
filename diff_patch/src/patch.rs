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

use crate::diff::{DiffPlus, DiffPlusParser};
use crate::diff_stats::DiffStatParser;
use crate::lines::*;
use crate::text_diff::DiffParseResult;
use crate::MultiListIter;

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

pub struct Patch {
    length: usize,
    header: PatchHeader,
    diff_pluses: Vec<DiffPlus>,
    rubbish: Vec<Lines>, // some tools put rubbish between diffs
}

impl Patch {
    pub fn len(&self) -> usize {
        self.length
    }

    pub fn iter(&self) -> MultiListIter<Line> {
        let mut mli = MultiListIter::<Line>::new(vec![self.header.iter()]);
        for (diff_plus, rubbish) in self.diff_pluses.iter().zip(self.rubbish.iter()) {
            mli.append(&mut diff_plus.iter());
            mli.push(rubbish.iter());
        }
        mli
    }

    pub fn num_files(&self) -> usize {
        self.diff_pluses.len()
    }
}

pub struct PatchParser {
    diff_plus_parser: DiffPlusParser,
}

impl PatchParser {
    pub fn new() -> PatchParser {
        PatchParser {
            diff_plus_parser: DiffPlusParser::new(),
        }
    }

    pub fn parse_lines(&self, lines: &[Line]) -> DiffParseResult<Option<Patch>> {
        let mut diffs_start_at: Option<usize> = None;
        let mut rubbish_starts_at = 0;
        let mut index = 0;
        let mut diff_pluses: Vec<DiffPlus> = Vec::new();
        let mut rubbish: Vec<Lines> = Vec::new();
        while index < lines.len() {
            if let Some(diff_plus) = self.diff_plus_parser.get_diff_plus_at(lines, index)? {
                if diffs_start_at.is_some() {
                    rubbish.push(lines[rubbish_starts_at..index].to_vec());
                } else {
                    diffs_start_at = Some(index);
                }
                index += diff_plus.len();
                rubbish_starts_at = index;
                diff_pluses.push(diff_plus);
            } else {
                index += 1;
            }
        }
        rubbish.push(lines[rubbish_starts_at..].to_vec());
        if let Some(end_of_header) = diffs_start_at {
            let header = PatchHeader::new(&lines[0..end_of_header]);
            Ok(Some(Patch {
                length: lines.len(),
                header,
                diff_pluses,
                rubbish,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn patch_parse_lines_works() {
        let lines = Lines::read_from(&Path::new("../test_diffs/test_1.diff")).unwrap();
        let lines_length = lines.len();
        let parser = PatchParser::new();
        let result = parser.parse_lines(&lines);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let patch = result.unwrap();
        for (line1, line2) in patch.iter().zip(lines.iter()) {
            assert!(line1 == line2);
        }
        assert!(patch.len() == lines_length);
        assert!(patch.iter().count() == lines_length);
        assert!(patch.iter().count() == patch.len());
        assert!(patch.num_files() == 2);
    }
}
