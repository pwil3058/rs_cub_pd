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

use std::io;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::slice::Iter;

use regex::Captures;

use crate::abstract_diff::{AbstractDiff, AbstractHunk, ApplnResult};
use crate::lines::*;
use crate::DiffFormat;
use crate::MultiListIter;

// TODO: implement Error for DiffParseError
#[derive(Debug)]
pub enum DiffParseError {
    MissingAfterFileData(usize),
    ParseNumberError(ParseIntError, usize),
    UnexpectedEndOfInput,
    UnexpectedEndHunk(DiffFormat, usize),
    SyntaxError(DiffFormat, usize),
}

pub type DiffParseResult<T> = Result<T, DiffParseError>;

#[derive(Debug, PartialEq, Clone)]
pub struct PathAndTimestamp {
    file_path: PathBuf,
    time_stamp: Option<String>,
}

#[derive(Debug)]
pub struct TextDiffHeader {
    pub lines: Lines,
    pub ante_pat: PathAndTimestamp,
    pub post_pat: PathAndTimestamp
}

pub trait TextDiffHunk {
    fn len(&self) -> usize;
    fn iter(&self) -> Iter<Line>;

    fn ante_lines(&self) -> Lines;
    fn post_lines(&self) -> Lines;

    fn get_abstract_diff_hunk(&self) -> AbstractHunk;
}

pub struct TextDiff<H: TextDiffHunk> {
    lines_consumed: Option<usize>, // time saver
    diff_format: DiffFormat,
    header: TextDiffHeader,
    hunks: Vec<H>,
}

impl<H> TextDiff<H> where H: TextDiffHunk {
    pub fn len(&mut self) -> usize {
        if let Some(length) = self.lines_consumed {
            length
        } else {
            let length = self.hunks.iter().fold(self.header.lines.len(), |n, h| n + h.len());
            self.lines_consumed = Some(length);
            length
        }
    }

    pub fn iter(&self) -> MultiListIter<Line> {
        let mut list = Vec::new();
        list.push(self.header.lines.iter());
        for hunk in self.hunks.iter() {
            list.push(hunk.iter())
        }
        MultiListIter::<Line>::new(list)
    }

    pub fn diff_format(&self) -> DiffFormat {
        self.diff_format
    }

    pub fn apply_to_lines<W>(&mut self, lines: &Lines, reverse: bool, err_w: &mut W, repd_file_path: Option<&Path>) -> ApplnResult
        where W: io::Write
    {
        let hunks = self.hunks.iter().map(|ref h| h.get_abstract_diff_hunk()).collect();
        let abstract_diff = AbstractDiff::new(hunks);
        abstract_diff.apply_to_lines(lines, reverse, err_w, repd_file_path)
    }
}

pub trait TextDiffParser<H: TextDiffHunk> {
    fn new() -> Self;
    fn diff_format(&self) -> DiffFormat;
    fn ante_file_rec<'t>(&self, line: &'t Line) -> Option<Captures<'t>>;
    fn post_file_rec<'t>(&self, line: &'t Line) -> Option<Captures<'t>>;
    fn get_hunk_at(&self, lines: &Lines, index: usize) -> DiffParseResult<Option<H>>;

    fn _get_file_data_fm_captures(&self, captures: &Captures) -> PathAndTimestamp {
        let file_path = if let Some(path) = captures.get(2) {
            path.as_str()
        } else {
            captures.get(3).unwrap().as_str() // TODO: confirm unwrap is OK here
        };
        let file_path = PathBuf::from(file_path);
        let time_stamp = if let Some(ts) = captures.get(4) {
            Some(ts.as_str().to_string())
        } else {
            None
        };
        PathAndTimestamp{file_path, time_stamp}
    }

    fn get_text_diff_header_at(&self, lines: &Lines, start_index: usize) -> DiffParseResult<Option<TextDiffHeader>> {
        let ante_pat = if let Some(ref captures) = self.ante_file_rec(&lines[start_index]) {
            self._get_file_data_fm_captures(captures)
        } else {
            return Ok(None)
        };
        let post_pat = if let Some(ref captures) = self.post_file_rec(&lines[start_index + 1]) {
            self._get_file_data_fm_captures(captures)
        } else {
            return Err(DiffParseError::MissingAfterFileData(start_index))
        };
        let lines = lines[start_index..start_index + 2].to_vec();
        Ok(Some(TextDiffHeader{lines, ante_pat, post_pat}))
    }

    fn get_diff_at(&self, lines: &Lines, start_index: usize) -> DiffParseResult<Option<TextDiff<H>>> {
        if lines.len() - start_index < 2 {
            return Ok(None)
        }
        let mut index = start_index;
        let header = if let Some(header) = self.get_text_diff_header_at(&lines, index)? {
            index += header.lines.len();
            header
        } else {
            return Ok(None)
        };
        let mut hunks: Vec<H> = Vec::new();
        while index < lines.len() {
            if let Some(hunk) = self.get_hunk_at(&lines, index)? {
                index += hunk.len();
                hunks.push(hunk);
            } else {
                break
            }
        }
        let diff = TextDiff::<H> {
            lines_consumed: None, //index - start_index,
            diff_format: self.diff_format(),
            header: header,
            hunks: hunks,
        };
        Ok(Some(diff))
    }
}

