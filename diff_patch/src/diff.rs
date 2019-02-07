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

use crate::context_diff::{ContextDiff, ContextDiffParser};
use crate::lines::Line;
use crate::preamble::{GitPreamble, Preamble, PreambleIfce, PreambleParser};
use crate::text_diff::{DiffParseResult, TextDiffParser};
use crate::unified_diff::{UnifiedDiff, UnifiedDiffParser};
use crate::MultiListIter;

pub enum Diff {
    Unified(UnifiedDiff),
    Context(ContextDiff),
    GitPreambleOnly(GitPreamble),
}

impl Diff {
    pub fn len(&self) -> usize {
        match self {
            Diff::Unified(diff) => diff.len(),
            Diff::Context(diff) => diff.len(),
            Diff::GitPreambleOnly(diff) => diff.len(),
        }
    }

    pub fn iter(&self) -> MultiListIter<Line> {
        match self {
            Diff::Unified(diff) => diff.iter(),
            Diff::Context(diff) => diff.iter(),
            Diff::GitPreambleOnly(diff) => MultiListIter::new(vec![diff.iter()]),
        }
    }
}

pub struct DiffParser {
    context_diff_parser: ContextDiffParser,
    unified_diff_parser: UnifiedDiffParser,
}

impl DiffParser {
    pub fn new() -> DiffParser {
        DiffParser {
            context_diff_parser: ContextDiffParser::new(),
            unified_diff_parser: UnifiedDiffParser::new(),
        }
    }

    pub fn get_diff_at(&self, lines: &[Line], start_index: usize) -> DiffParseResult<Option<Diff>> {
        // try diff types in occurence likelihood order
        if let Some(result) = self.unified_diff_parser.get_diff_at(lines, start_index)? {
            Ok(Some(Diff::Unified(result)))
        } else if let Some(result) = self.context_diff_parser.get_diff_at(lines, start_index)? {
            Ok(Some(Diff::Context(result)))
        } else {
            Ok(None)
        }
    }
}

pub struct DiffPlus {
    preamble: Option<Preamble>,
    diff: Diff,
}

impl DiffPlus {
    pub fn len(&self) -> usize {
        if let Some(ref preamble) = self.preamble {
            preamble.len() + self.diff.len()
        } else {
            self.len()
        }
    }

    pub fn iter(&self) -> MultiListIter<Line> {
        let mut iter = self.diff.iter();
        if let Some(preamble) = &self.preamble {
            iter.prepend(preamble.iter());
        };
        iter
    }

    pub fn preamble(&self) -> &Option<Preamble> {
        &self.preamble
    }

    pub fn diff(&self) -> &Diff {
        &self.diff
    }
}

pub struct DiffPlusParser {
    preamble_parser: PreambleParser,
    diff_parser: DiffParser,
}

impl DiffPlusParser {
    pub fn new() -> DiffPlusParser {
        DiffPlusParser {
            preamble_parser: PreambleParser::new(),
            diff_parser: DiffParser::new(),
        }
    }

    pub fn get_diff_plus_at(
        &self,
        lines: &[Line],
        start_index: usize,
    ) -> DiffParseResult<Option<DiffPlus>> {
        if let Some(preamble) = self.preamble_parser.get_preamble_at(lines, start_index) {
            if let Some(diff) = self
                .diff_parser
                .get_diff_at(lines, start_index + preamble.len())?
            {
                Ok(Some(DiffPlus {
                    preamble: Some(preamble),
                    diff,
                }))
            } else if let Preamble::Git(git_preamble) = preamble {
                Ok(Some(DiffPlus {
                    preamble: None,
                    diff: Diff::GitPreambleOnly(git_preamble),
                }))
            } else {
                Ok(None)
            }
        } else if let Some(diff) = self.diff_parser.get_diff_at(lines, start_index)? {
            Ok(Some(DiffPlus {
                preamble: None,
                diff,
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

    use crate::lines::*;

    #[test]
    fn get_diff_plus_at_works() {
        let lines = Lines::read(&Path::new("../test_diffs/test_1.diff")).unwrap();
        let parser = DiffPlusParser::new();
        let result = parser.get_diff_plus_at(&lines, 0);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        let result = parser.get_diff_plus_at(&lines, 12);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let diff = result.unwrap();
        assert!(diff.iter().count() == diff.len());
    }
}
