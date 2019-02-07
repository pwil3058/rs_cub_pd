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

use std::convert::From;
use std::slice::Iter;
use std::str::FromStr;

use lcs::{DiffComponent, LcsTable};
use regex::{Captures, Regex};

use crate::abstract_diff::{AbstractChunk, AbstractHunk};
use crate::lines::{Line, Lines};
use crate::text_diff::*;
use crate::{DiffFormat, ALT_TIMESTAMP_RE_STR, PATH_RE_STR, TIMESTAMP_RE_STR};

#[derive(Debug, Clone, Copy)]
pub struct UnifiedDiffChunk {
    start_line_num: usize,
    length: usize,
}

impl From<&AbstractChunk> for UnifiedDiffChunk {
    fn from(abstract_chunk: &AbstractChunk) -> Self {
        UnifiedDiffChunk {
            start_line_num: abstract_chunk.start_index + 1,
            length: abstract_chunk.lines.len(),
        }
    }
}

impl UnifiedDiffChunk {
    fn from_captures(
        captures: &Captures,
        line_num: usize,
        length: usize,
        line_number: usize,
    ) -> DiffParseResult<UnifiedDiffChunk> {
        let start_line_num: usize = if let Some(m) = captures.get(line_num) {
            usize::from_str(m.as_str())
                .map_err(|e| DiffParseError::ParseNumberError(e, line_number))?
        } else {
            return Err(DiffParseError::SyntaxError(
                DiffFormat::Unified,
                line_number,
            ));
        };
        let length: usize = if let Some(m) = captures.get(length) {
            usize::from_str(m.as_str())
                .map_err(|e| DiffParseError::ParseNumberError(e, line_number))?
        } else {
            1
        };
        Ok(UnifiedDiffChunk {
            start_line_num,
            length,
        })
    }
}

pub struct UnifiedDiffHunk {
    pub lines: Lines,
    pub ante_chunk: UnifiedDiffChunk,
    pub post_chunk: UnifiedDiffChunk,
}

pub type UnifiedDiff = TextDiff<UnifiedDiffHunk>;

impl TextDiffHunk for UnifiedDiffHunk {
    fn len(&self) -> usize {
        self.lines.len()
    }

    fn iter(&self) -> Iter<Line> {
        self.lines.iter()
    }

    fn ante_lines(&self) -> Lines {
        extract_source_lines(&self.lines[1..], 1, |l| l.starts_with("+"))
    }

    fn post_lines(&self) -> Lines {
        extract_source_lines(&self.lines[1..], 1, |l| l.starts_with("-"))
    }

    fn get_abstract_diff_hunk(&self) -> AbstractHunk {
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
        let ante_chunk = AbstractChunk {
            start_index: if ante_lines.len() > 0 {
                self.ante_chunk.start_line_num - 1
            } else {
                self.ante_chunk.start_line_num
            },
            lines: ante_lines,
        };
        let post_chunk = AbstractChunk {
            start_index: self.post_chunk.start_line_num - 1,
            lines: post_lines,
        };
        AbstractHunk::new(ante_chunk, post_chunk)
    }
}

//@@ -l,s +l,s @@ optional section heading
//
//The hunk range information contains two hunk ranges. The range for
//the hunk of the original file is preceded by a minus symbol, and
//the range for the new file is preceded by a plus symbol. Each hunk
//range is of the format l,s where l is the starting line number and
//s is the number of lines the change hunk applies to for each
//respective file. In many versions of GNU diff, each range can omit
//the comma and trailing value s, in which case s defaults to 1.
//Note that the only really interesting value is the l line number of
//the first range; all the other values can be computed from the diff.
//
//The hunk range for the original should be the sum of all contextual
//and deletion (including changed) hunk lines. The hunk range for
//the new file should be a sum of all contextual and addition
//(including changed) hunk lines. If hunk size information does not
//correspond with the number of lines in the hunk, then the diff
//could be considered invalid and be rejected.

//Optionally, the hunk range can be followed by the heading of the
//section or function that the hunk is part of. This is mainly useful
//to make the diff easier to read.
// TODO: check whether Gnu version is necessary for "patch" to work
fn hunk_header_line(
    ante_chunk: &UnifiedDiffChunk,
    post_chunk: &UnifiedDiffChunk,
    extra_text: Option<&str>,
) -> Line {
    let string = if let Some(extra_text) = extra_text {
        format!(
            "@@ -{},{} +{},{} @@ {}\n",
            ante_chunk.start_line_num,
            ante_chunk.length,
            post_chunk.start_line_num,
            post_chunk.length,
            extra_text,
        )
    } else {
        format!(
            "@@ -{},{} +{},{} @@\n",
            ante_chunk.start_line_num,
            ante_chunk.length,
            post_chunk.start_line_num,
            post_chunk.length,
        )
    };
    Line::new(string)
}

