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

use regex::Regex;

use crate::lines::*;

// TODO: implement Error for DiffParseError
pub enum DiffParseError {
    MissingAfterFileData(usize),
}

#[derive(Debug, PartialEq)]
pub struct PathAndTimestamp {
    file_path: PathBuf,
    time_stamp: Option<String>,
}

pub trait TextDiffParser<H, D> {
    fn before_file_cre(&self) -> Regex;
    fn after_file_cre(&self) -> Regex;
    fn get_hunk_at(&self, lines: &Lines, index: usize) -> (Option<H>, usize);

    fn _get_file_data_at(&self, cre: &Regex, lines: &Lines, index: usize) -> (Option<PathAndTimestamp>, usize) {
        if let Some(captures) = cre.captures(&lines[index]) {
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
            return (Some(PathAndTimestamp{file_path, time_stamp}), index + 1)
        }
        (None, index)
    }

    fn get_before_file_data_at(&self, lines: &Lines, index: usize) -> (Option<PathAndTimestamp>, usize) {
        self._get_file_data_at(&self.before_file_cre(), lines, index)
    }

    fn get_after_file_data_at(&self, lines: &Lines, index: usize) -> (Option<PathAndTimestamp>, usize) {
        self._get_file_data_at(&self.after_file_cre(), lines, index)
    }

    fn get_diff_at(&self, lines: Lines, start_index: usize, fail_if_malformed: bool) -> Result<(Option<D>, usize), DiffParseError> {
        if lines.len() - start_index < 2 {
            return Ok((None, start_index))
        }
        // TODO: finish implementing get_diff_at()
        Ok((None, start_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const TIMESTAMP_RE_STR: &str = r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}(\.\d{9})? [-+]{1}\d{4}";
    const ALT_TIMESTAMP_RE_STR: &str = r"[A-Z][a-z]{2} [A-Z][a-z]{2} \d{2} \d{2}:\d{2}:\d{2} \d{4} [-+]{1}\d{4}";
    const PATH_RE_STR: &str = r###""([^"]+)"|(\S+)"###;

    #[derive(Debug, Default)]
    struct DummyDiffParser {}

    impl TextDiffParser<i32, i32> for DummyDiffParser {
        fn before_file_cre(&self) -> Regex {
            let e_ts_re_str = format!("({}|{})", TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR);
            let e = format!(r"^--- ({})(\s+{})?(.*)$", PATH_RE_STR, e_ts_re_str);
            Regex::new(&e).unwrap()
        }

        fn after_file_cre(&self) -> Regex {
            let e_ts_re_str = format!("({}|{})", TIMESTAMP_RE_STR, ALT_TIMESTAMP_RE_STR);
            let e = format!(r"^\+\+\+ ({})(\s+{})?(.*)$", PATH_RE_STR, e_ts_re_str);
            Regex::new(&e).unwrap()
        }

        fn get_hunk_at(&self, lines: &Lines, index: usize) -> (Option<i32>, usize) {
            (None, index)
        }
    }

    #[test]
    fn get_file_data_works() {
        let mut lines: Lines = Vec::new();
        lines.push(Line::new("--- /path/to/original".to_string()));
        lines.push(Line::new("+++ /path/to/new".to_string()));
        let ddp = DummyDiffParser::default();
        let index = 0;
        let (bfd, index) = ddp.get_before_file_data_at(&lines, index);
        assert_eq!(bfd, Some(PathAndTimestamp{file_path: PathBuf::from("/path/to/original"), time_stamp: None}));
        let (afd, _) = ddp.get_after_file_data_at(&lines, index);
        assert_eq!(afd, Some(PathAndTimestamp{file_path: PathBuf::from("/path/to/new"), time_stamp: None}));
    }
}
