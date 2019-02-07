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

extern crate lcs;
extern crate regex;

use std::slice::Iter;

pub mod abstract_diff;
pub mod context_diff;
pub mod diff;
pub mod diff_stats;
pub mod lines;
pub mod patch;
pub mod preamble;
pub mod text_diff;
pub mod unified_diff;

pub const TIMESTAMP_RE_STR: &str = r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}(\.\d{9})? [-+]{1}\d{4}";
pub const ALT_TIMESTAMP_RE_STR: &str =
    r"[A-Z][a-z]{2} [A-Z][a-z]{2} \d{2} \d{2}:\d{2}:\d{2} \d{4} [-+]{1}\d{4}";
pub const PATH_RE_STR: &str = r###""([^"]+)"|(\S+)"###;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DiffFormat {
    Unified,
    Context,
    GitBinary,
}

pub trait ApplyOffset {
    fn apply_offset(self, offset: i64) -> Self;
}

impl ApplyOffset for usize {
    fn apply_offset(self, offset: i64) -> usize {
        (self as i64 + offset) as usize
    }
}

pub struct MultiListIter<'a, T> {
    iters: Vec<Iter<'a, T>>,
    current_iter: usize,
}

impl<'a, T> MultiListIter<'a, T> {
    pub fn new(iters: Vec<Iter<'a, T>>) -> MultiListIter<T> {
        MultiListIter {
            iters: iters,
            current_iter: 0,
        }
    }

    pub fn push(&mut self, iter: Iter<'a, T>) {
        self.iters.push(iter);
    }

    pub fn prepend(&mut self, iter: Iter<'a, T>) {
        self.iters.insert(self.current_iter, iter);
    }

    pub fn append(&mut self, rhs: &mut MultiListIter<'a, T>) {
        for iter in rhs.iters.drain(rhs.current_iter..) {
            self.iters.push(iter)
        }
    }
}

impl<'a, T> Iterator for MultiListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_iter < self.iters.len() {
                if let Some(item) = self.iters[self.current_iter].next() {
                    return Some(item);
                }
            } else {
                break;
            };
            self.current_iter += 1
        }
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
