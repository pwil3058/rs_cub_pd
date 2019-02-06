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

use std::collections::{hash_map, HashMap};
use std::path::PathBuf;
use std::slice::Iter;

use regex::Regex;

use crate::lines::{Line, Lines};
use crate::PATH_RE_STR;

pub trait PreambleIfce {
    fn len(&self) -> usize;
    fn iter(&self) -> Iter<Line>;
}

pub trait PreambleParser<P: PreambleIfce> {
    fn new() -> Self;
    fn get_preamble_at(&self, lines: &Lines, start_index: usize) -> Option<P>;
}

pub struct GitPreamble {
    lines: Lines,
    ante_file_path: String,
    post_file_path: String,
    extras: HashMap<String, (String, usize)>,
}

// TODO: should we be returning Path or &Path instead of PathBuf
impl GitPreamble {
    pub fn ante_file_path_as_str(&self) -> &str {
        self.ante_file_path.as_str()
    }

    pub fn post_file_path_as_str(&self) -> &str {
        self.post_file_path.as_str()
    }

    pub fn ante_file_path_buf(&self) -> PathBuf {
        self.ante_file_path.clone().into()
    }

    pub fn post_file_path_buf(&self) -> PathBuf {
        self.post_file_path.clone().into()
    }

    pub fn iter_extras(&self) -> hash_map::Iter<String, (String, usize)> {
        self.extras.iter()
    }

    pub fn get_extra(&self, name: &str) -> Option<&str> {
        match self.extras.get(name) {
            Some(extra) => Some(&extra.0),
            None => None,
        }
    }

    pub fn get_extra_line_index(&self, name: &str) -> Option<usize> {
        match self.extras.get(name) {
            Some(extra) => Some(extra.1),
            None => None,
        }
    }
}

impl PreambleIfce for GitPreamble {
    fn len(&self) -> usize {
        self.lines.len()
    }

    fn iter(&self) -> Iter<Line> {
        self.lines.iter()
    }
}

pub struct GitPreambleParser {
    diff_cre: Regex,
    extras_cres: Vec<Regex>,
}

impl PreambleParser<GitPreamble> for GitPreambleParser {
    fn new() -> GitPreambleParser {
        let diff_cre_str = format!(
            r"^diff\s+--git\s+({})\s+({})(\n)?$",
            PATH_RE_STR, PATH_RE_STR
        );
        let diff_cre = Regex::new(&diff_cre_str).unwrap();

        let extras_cres = [
            r"^(old mode)\s+(\d*)(\n)?$",
            r"^(new mode)\s+(\d*)(\n)?$",
            r"^(deleted file mode)\s+(\d*)(\n)?$",
            r"^(new file mode)\s+(\d*)(\n)?$",
            r"^(similarity index)\s+((\d*)%)(\n)?$",
            r"^(dissimilarity index)\s+((\d*)%)(\n)?$",
            r"^(index)\s+(([a-fA-F0-9]+)..([a-fA-F0-9]+)( (\d*))?)(\n)?$",
            &format!(r"^(copy from)\s+({})(\n)?$", PATH_RE_STR),
            &format!(r"^(copy to)\s+({0})(\n)?$", PATH_RE_STR),
            &format!(r"^(rename from)\s+({0})(\n)?$", PATH_RE_STR),
            &format!(r"^(rename to)\s+({0})(\n)?$", PATH_RE_STR),
        ]
        .iter()
        .map(|cre_str| Regex::new(cre_str).unwrap())
        .collect();

        GitPreambleParser {
            diff_cre,
            extras_cres,
        }
    }

    fn get_preamble_at(&self, lines: &Lines, start_index: usize) -> Option<GitPreamble> {
        let captures = if let Some(captures) = self.diff_cre.captures(&lines[start_index]) {
            captures
        } else {
            return None;
        };
        let ante_file_path = if let Some(path) = captures.get(3) {
            path.as_str().to_string()
        } else {
            captures.get(4).unwrap().as_str().to_string() // TODO: confirm unwrap is OK here
        };
        let post_file_path = if let Some(path) = captures.get(6) {
            path.as_str().to_string()
        } else {
            captures.get(7).unwrap().as_str().to_string() // TODO: confirm unwrap is OK here
        };

        let mut extras: HashMap<String, (String, usize)> = HashMap::new();
        for index in start_index + 1..lines.len() {
            let mut found = false;
            for cre in self.extras_cres.iter() {
                if let Some(captures) = cre.captures(&lines[index]) {
                    extras.insert(
                        captures.get(1).unwrap().as_str().to_string(),
                        (
                            captures.get(2).unwrap().as_str().to_string(),
                            index - start_index,
                        ),
                    );
                    found = true;
                    break;
                };
            }
            if !found {
                break;
            }
        }
        Some(GitPreamble {
            lines: lines[start_index..start_index + extras.len() + 1].to_vec(),
            ante_file_path,
            post_file_path,
            extras,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn it_works() {
        let mut lines: Lines = Vec::new();
        for s in &[
            "diff --git a/src/preamble.rs b/src/preamble.rs\n",
            "new file mode 100644\n",
            "index 0000000..0503e55\n",
        ] {
            lines.push(Arc::new(s.to_string()))
        }

        let parser = GitPreambleParser::new();

        let preamble = parser.get_preamble_at(&lines, 0);
        assert!(preamble.is_some());
        let preamble = preamble.unwrap();
        assert!(preamble.get_extra_line_index("index") == Some(2));
    }
}
