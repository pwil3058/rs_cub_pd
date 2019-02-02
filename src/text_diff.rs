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

use std::path::PathBuf;

use regex::Captures;

use crate::lines::*;
use crate::DiffFormat;

// TODO: implement Error for DiffParseError
#[derive(Debug)]
pub enum DiffParseError {
    MissingAfterFileData(usize),
}

pub type DiffParseResult<T> = Result<T, DiffParseError>;

#[derive(Debug, PartialEq, Clone)]
pub struct PathAndTimestamp {
    file_path: PathBuf,
    time_stamp: Option<String>,
}

pub struct TextDiffHeader {
    pub lines: Lines,
    pub ante_pat: PathAndTimestamp,
    pub post_pat: PathAndTimestamp
}

pub trait TextDiffHunk {
    fn num_lines(&self) -> usize;
}

pub struct TextDiff<H: TextDiffHunk> {
    pub lines_consumed: usize,
    pub diff_format: DiffFormat,
    pub header: TextDiffHeader,
    pub hunks: Vec<H>
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

    fn get_diff_at(&self, lines: Lines, start_index: usize) -> DiffParseResult<Option<TextDiff<H>>> {
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
                index += hunk.num_lines();
                hunks.push(hunk);
            } else {
                break
            }
        }
        let diff = TextDiff::<H> {
            lines_consumed: index - start_index,
            diff_format: self.diff_format(),
            header,
            hunks
        };
        Ok(Some(diff))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use regex::{Captures, Regex};

    use crate::{TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR, PATH_RE_STR};

    #[derive(Debug)]
    struct DummyDiffParser {
        ante_file_cre: Regex,
        post_file_cre: Regex,
    }

    impl TextDiffHunk for i32 {
        fn num_lines(&self) -> usize {
            0
        }
    }

    impl TextDiffParser<i32> for DummyDiffParser {
        fn new() -> Self {
            let e_ts_re_str = format!("({}|{})", TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR);
            let e = format!(r"^--- ({})(\s+{})?(.*)$", PATH_RE_STR, e_ts_re_str);
            let ante_file_cre = Regex::new(&e).unwrap();
            let e_ts_re_str = format!("({}|{})", TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR);
            let e = format!(r"^\+\+\+ ({})(\s+{})?(.*)$", PATH_RE_STR, e_ts_re_str);
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

        fn get_hunk_at(&self, _lines: &Lines, _index: usize) -> DiffParseResult<Option<i32>> {
            Ok(None)
        }
    }

    #[test]
    fn get_file_data_works() {
        let mut lines: Lines = Vec::new();
        lines.push(Line::new("--- /path/to/original".to_string()));
        lines.push(Line::new("+++ /path/to/new".to_string()));
        let ddp = DummyDiffParser::new();
        let tdh = ddp.get_text_diff_header_at(&lines, 0).unwrap().unwrap();
        assert_eq!(tdh.ante_pat, PathAndTimestamp{file_path: PathBuf::from("/path/to/original"), time_stamp: None});
        assert_eq!(tdh.post_pat, PathAndTimestamp{file_path: PathBuf::from("/path/to/new"), time_stamp: None});
    }
}
