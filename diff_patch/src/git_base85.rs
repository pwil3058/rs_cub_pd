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

use std::collections::HashMap;

use crate::lines::Line;
use crate::text_diff::{DiffParseError, DiffParseResult};
use crate::DiffFormat;

const ENCODE: &[u8; 85] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";
const MAX_VAL: u64 = 0xFFFFFFFF;

pub struct Encoding {
    string: Vec<u8>,
    size: usize,
}

pub struct GitBase85 {
    decode_map: HashMap<u8, u64>,
}

impl GitBase85 {
    pub fn new() -> GitBase85 {
        let mut decode_map = HashMap::new();
        for (index, chr) in ENCODE.iter().enumerate() {
            decode_map.insert(*chr, index as u64);
        }
        GitBase85 { decode_map }
    }

    pub fn encode(&self, data: &[u8]) -> Encoding {
        let mut string: Vec<u8> = Vec::new();
        let mut index = 0;
        while index < data.len() {
            let mut acc: u64 = 0;
            for cnt in [24, 16, 8, 0].iter() {
                acc |= (data[index] as u64) << cnt;
                index += 1;
                if index == data.len() {
                    break;
                }
            }
            let mut snippet: Vec<u8> = Vec::new();
            for _ in 0..5 {
                let val = acc % 85;
                acc /= 85;
                snippet.insert(0, ENCODE[val as usize]);
            }
            string.append(&mut snippet);
        }
        Encoding {
            string: string,
            size: data.len(),
        }
    }

    fn decode(&self, encoding: &Encoding) -> DiffParseResult<Vec<u8>> {
        let mut data = vec![0u8; encoding.size];
        let mut d_index: usize = 0;
        let mut s_index: usize = 0;
        while d_index < encoding.size {
            let mut acc: u64 = 0;
            for _ in 0..5 {
                if s_index == encoding.string.len() {
                    break;
                }
                if let Some(ch) = encoding.string.get(s_index) {
                    if let Some(d) = self.decode_map.get(ch) {
                        acc = acc * 85 + d;
                    } else {
                        return Err(DiffParseError::Base85Error(
                            "Illegal git base 85 character".to_string(),
                        ));
                    }
                    s_index += 1;
                } else {
                    return Err(DiffParseError::Base85Error(format!(
                        "{0}: base85 source access out of range.",
                        s_index
                    )));
                }
            }
            if acc > MAX_VAL {
                return Err(DiffParseError::Base85Error(format!(
                    "{0}: base85 accumulator overflow.",
                    acc
                )));
            }
            for _ in 0..4 {
                if d_index == encoding.size {
                    break;
                }
                acc = (acc << 8) | (acc >> 24);
                data[d_index] = (acc % 256) as u8;
                d_index += 1;
            }
        }
        Ok(data)
    }

    pub fn decode_size(&self, ch: u8) -> DiffParseResult<usize> {
        if 'A' as u8 <= ch && ch <= 'Z' as u8 {
            Ok((ch - 'A' as u8) as usize)
        } else if 'a' as u8 <= ch && ch <= 'z' as u8 {
            Ok((ch - 'a' as u8 + 27) as usize)
        } else {
            Err(DiffParseError::UnexpectedInput(
                DiffFormat::GitBinary,
                format!("{}: expected char in range [azAZ]", ch as char),
            ))
        }
    }

    pub fn decode_line(&self, line: &Line) -> DiffParseResult<Vec<u8>> {
        let string = line.trim_right().as_bytes();
        let size = self.decode_size(string[0])?;
        let encoding = Encoding {
            string: string[1..].to_vec(),
            size,
        };
        Ok(self.decode(&encoding)?)
    }

    pub fn decode_lines(&self, lines: &[Line]) -> DiffParseResult<Vec<u8>> {
        let mut data: Vec<u8> = Vec::new();
        for line in lines.iter() {
            data.append(&mut self.decode_line(line)?);
        }
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test over a range of data sizes
    const TEST_DATA: &[u8] = b"uioyf2oyqo;3nhi8uydjauyo98ua 54\000jhkh\034hh;kjjh";

    #[test]
    fn git_base85_encode_decode_work() {
        let git_base85 = GitBase85::new();
        for i in 0..10 {
            let encoding = git_base85.encode(&TEST_DATA[i..]);
            let decoding = git_base85.decode(&encoding).unwrap();
            assert_eq!(decoding, TEST_DATA[i..].to_vec());
        }
    }
}
