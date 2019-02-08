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

const ENCODE: &[u8; 85] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";

struct Encoding {
    string: String,
    size: usize,
}

struct GitBase85 {
    decode_map: HashMap<char, usize>
}

impl GitBase85 {
    pub fn new() -> GitBase85 {
        let mut decode_map = HashMap::new();
        for (index, chr) in ENCODE.iter().enumerate() {
            decode_map.insert(*chr as char, index);
        }
        GitBase85 { decode_map }
    }

    fn decode(&self, encoding: Encoding) -> Result<Vec<u8>, String> {
        let mut data = Vec::<u8>::with_capacity(encoding.size);
        let d_index: usize = 0;
        let s_index:  usize = 0;
        while d_index < encoding.size {
            let acc: usize = 0;
            for _ in 0..5 {
                //if let Some(d) = self.decode_map.get(encoding.string[d_index]) {
                    //acc = acc * 85 + d;
                //} else {
                    //return Err("Illegal git base 85 character".to_string())
                //}
            }
        }
        Ok(data)
    }
//def decode(encoding):
    //assert is_consistent(encoding)
    //data = bytearray(encoding.size)
    //dindex = 0
    //sindex = 0
    //while dindex < encoding.size:
        //acc = 0
        //for _cnt in range(5):
            //try:
                //acc = acc * 85 + DECODE[encoding.string[sindex]]
            //except KeyError:
                //raise ParseError(_("Illegal git base 85 character"))
            //sindex += 1
        //if acc > _MAX_VAL:
            //raise RangeError(_("{0} too big.").format(acc))
        //for _cnt in range(4):
            //if dindex == encoding.size:
                //break
            //acc = (acc << 8) | (acc >> 24)
            //data[dindex] = acc % 256
            //dindex += 1
    //return data
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {

    }
}
