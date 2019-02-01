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

extern crate regex;

pub mod abstract_diff;
pub mod lines;
pub mod text_diff;

pub const TIMESTAMP_RE_STR: &str = r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}(\.\d{9})? [-+]{1}\d{4}";
pub const ALT_TIMESTAMP_RE_STR: &str = r"[A-Z][a-z]{2} [A-Z][a-z]{2} \d{2} \d{2}:\d{2}:\d{2} \d{4} [-+]{1}\d{4}";
pub const PATH_RE_STR: &str = r###""([^"]+)"|(\S+)"###;

#[derive(Debug, PartialEq, Clone)]
pub enum DiffFormat {
    Unified,
    Context,
    GitBinary
}

pub trait ApplyOffset {
    fn apply_offset(self, offset: i64) -> Self;
}

impl ApplyOffset for usize {
    fn apply_offset(self, offset: i64) -> usize {
        (self as i64 + offset) as usize
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
