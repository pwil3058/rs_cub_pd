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

use std::str::FromStr;

use regex::{Captures, Regex};

use crate::{DiffFormat, PATH_RE_STR, TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR};
use crate::lines::{Line, Lines};
use crate::text_diff::*;

pub struct UnifiedDiffChunk {
    start_index: usize,
    length: usize,
}

impl UnifiedDiffChunk {
    fn from_captures(captures: &Captures, index: usize, length: usize, line_number: usize) -> DiffParseResult<UnifiedDiffChunk> {
        let start_index: usize = if let Some(m) = captures.get(index) {
            usize::from_str(m.as_str()).map_err(|e| DiffParseError::ParseNumberError(e))?
        } else {
            return Err(DiffParseError::SyntaxError(DiffFormat::Unified, line_number))
        };
        let length: usize = if let Some(m) = captures.get(length) {
            usize::from_str(m.as_str()).map_err(|e| DiffParseError::ParseNumberError(e))?
        } else {
            1
        };
        Ok(UnifiedDiffChunk{start_index, length})
    }
}

impl TextDiffChunk for UnifiedDiffChunk {
    fn start_index(&self) -> usize {
        self.start_index
    }
}

struct UnifiedDiffParser {
    ante_file_cre: Regex,
    post_file_cre: Regex,
    hunk_data_cre: Regex,
}

impl TextDiffParser<UnifiedDiffChunk> for UnifiedDiffParser {
    fn new() -> Self {
        let e_ts_re_str = format!("({}|{})", TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR);

        let e = format!(r"^--- ({})(\s+{})?(.*)$", PATH_RE_STR, e_ts_re_str);
        let ante_file_cre = Regex::new(&e).unwrap();

        let e = format!(r"^\+\+\+ ({})(\s+{})?(.*)$", PATH_RE_STR, e_ts_re_str);
        let post_file_cre = Regex::new(&e).unwrap();

        let hunk_data_cre = Regex::new(r"^@@\s+-(\d+)(,(\d+))?\s+\+(\d+)(,(\d+))?\s+@@\s*(.*)$").unwrap();

        UnifiedDiffParser{ante_file_cre, post_file_cre, hunk_data_cre}
    }

    fn diff_format(&self) -> DiffFormat {
        DiffFormat::Unified
    }

    fn ante_file_rec<'t>(&self, line: &'t Line) -> Option<Captures<'t>> {
        self.ante_file_cre.captures(line)
    }

    fn post_file_rec<'t>(&self, line: &'t Line) -> Option<Captures<'t>> {
        self.post_file_cre.captures(line)
    }

    fn get_hunk_at(&self, lines: &Lines, start_index: usize) -> DiffParseResult<Option<TextDiffHunk<UnifiedDiffChunk>>> {
        let captures = if let Some(captures) = self.hunk_data_cre.captures(&lines[start_index]) {
            captures
        } else {
            return Ok(None)
        };
        let ante_chunk = UnifiedDiffChunk::from_captures(&captures, 1, 3, start_index)?;
        let post_chunk = UnifiedDiffChunk::from_captures(&captures, 4, 6, start_index)?;
        let mut index = start_index + 1;
        let mut ante_count = 0;
        let mut post_count = 0;
        while ante_count < ante_chunk.length || post_count < post_chunk.length {
            if index >= lines.len() {
                return Err(DiffParseError::UnexpectedEndOfInput)
            }
            if lines[index].starts_with("-") {
                ante_count += 1
            } else if lines[index].starts_with("+") {
                post_count += 1
            } else if lines[index].starts_with(" ") {
                ante_count += 1;
                post_count += 1
            } else if !lines[index].starts_with("\\") {
                return Err(DiffParseError::UnexpectedEndHunk(DiffFormat::Unified, index))
            }
            index += 1
        };
        if index < lines.len() && lines[index].starts_with("\\") {
            index += 1
        }
        let hunk = TextDiffHunk::<UnifiedDiffChunk> {
            lines: lines[start_index..index].to_vec(),
            ante_chunk,
            post_chunk
        };
        Ok(Some(hunk))
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn get_hunk_at_works() {

    }
}