// TODO: add "extra string" to abstract text content
impl From<&AbstractHunk> for UnifiedDiffHunk {
    fn from(abstract_hunk: &AbstractHunk) -> Self {
        let abs_ante_chunk = abstract_hunk.ante_chunk();
        let ante_chunk = abs_ante_chunk.into();
        let abs_post_chunk = abstract_hunk.post_chunk();
        let post_chunk = abs_post_chunk.into();

        let mut lines = Vec::new();
        lines.push(hunk_header_line(&ante_chunk, &post_chunk, None));
        let lcs_table = LcsTable::new(&abs_ante_chunk.lines, &abs_post_chunk.lines);
        for diff_component in lcs_table.diff() {
            match diff_component {
                DiffComponent::Insertion(line) => lines.push(Line::new(format!("+{}", line))),
                DiffComponent::Unchanged(line, _) => lines.push(Line::new(format!(" {}", line))),
                DiffComponent::Deletion(line) => lines.push(Line::new(format!("-{}", line))),
            }
        }
        UnifiedDiffHunk {
            lines: lines,
            ante_chunk: ante_chunk,
            post_chunk: post_chunk,
        }
    }
}

pub struct UnifiedDiffParser {
    ante_file_cre: Regex,
    post_file_cre: Regex,
    hunk_data_cre: Regex,
}

impl TextDiffParser<UnifiedDiffHunk> for UnifiedDiffParser {
    fn new() -> Self {
        let e_ts_re_str = format!("({}|{})", TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR);

        let e = format!(r"^--- ({})(\s+{})?(.*)(\n)?$", PATH_RE_STR, e_ts_re_str);
        let ante_file_cre = Regex::new(&e).unwrap();

        let e = format!(r"^\+\+\+ ({})(\s+{})?(.*)(\n)?$", PATH_RE_STR, e_ts_re_str);
        let post_file_cre = Regex::new(&e).unwrap();

        let hunk_data_cre =
            Regex::new(r"^@@\s+-(\d+)(,(\d+))?\s+\+(\d+)(,(\d+))?\s+@@\s*(.*)(\n)?$").unwrap();

        UnifiedDiffParser {
            ante_file_cre,
            post_file_cre,
            hunk_data_cre,
        }
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

    fn get_hunk_at(
        &self,
        lines: &[Line],
        start_index: usize,
    ) -> DiffParseResult<Option<UnifiedDiffHunk>> {
        let captures = if let Some(captures) = self.hunk_data_cre.captures(&lines[start_index]) {
            captures
        } else {
            return Ok(None);
        };
        let ante_chunk = UnifiedDiffChunk::from_captures(&captures, 1, 3, start_index)?;
        let post_chunk = UnifiedDiffChunk::from_captures(&captures, 4, 6, start_index)?;
        let mut index = start_index + 1;
        let mut ante_count = 0;
        let mut post_count = 0;
        while ante_count < ante_chunk.length || post_count < post_chunk.length {
            if index >= lines.len() {
                return Err(DiffParseError::UnexpectedEndOfInput);
            }
            if lines[index].starts_with("-") {
                ante_count += 1
            } else if lines[index].starts_with("+") {
                post_count += 1
            } else if lines[index].starts_with(" ") {
                ante_count += 1;
                post_count += 1
            } else if !lines[index].starts_with("\\") {
                return Err(DiffParseError::UnexpectedEndHunk(
                    DiffFormat::Unified,
                    index,
                ));
            }
            index += 1
        }
        if index < lines.len() && lines[index].starts_with("\\") {
            index += 1
        }
        let hunk = UnifiedDiffHunk {
            lines: lines[start_index..index].to_vec(),
            ante_chunk,
            post_chunk,
        };
        Ok(Some(hunk))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lines::{Lines, LinesIfce};
    use std::path::Path;

    #[test]
    fn get_hunk_at_works() {
        let lines = Lines::read(&Path::new("../test_diffs/test_1.diff")).unwrap();
        let parser = UnifiedDiffParser::new();
        let result = parser.get_diff_at(&lines, 0);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_some());

        let result = parser.get_diff_at(&lines, 14);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let diff = result.unwrap();
        assert!(diff.iter().count() == diff.len());
    }
}
