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

use regex::Regex;

use crate::git_base85::GitBase85;
use crate::lines::{Line, Lines};
use crate::text_diff::{DiffParseError, DiffParseResult};
use crate::DiffFormat;

pub struct GitBinaryDiffData {}

impl GitBinaryDiffData {
    pub fn len(&self) -> usize {
        0
    }
}

pub struct GitBinaryDiff {
    lines: Lines,
    forward: GitBinaryDiffData,
    reverse: GitBinaryDiffData,
}

pub struct GitBinaryDiffParser {
    start_cre: Regex,
    data_start_cre: Regex,
    blank_line_cre: Regex,
    data_line_cre: Regex,
    git_base85: GitBase85,
    //START_CRE = re.compile(r"^GIT binary patch$")
    //DATA_START_CRE = re.compile(r"^(literal|delta) (\d+)$")
    //BLANK_LINE_CRE = re.compile(r"^\s*$")
    //DATA_LINE_CRE = re.compile("^([a-zA-Z])(([0-9a-zA-Z!#$%&()*+;<=>?@^_`{|}~-]{5})+)$")
}

impl GitBinaryDiffParser {
    pub fn new() -> GitBinaryDiffParser {
        GitBinaryDiffParser {
            start_cre: Regex::new(r"^GIT binary patch(\n)?$").unwrap(),
            data_start_cre: Regex::new(r"^(literal|delta) (\d+)(\n)?$").unwrap(),
            blank_line_cre: Regex::new(r"^\s*(\n)?$").unwrap(),
            data_line_cre: Regex::new(
                r"^([a-zA-Z])(([0-9a-zA-Z!#$%&()*+;<=>?@^_`{|}~-]{5})+)(\n)?$",
            )
            .unwrap(),
            git_base85: GitBase85::new(),
        }
    }

    fn get_data_at(
        &self,
        lines: &[Line],
        start_index: usize,
    ) -> DiffParseResult<GitBinaryDiffData> {
        let captures = if let Some(captures) = self.data_start_cre.captures(&lines[start_index]) {
            captures
        } else {
            return Err(DiffParseError::SyntaxError(
                DiffFormat::GitBinary,
                start_index + 1,
            ));
        };
        let method = captures.get(1).unwrap().as_str();
        let size = usize::from_str(captures.get(2).unwrap().as_str())
            .map_err(|e| DiffParseError::ParseNumberError(e, start_index + 1))?;
        let mut index = start_index + 1;
        while index < lines.len() && self.data_line_cre.is_match(&lines[index]) {
            index += 1;
        }
        let end_data = index;
        // absorb the blank line if there is one
        if index < lines.len() && self.blank_line_cre.is_match(&lines[index]) {
            index += 1;
        }
        //let data_zipped = self.git_base85.decode_lines(&lines[start_index..end_data])?;
        //dlines = lines[start_index:index]
        //try:
        //  data_zipped = gitbase85.decode_lines(lines[start_index + 1:end_data])
        //except AssertionError:
        //  raise DataError(_("Inconsistent git binary patch data."), lineno=start_index)
        //raw_size = len(zlib.decompress(bytes(data_zipped)))
        //if raw_size != size:
        //  emsg = _("Git binary patch expected {0} bytes. Got {1} bytes.").format(size, raw_size)
        //  raise DataError(emsg, lineno=start_index)
        //return (GitBinaryDiffData(dlines, method, raw_size, data_zipped), index)

        Ok(GitBinaryDiffData {})
    }

    pub fn get_diff_at(
        &self,
        lines: &[Line],
        start_index: usize,
    ) -> DiffParseResult<Option<GitBinaryDiff>> {
        if !self.start_cre.is_match(&lines[start_index]) {
            return Ok(None);
        }
        let mut index = start_index + 1;
        let forward = self.get_data_at(lines, index)?;
        index += forward.len();
        let reverse = self.get_data_at(lines, index)?;
        index += reverse.len();
        Ok(Some(GitBinaryDiff {
            lines: lines[start_index..index].to_vec(),
            forward,
            reverse,
        }))
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