pub fn extract_source_lines<F: Fn(&Line) -> bool>(lines: &[Line], trim_left_n: usize, skip: F) -> Lines {
    let mut trimmed_lines: Lines = vec![];
    for (index, ref line) in lines.iter().enumerate() {
        if skip(line) || line.starts_with("\\") {
            continue
        }
        if (index + 1) == lines.len() || !lines[index + 1].starts_with("\\") {
            trimmed_lines.push(Arc::new(line[trim_left_n..].to_string()));
        } else {
            trimmed_lines.push(Arc::new(line[trim_left_n..].trim_right_matches("\n").to_string()));
        }
    }
    trimmed_lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use regex::{Captures, Regex};

    use crate::{TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR, PATH_RE_STR};
    use crate::abstract_diff::AbstractChunk;

    #[derive(Debug)]
    struct DummyDiffParser {
        ante_file_cre: Regex,
        post_file_cre: Regex,
    }

    struct DummyDiffHunk {
        lines: Lines
    }

    impl TextDiffHunk for DummyDiffHunk {
        fn len(&self) -> usize {
            self.lines.len()
        }

        fn iter(&self) -> Iter<Line> {
            self.lines.iter()
        }

        fn ante_lines(&self) -> Lines {
            vec![]
        }

        fn post_lines(&self) -> Lines {
            vec![]
        }

        fn get_abstract_diff_hunk(self) -> AbstractHunk {
            let a1 = AbstractChunk{start_index: 1, lines: Vec::<Line>::new()};
            let a2 = AbstractChunk{start_index: 1, lines: Vec::<Line>::new()};
            AbstractHunk::new(a1, a2)
        }
    }

    impl TextDiffParser<DummyDiffHunk> for DummyDiffParser {
        fn new() -> Self {
            let e_ts_re_str = format!("({}|{})", TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR);
            let e = format!(r"^--- ({})(\s+{})?(.*)(\n)?$", PATH_RE_STR, e_ts_re_str);
            let ante_file_cre = Regex::new(&e).unwrap();
            let e_ts_re_str = format!("({}|{})", TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR);
            let e = format!(r"^\+\+\+ ({})(\s+{})?(.*)(\n)?$", PATH_RE_STR, e_ts_re_str);
            let post_file_cre = Regex::new(&e).unwrap();
            DummyDiffParser{ante_file_cre, post_file_cre}
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

        fn get_hunk_at(&self, _lines: &Lines, _index: usize) -> DiffParseResult<Option<DummyDiffHunk>> {
            Ok(None)
        }
    }

    #[test]
    fn get_file_data_works() {
        let mut lines: Lines = Vec::new();
        lines.push(Line::new("--- a/path/to/original\n".to_string()));
        lines.push(Line::new("+++ b/path/to/new\n".to_string()));
        let ddp = DummyDiffParser::new();
        let tdh = ddp.get_text_diff_header_at(&lines, 0).unwrap().unwrap();
        assert_eq!(tdh.ante_pat, PathAndTimestamp{file_path: PathBuf::from("a/path/to/original"), time_stamp: None});
        assert_eq!(tdh.post_pat, PathAndTimestamp{file_path: PathBuf::from("b/path/to/new"), time_stamp: None});
    }
}
