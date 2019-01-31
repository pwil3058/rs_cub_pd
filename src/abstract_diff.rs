//Copyright 2019 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at

    //http://www.apache.org/licenses/LICENSE-2.0

//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

use std::fmt;
use std::io;
use std::path::Path;
use std::rc::Rc;

use super::Line;

// Does "lines" contain "sub_lines" starting at "index"?
fn lines_contains_sub_lines_at(lines: &[Line], sub_lines: &[Line], index: usize) -> bool {
    if sub_lines.len() + index > lines.len() {
        return false
    }
    for i in 0..sub_lines.len() {
        if sub_lines[i] != lines[i + index] {
            return false
        }
    };
    true
}

// Find index of the first instance of "sub_lines" in "lines" at or after "start_index"
fn find_first_sub_lines(lines: &[Line], sub_lines: &[Line], start_index: usize) -> Option<usize> {
    for index in start_index..start_index + lines.len() - sub_lines.len() + 1 {
        if lines_contains_sub_lines_at(lines, sub_lines, index) {
            return Some(index)
        }
    };
    None
}

trait ApplyOffset {
    fn apply_offset(self, offset: i64) -> usize;
}

impl ApplyOffset for usize {
    fn apply_offset(self, offset: i64) -> usize {
        (self as i64 + offset) as usize
    }    
}

pub struct AbstractChunk {
    start_index: usize,
    lines: Vec<Line>
}

impl AbstractChunk {
    fn end_index(&self) -> usize {
        self.start_index + self.lines.len()
    }

    // Do "lines" match this chunk?
    fn matches_lines(&self, lines: &[Line], offset: i64) -> bool {
        let start_index = self.start_index.apply_offset(offset);
        lines_contains_sub_lines_at(lines, &self.lines, start_index)
    }
}

const BEFORE: usize = 0;
const AFTER: usize = 1;
const FUZZ_FACTOR: usize = 2;
 
pub struct AbstractHunk {
    chunk: [AbstractChunk; 2],
    ante_context_len: usize,
    post_context_len: usize
}

pub struct CompromisedPosnData {
    start_index: usize,
    ante_context_redn: usize,
    post_context_redn: usize
    
}

#[derive(Debug)]
pub struct AppliedPosnData {
    start_posn: usize,
    length: usize
}

impl fmt::Display for AppliedPosnData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "line {} ({} lines)", self.start_posn, self.length)
    }
}

impl AbstractHunk {
    // If it exists find the position in "lines" where this hunk will
    // apply reducing context if/as necessary.  Return the position
    // and any context reductions that were used.
    fn get_compromised_posn(&self, lines: &[Line], start_index: usize, fuzz_factor: usize, reverse: bool) -> Option<CompromisedPosnData> {
        for context_redn in 0..fuzz_factor.min(self.ante_context_len.max(self.post_context_len)) + 1 {
            let ante_context_redn = context_redn.min(self.ante_context_len);
            let post_context_redn = context_redn.min(self.post_context_len);
            let fm = ante_context_redn;
            let before = if reverse { AFTER } else { BEFORE };
            let to = self.chunk[before].lines.len() - post_context_redn;
            if let Some(start_index) = find_first_sub_lines(lines, &self.chunk[before].lines[fm..to], start_index) {
                return Some(CompromisedPosnData{start_index, ante_context_redn, post_context_redn})
            }
        };
        None
    }

    // Return the before applied position data for this hunk.
    fn get_applied_posn(&self, end_posn: usize, post_context_redn: usize, reverse: bool) -> AppliedPosnData {
        let after = if reverse { BEFORE } else { AFTER };
        let length = self.chunk[after].lines.len() - self.ante_context_len - self.post_context_len;
        let start_posn = end_posn - length - (self.post_context_len - post_context_redn) + 1;
        AppliedPosnData{start_posn, length}
    }

    fn is_already_applied(&self, lines: &[Line], offset: i64, reverse: bool) -> bool {
        let (before, after) = if reverse { (AFTER, BEFORE) } else { (BEFORE, AFTER) };
        let fr_offset = self.chunk[before].start_index as i64 - self.chunk[after].start_index as i64;
        self.chunk[after].matches_lines(lines, fr_offset + offset)
    }

    fn length_diff(&self, reverse: bool) -> i64 {
        if reverse {
            self.chunk[BEFORE].lines.len() as i64 - self.chunk[AFTER].lines.len() as i64
        } else {
            self.chunk[AFTER].lines.len() as i64 - self.chunk[BEFORE].lines.len() as i64
        }
    }

    fn len_minus_post_context(&self, reverse: bool) -> usize {
        if reverse {
            self.chunk[BEFORE].lines.len() - self.post_context_len
        } else {
            self.chunk[AFTER].lines.len() - self.post_context_len
        }
    }
}

#[derive(Debug, Default)]
pub struct ApplnResult {
    lines: Vec<Line>,
    successes: u64,
    merges: u64,
    already_applied: u64,
    failures: u64
}

pub struct AbstractDiff {
    hunks: Vec<AbstractHunk>,
}

