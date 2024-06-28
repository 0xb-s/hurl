/*
 * Hurl (https://hurl.dev)
 * Copyright (C) 2024 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use super::error::*;
use super::reader::Reader;
use super::{ParseFunc, ParseResult};

pub fn zero_or_more<T>(f: ParseFunc<T>, p: &mut Reader) -> ParseResult<Vec<T>> {
    let _start = p.state;

    let mut v: Vec<T> = Vec::new();
    loop {
        let initial_state = p.state;
        if p.is_eof() {
            return Ok(v);
        }

        match f(p) {
            Ok(r) => {
                v.push(r);
            }
            Err(e) => {
                return if e.recoverable {
                    p.state.pos = initial_state.pos;
                    p.state.cursor = initial_state.cursor;
                    Ok(v)
                } else {
                    Err(e)
                };
            }
        }
    }
}

/// Tries to apply the list of parser functions `fs` until one of them succeeds.
/// Typically this should be recoverable
pub fn choice<T>(fs: &[ParseFunc<T>], reader: &mut Reader) -> ParseResult<T> {
    for (pos, f) in fs.iter().enumerate() {
        let start = reader.state;
        if pos == fs.len() - 1 {
            return f(reader);
        }
        match f(reader) {
            Err(ParseError {
                recoverable: true, ..
            }) => {
                reader.state = start;
                continue;
            }
            x => return x,
        }
    }
    panic!("You can't call choice with an empty vector of choice")
}
