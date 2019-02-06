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

use std::fmt::{self, Display, Formatter};
use std::slice::Iter;

use crate::lines::{Line, Lines};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum DiffStatsCategory {
    Inserted = 0,
    Deleted = 1,
    Modified = 2,
    Unchanged = 3,
}

impl Display for DiffStatsCategory {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            DiffStatsCategory::Inserted => write!(f, "inserted"),
            DiffStatsCategory::Deleted => write!(f, "deleted"),
            DiffStatsCategory::Modified => write!(f, "modified"),
            DiffStatsCategory::Unchanged => write!(f, "unchanged"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct DiffStats {
    stats: [u64; 4],
}

impl DiffStats {
    pub fn count(&self, category: DiffStatsCategory) -> u64 {
        self.stats[category as usize]
    }

    pub fn incr_count(&mut self, category: DiffStatsCategory, by: u64) {
        self.stats[category as usize] += by
    }
}

pub struct DiffStatsLines {
    lines: Lines,
    stats: DiffStats,
}

impl DiffStatsLines {
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn iter(&self) -> Iter<Line> {
        self.lines.iter()
    }

    pub fn stats(&self) -> &DiffStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