impl AbstractDiff {
    // Apply this diff to lines
    pub fn apply_to_lines<W>(&self, lines: &[Line], reverse: bool, err_w: &mut W, repd_file_path: Option<&Path>) -> ApplnResult
        where W: io::Write
    {
        let mut result = ApplnResult::default();
        let mut current_offset: i64 = 0;
        let mut lines_index: usize = 0;
        let (before, after) = if reverse { (AFTER, BEFORE) } else { (BEFORE, AFTER) };
        for (hunk_index, hunk) in self.hunks.iter().enumerate() {
            if hunk.chunk[before].matches_lines(lines, current_offset) {
                let index = hunk.chunk[before].start_index.apply_offset(current_offset);
                for line in &lines[lines_index..index] {
                    result.lines.push(line.clone());
                }
                for line in &hunk.chunk[after].lines {
                    result.lines.push(line.clone());
                }
                lines_index = (hunk.chunk[before].start_index + hunk.chunk[before].lines.len()).apply_offset(current_offset);
                result.successes += 1;
                continue
            }
            if let Some(cpd) = hunk.get_compromised_posn(lines, lines_index, FUZZ_FACTOR, reverse) {
                for line in &lines[lines_index..cpd.start_index] {
                    result.lines.push(line.clone());
                }
                let end = &hunk.chunk[before].lines.len() - cpd.post_context_redn;
                for line in &hunk.chunk[before].lines[cpd.ante_context_redn..end] {
                    result.lines.push(line.clone());
                }
                lines_index = cpd.start_index + hunk.chunk[before].lines.len() - cpd.ante_context_redn - cpd.post_context_redn;
                current_offset = cpd.start_index as i64 - hunk.chunk[before].start_index as i64 - cpd.ante_context_redn as i64;
                let applied_posn = hunk.get_applied_posn(result.lines.len(), cpd.post_context_redn, reverse);
                if let Some(file_path) = repd_file_path {
                    write!(err_w, "{:?}: Hunk #{} merged at {}.\n", file_path, hunk_index + 1, applied_posn).unwrap();
                } else {
                    write!(err_w, "Hunk #{} merged at {}.\n", hunk_index + 1, applied_posn).unwrap();
                }
                result.merges += 1;
                continue
            }
            if hunk.is_already_applied(lines, current_offset, reverse) {
                let new_lines_index = hunk.chunk[after].end_index().apply_offset(current_offset);
                for line in &lines[lines_index..new_lines_index] {
                    result.lines.push(line.clone());
                }
                lines_index = new_lines_index;
                current_offset += hunk.length_diff(reverse);
                let applied_posn = hunk.get_applied_posn(result.lines.len(), 0, reverse);
                if let Some(file_path) = repd_file_path {
                    write!(err_w, "{:?}: Hunk #{} already applied at {}.\n", file_path, hunk_index + 1, applied_posn).unwrap();
                } else {
                    write!(err_w, "Hunk #{} already applied at {}.\n", hunk_index + 1, applied_posn).unwrap();
                }
                result.already_applied += 1;
                continue
            }
            let before_hlen = hunk.chunk[before].lines.len() - hunk.post_context_len;
            if (hunk.chunk[before].start_index + before_hlen).apply_offset(current_offset) > lines.len() {
                // We've run out of lines to patch
                if let Some(file_path) = repd_file_path {
                    write!(err_w, "{:?}: Unexpected end of file: ", file_path).unwrap();
                } else {
                    write!(err_w, "Unexpected end of file: ").unwrap();
                }
                let remaining_hunks = self.hunks.len() - hunk_index;
                if remaining_hunks > 1 {
                    write!(err_w, "Hunks #{}-{} could NOT be applied.\n", hunk_index + 1, self.hunks.len()).unwrap()
                } else {
                    write!(err_w, "Hunk #{} could NOT be applied.\n", hunk_index + 1).unwrap()
                }
                result.failures += remaining_hunks as u64;
                break
            }
            let end_index = hunk.chunk[before].start_index.apply_offset(current_offset);
            for line in &lines[lines_index..end_index] {
                result.lines.push(line.clone())
            }
            lines_index = end_index;
            result.lines.push(Rc::new(String::from("<<<<<<<")));
            let start_line = result.lines.len();
            for line in &lines[lines_index..lines_index + before_hlen] {
                result.lines.push(line.clone())
            }
            lines_index += before_hlen;
            result.lines.push(Rc::new(String::from("=======<")));
            for line in &hunk.chunk[after].lines[..hunk.len_minus_post_context(reverse)] {
                result.lines.push(line.clone())
            }
            result.lines.push(Rc::new(String::from(">>>>>>>")));
            let end_line = result.lines.len();
            if let Some(file_path) = repd_file_path {
                write!(err_w, "{:?}: Hunk #{} NOT MERGED at {}-{}.\n", file_path, hunk_index + 1, start_line, end_line).unwrap();
            } else {
                write!(err_w, "Hunk #{} NOT MERGED at {}-{}.\n", hunk_index + 1, start_line, end_line).unwrap();
            }
        }
        for line in &lines[lines_index..] {
            result.lines.push(line.clone());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
