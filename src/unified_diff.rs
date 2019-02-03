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
use std::sync::Arc;

use regex::{Captures, Regex};

use crate::{DiffFormat, PATH_RE_STR, TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR};
use crate::abstract_diff::{AbstractChunk, AbstractHunk};
use crate::lines::{Line, Lines};
use crate::text_diff::*;

pub struct UnifiedDiffChunk {
    start_line_num: usize,
    length: usize,
}

impl UnifiedDiffChunk {
    fn from_captures(captures: &Captures, line_num: usize, length: usize, line_number: usize) -> DiffParseResult<UnifiedDiffChunk> {
        let start_line_num: usize = if let Some(m) = captures.get(line_num) {
            usize::from_str(m.as_str()).map_err(|e| DiffParseError::ParseNumberError(e))?
        } else {
            return Err(DiffParseError::SyntaxError(DiffFormat::Unified, line_number))
        };
        let length: usize = if let Some(m) = captures.get(length) {
            usize::from_str(m.as_str()).map_err(|e| DiffParseError::ParseNumberError(e))?
        } else {
            1
        };
        Ok(UnifiedDiffChunk{start_line_num, length})
    }
}

pub struct UnifiedDiffHunk {
    pub lines: Lines,
    pub ante_chunk: UnifiedDiffChunk,
    pub post_chunk: UnifiedDiffChunk,
}

impl UnifiedDiffHunk {
    fn lines_not_starting_with(&self, bad_start: &str) -> Lines {
        let mut lines: Lines = vec![];
        for (index, ref line) in self.lines[1..].iter().enumerate() {
            if line.starts_with(bad_start) || line.starts_with("\\") {
                continue
            }
            if (index + 1) == self.lines.len() || !line.starts_with("\\") {
                lines.push(Arc::new(line[1..].to_string()));
            } else {
                lines.push(Arc::new(line[1..].trim_right_matches("\n").to_string()));
            }
        }
        lines
    }
}

impl TextDiffHunk for UnifiedDiffHunk {
    fn len(&self) -> usize {
        self.lines.len()
    }

    fn ante_lines(&self) -> Lines {
        self.lines_not_starting_with("+")
    }

    fn post_lines(&self) -> Lines {
        self.lines_not_starting_with("-")
    }

    fn get_abstract_diff_hunk(self) -> AbstractHunk {
        // NB: convert starting line numbers to 0 based indices
        // <https://www.gnu.org/software/diffutils/manual/html_node/Detailed-Unified.html#Detailed-Unified>
        // If a hunk contains just one line, only its start line number appears. Otherwise its line numbers
        // look like ‘start,count’. An empty hunk is considered to start at the line that follows the hunk.
        //
        // If a hunk and its context contain two or more lines, its line numbers look like ‘start,count’.
        // Otherwise only its end line number appears. An empty hunk is considered to end at the line that
        // precedes the hunk.

        let ante_lines = self.ante_lines();
        let post_lines = self.post_lines();
        let ante_chunk = AbstractChunk{
            start_index: if ante_lines.len() > 0 { self.ante_chunk.start_line_num - 1 } else { self.ante_chunk.start_line_num},
            lines: ante_lines,
        };
        let post_chunk = AbstractChunk{
            start_index: self.post_chunk.start_line_num - 1,
            lines: post_lines,
        };
        AbstractHunk::new(ante_chunk, post_chunk)
    }
}

struct UnifiedDiffParser {
    ante_file_cre: Regex,
    post_file_cre: Regex,
    hunk_data_cre: Regex,
}

impl TextDiffParser<UnifiedDiffHunk> for UnifiedDiffParser {
    fn new() -> Self {
        let e_ts_re_str = format!("({}|{})", TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR);

        let e = format!(r"(?s)^--- ({})(\s+{})?(.*)$", PATH_RE_STR, e_ts_re_str);
        let ante_file_cre = Regex::new(&e).unwrap();

        let e = format!(r"(?s)^\+\+\+ ({})(\s+{})?(.*)$", PATH_RE_STR, e_ts_re_str);
        let post_file_cre = Regex::new(&e).unwrap();

        let hunk_data_cre = Regex::new(r"(?s)^@@\s+-(\d+)(,(\d+))?\s+\+(\d+)(,(\d+))?\s+@@\s*(.*)$").unwrap();

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

    fn get_hunk_at(&self, lines: &Lines, start_index: usize) -> DiffParseResult<Option<UnifiedDiffHunk>> {
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
        let hunk = UnifiedDiffHunk {
            lines: lines[start_index..index].to_vec(),
            ante_chunk,
            post_chunk
        };
        Ok(Some(hunk))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use crate::lines::{Lines, LinesIfce};

    #[test]
    fn get_hunk_at_works() {
        let lines = Lines::read(&Path::new("test_diffs/test_1.diff")).unwrap();
        let parser = UnifiedDiffParser::new();
        let result = parser.get_diff_at(&lines, 0);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_some());

        let result = parser.get_diff_at(&lines, 14);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
