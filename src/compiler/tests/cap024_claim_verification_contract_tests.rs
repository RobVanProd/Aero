use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::ops::{Index, IndexMut};
use std::path::{Path, PathBuf};
use std::process::Command;

type Map = BTreeMap<String, Value>;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Null,
    Bool(bool),
    Number(i64),
    String(String),
    Array(Vec<Value>),
    Object(Map),
}

impl Value {
    fn as_object(&self) -> Option<&Map> {
        match self {
            Self::Object(value) => Some(value),
            _ => None,
        }
    }

    fn as_object_mut(&mut self) -> Option<&mut Map> {
        match self {
            Self::Object(value) => Some(value),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Self::Array(value) => Some(value),
            _ => None,
        }
    }

    fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Self::Array(value) => Some(value),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        self.as_i64().and_then(|value| value.try_into().ok())
    }

    fn get(&self, key: &str) -> Option<&Value> {
        self.as_object()?.get(key)
    }

    fn pointer(&self, pointer: &str) -> Option<&Value> {
        let mut current = self;
        for token in pointer.strip_prefix('/')?.split('/') {
            let token = token.replace("~1", "/").replace("~0", "~");
            current = match current {
                Self::Object(object) => object.get(&token)?,
                Self::Array(array) => array.get(token.parse::<usize>().ok()?)?,
                _ => return None,
            };
        }
        Some(current)
    }

    fn pointer_mut(&mut self, pointer: &str) -> Option<&mut Value> {
        let tokens: Vec<String> = pointer
            .strip_prefix('/')?
            .split('/')
            .map(|token| token.replace("~1", "/").replace("~0", "~"))
            .collect();
        let mut current = self;
        for token in tokens {
            current = match current {
                Self::Object(object) => object.get_mut(&token)?,
                Self::Array(array) => array.get_mut(token.parse::<usize>().ok()?)?,
                _ => return None,
            };
        }
        Some(current)
    }
}

impl Index<&str> for Value {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("JSON object key {index:?} missing"))
    }
}

impl IndexMut<&str> for Value {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.as_object_mut()
            .and_then(|object| object.get_mut(index))
            .unwrap_or_else(|| panic!("JSON object key {index:?} missing"))
    }
}

impl Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        &self
            .as_array()
            .unwrap_or_else(|| panic!("JSON value is not an array"))[index]
    }
}

impl IndexMut<usize> for Value {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self
            .as_array_mut()
            .unwrap_or_else(|| panic!("JSON value is not an array"))[index]
    }
}

trait IntoValue {
    fn into_value(self) -> Value;
}

impl IntoValue for Value {
    fn into_value(self) -> Value {
        self
    }
}

impl IntoValue for Map {
    fn into_value(self) -> Value {
        Value::Object(self)
    }
}

impl IntoValue for &Value {
    fn into_value(self) -> Value {
        self.clone()
    }
}

impl IntoValue for &str {
    fn into_value(self) -> Value {
        Value::String(self.to_owned())
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::String(self)
    }
}

impl IntoValue for bool {
    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}

macro_rules! integer_into_value {
    ($($kind:ty),+ $(,)?) => {
        $(impl IntoValue for $kind {
            fn into_value(self) -> Value {
                Value::Number(self as i64)
            }
        })+
    };
}

integer_into_value!(i32, i64, u32, u64, usize);

impl<T: IntoValue, const N: usize> IntoValue for [T; N] {
    fn into_value(self) -> Value {
        Value::Array(self.into_iter().map(IntoValue::into_value).collect())
    }
}

impl<T: IntoValue> IntoValue for Vec<T> {
    fn into_value(self) -> Value {
        Value::Array(self.into_iter().map(IntoValue::into_value).collect())
    }
}

fn into_value<T: IntoValue>(value: T) -> Value {
    value.into_value()
}

macro_rules! json {
    (null) => {
        Value::Null
    };
    ([$($element:expr),* $(,)?]) => {
        Value::Array(vec![$(into_value($element)),*])
    };
    ({$($key:literal : $value:expr),* $(,)?}) => {{
        let mut object = Map::new();
        $(object.insert($key.to_owned(), into_value($value));)*
        Value::Object(object)
    }};
    ($other:expr) => {
        into_value($other)
    };
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> JsonParser<'a> {
    fn parse(bytes: &'a [u8]) -> Result<Value, String> {
        std::str::from_utf8(bytes).map_err(|error| format!("JSON is not UTF-8: {error}"))?;
        let mut parser = Self { bytes, position: 0 };
        let value = parser.parse_value()?;
        parser.skip_whitespace();
        if parser.position != bytes.len() {
            return Err(format!("trailing JSON bytes at {}", parser.position));
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'n') => {
                self.literal(b"null")?;
                Ok(Value::Null)
            }
            Some(b't') => {
                self.literal(b"true")?;
                Ok(Value::Bool(true))
            }
            Some(b'f') => {
                self.literal(b"false")?;
                Ok(Value::Bool(false))
            }
            Some(b'\"') => self.parse_string().map(Value::String),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(Value::Number),
            Some(byte) => Err(format!(
                "unexpected JSON byte {byte:?} at {}",
                self.position
            )),
            None => Err("unexpected end of JSON".to_owned()),
        }
    }

    fn parse_array(&mut self) -> Result<Value, String> {
        self.expect(b'[')?;
        self.skip_whitespace();
        let mut values = Vec::new();
        if self.consume(b']') {
            return Ok(Value::Array(values));
        }
        loop {
            values.push(self.parse_value()?);
            self.skip_whitespace();
            if self.consume(b']') {
                return Ok(Value::Array(values));
            }
            self.expect(b',')?;
        }
    }

    fn parse_object(&mut self) -> Result<Value, String> {
        self.expect(b'{')?;
        self.skip_whitespace();
        let mut object = Map::new();
        if self.consume(b'}') {
            return Ok(Value::Object(object));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect(b':')?;
            let value = self.parse_value()?;
            if object.insert(key.clone(), value).is_some() {
                return Err(format!("duplicate JSON key {key:?}"));
            }
            self.skip_whitespace();
            if self.consume(b'}') {
                return Ok(Value::Object(object));
            }
            self.expect(b',')?;
        }
    }

    fn parse_number(&mut self) -> Result<i64, String> {
        let start = self.position;
        self.consume(b'-');
        if self.consume(b'0') {
            if self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                return Err(format!("leading zero at {start}"));
            }
        } else {
            let digits = self.position;
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.position += 1;
            }
            if self.position == digits {
                return Err(format!("missing JSON number at {start}"));
            }
        }
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(format!(
                "non-integral number is forbidden by this evidence contract at {start}"
            ));
        }
        std::str::from_utf8(&self.bytes[start..self.position])
            .unwrap()
            .parse()
            .map_err(|error| format!("invalid integer at {start}: {error}"))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect(b'\"')?;
        let mut value = String::new();
        loop {
            let byte = self.next().ok_or("unterminated JSON string")?;
            match byte {
                b'\"' => return Ok(value),
                b'\\' => {
                    let escape = self.next().ok_or("unterminated JSON escape")?;
                    match escape {
                        b'\"' => value.push('\"'),
                        b'\\' => value.push('\\'),
                        b'/' => value.push('/'),
                        b'b' => value.push('\u{0008}'),
                        b'f' => value.push('\u{000c}'),
                        b'n' => value.push('\n'),
                        b'r' => value.push('\r'),
                        b't' => value.push('\t'),
                        b'u' => value.push(self.parse_unicode_escape()?),
                        _ => return Err(format!("invalid JSON escape at {}", self.position)),
                    }
                }
                0..=0x1f => {
                    return Err(format!("control byte in JSON string at {}", self.position));
                }
                0x20..=0x7f => value.push(char::from(byte)),
                _ => {
                    let width = utf8_width(byte)
                        .ok_or_else(|| format!("invalid UTF-8 at {}", self.position - 1))?;
                    let start = self.position - 1;
                    self.position = self
                        .position
                        .checked_add(width - 1)
                        .filter(|end| *end <= self.bytes.len())
                        .ok_or("truncated UTF-8")?;
                    let text = std::str::from_utf8(&self.bytes[start..self.position])
                        .map_err(|error| format!("invalid UTF-8 string: {error}"))?;
                    value.push_str(text);
                }
            }
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, String> {
        let start = self.position;
        let end = start
            .checked_add(4)
            .filter(|end| *end <= self.bytes.len())
            .ok_or("truncated Unicode escape")?;
        self.position = end;
        let code = u16::from_str_radix(std::str::from_utf8(&self.bytes[start..end]).unwrap(), 16)
            .map_err(|error| format!("invalid Unicode escape: {error}"))?;
        if (0xd800..=0xdbff).contains(&code) {
            self.expect(b'\\')?;
            self.expect(b'u')?;
            let low_start = self.position;
            let low_end = low_start
                .checked_add(4)
                .filter(|end| *end <= self.bytes.len())
                .ok_or("truncated low surrogate")?;
            self.position = low_end;
            let low = u16::from_str_radix(
                std::str::from_utf8(&self.bytes[low_start..low_end]).unwrap(),
                16,
            )
            .map_err(|error| format!("invalid low surrogate: {error}"))?;
            if !(0xdc00..=0xdfff).contains(&low) {
                return Err("invalid low surrogate".to_owned());
            }
            let scalar = 0x1_0000 + ((u32::from(code) - 0xd800) << 10) + (u32::from(low) - 0xdc00);
            char::from_u32(scalar).ok_or_else(|| "invalid Unicode scalar".to_owned())
        } else {
            char::from_u32(u32::from(code)).ok_or_else(|| "invalid Unicode scalar".to_owned())
        }
    }

    fn literal(&mut self, literal: &[u8]) -> Result<(), String> {
        if self.bytes.get(self.position..self.position + literal.len()) == Some(literal) {
            self.position += literal.len();
            Ok(())
        } else {
            Err(format!("invalid JSON literal at {}", self.position))
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.position += 1;
        Some(byte)
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: u8) -> Result<(), String> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(format!("expected byte {expected:?} at {}", self.position))
        }
    }
}

fn utf8_width(lead: u8) -> Option<usize> {
    match lead {
        0xc2..=0xdf => Some(2),
        0xe0..=0xef => Some(3),
        0xf0..=0xf4 => Some(4),
        _ => None,
    }
}

fn canonical_json(value: &Value) -> String {
    fn write(value: &Value, output: &mut String) {
        match value {
            Value::Null => output.push_str("null"),
            Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
            Value::Number(value) => output.push_str(&value.to_string()),
            Value::String(value) => {
                output.push('\"');
                for character in value.chars() {
                    match character {
                        '\"' => output.push_str("\\\""),
                        '\\' => output.push_str("\\\\"),
                        '\n' => output.push_str("\\n"),
                        '\r' => output.push_str("\\r"),
                        '\t' => output.push_str("\\t"),
                        '\u{0008}' => output.push_str("\\b"),
                        '\u{000c}' => output.push_str("\\f"),
                        character if character < '\u{0020}' => {
                            output.push_str(&format!("\\u{:04x}", u32::from(character)));
                        }
                        character => output.push(character),
                    }
                }
                output.push('\"');
            }
            Value::Array(values) => {
                output.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                    }
                    write(value, output);
                }
                output.push(']');
            }
            Value::Object(object) => {
                output.push('{');
                for (index, (key, value)) in object.iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                    }
                    write(&Value::String(key.clone()), output);
                    output.push(':');
                    write(value, output);
                }
                output.push('}');
            }
        }
    }

    let mut output = String::new();
    write(value, &mut output);
    output
}

fn canonical_json_file_bytes(value: &Value) -> Vec<u8> {
    let mut bytes = canonical_json(value).into_bytes();
    bytes.push(b'\n');
    bytes
}

fn parse_canonical_json_file(bytes: &[u8], label: &str) -> Result<Value, String> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(format!("{label} contains a forbidden UTF-8 BOM"));
    }
    let value = parse_json(bytes, label)?;
    if canonical_json_file_bytes(&value) != bytes {
        return Err(format!(
            "{label} must be sorted compact canonical JSON followed by exactly one LF"
        ));
    }
    Ok(value)
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    fn sextet(byte: u8) -> Option<u8> {
        match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }

    let bytes = input.as_bytes();
    if bytes.len() % 4 != 0 {
        return Err("base64 length is not a multiple of four".to_owned());
    }
    let mut output = Vec::with_capacity(bytes.len() / 4 * 3);
    for (index, chunk) in bytes.chunks_exact(4).enumerate() {
        let final_chunk = index + 1 == bytes.len() / 4;
        let padding = match (chunk[2], chunk[3]) {
            (b'=', b'=') => 2,
            (_, b'=') => 1,
            (b'=', _) => return Err("base64 padding is noncanonical".to_owned()),
            _ => 0,
        };
        if padding > 0 && !final_chunk {
            return Err("base64 padding precedes the final chunk".to_owned());
        }
        let a = sextet(chunk[0]).ok_or("invalid base64 character")?;
        let b = sextet(chunk[1]).ok_or("invalid base64 character")?;
        let c = if padding == 2 {
            0
        } else {
            sextet(chunk[2]).ok_or("invalid base64 character")?
        };
        let d = if padding > 0 {
            0
        } else {
            sextet(chunk[3]).ok_or("invalid base64 character")?
        };
        if (padding == 2 && b & 0x0f != 0) || (padding == 1 && c & 0x03 != 0) {
            return Err("base64 padding has nonzero discarded bits".to_owned());
        }
        output.push((a << 2) | (b >> 4));
        if padding < 2 {
            output.push((b << 4) | (c >> 2));
        }
        if padding == 0 {
            output.push((c << 6) | d);
        }
    }
    Ok(output)
}

fn encode_base64(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(char::from(ALPHABET[usize::from(first >> 2)]));
        output.push(char::from(
            ALPHABET[usize::from(((first & 0x03) << 4) | (second >> 4))],
        ));
        if chunk.len() > 1 {
            output.push(char::from(
                ALPHABET[usize::from(((second & 0x0f) << 2) | (third >> 6))],
            ));
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(char::from(ALPHABET[usize::from(third & 0x3f)]));
        } else {
            output.push('=');
        }
    }
    output
}

fn sha256_hex(input: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state = [
        0x6a09e667_u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_length = (input.len() as u64).wrapping_mul(8);
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_length.to_be_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut words = [0_u32; 64];
        for (word, bytes) in words.iter_mut().take(16).zip(chunk.chunks_exact(4)) {
            *word = u32::from_be_bytes(bytes.try_into().unwrap());
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for index in 0..64 {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temporary1 = h
                .wrapping_add(sum1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temporary2 = sum0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temporary1);
            d = c;
            c = b;
            b = a;
            a = temporary1.wrapping_add(temporary2);
        }
        for (word, addition) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *word = word.wrapping_add(addition);
        }
    }
    let mut result = String::with_capacity(64);
    for word in state {
        result.push_str(&format!("{word:08x}"));
    }
    result
}

const AUTHORIZATION_HEAD: &str = "afd251d1f653649c0bf0ad6d000c62698fce840a";
const SUBJECT_COMMIT: &str = "918c9222eb61e2435e18847e30b946cd08013238";
const SUBJECT_TREE: &str = "aba2876644b0183ab877b2e28d5e14001328c99a";
const SUBJECT_PARENTS: [&str; 2] = [
    "e9b281504446465cfc8fcbe17c65cce92df0e83a",
    "d21c91fc312c70c47c6bb865ba1465e762255f0c",
];
const COMPILER_TREE: &str = "0ba0d06899b7e95d6b5b6f90a14804d18651806c";
const CLAIM_ID: &str = "aero_cap023_inference_correctness_918c9222_20260813";
const CLAIM_STATUS: &str = "verified_correctness_reproducibility_only";
const TOOL_ID: &str = "cap024-inference-evidence-v1";
const ORACLE_ID: &str = "cap023-relu-argmax-inference-oracle-v1";
const HISTORICAL_CLAIM_HEADER_SHA256: &str =
    "d05b49bb7993a2615367e0157df775d3defa841602a2b5474e0fc32a60950b32";
const HISTORICAL_CLAIM_SHA256: [&str; 6] = [
    "fef14027137f8e8724d18c205c6451ebb5b4e2048a84267d3410002f4fb706ce",
    "005f6889f8f53146e892f63459c6716683ae6829056f99b240f1089fdc7e5fc5",
    "afed36403272ff8ecc3dbb7b087716ec02a0a05cddd287c869b682a0caecccc1",
    "ba90802e201cfc9355894854e7e8249f4a87f7b5090b880faa9493da13f3b530",
    "bfe628f6c6966a61a34d667a27375133bbc61d32cac45e60b31395642c198e1d",
    "c359f9f16a8830a3f548ac1cd6c609b30d9f4399631cdec51fd5a2b5eba2976f",
];
const SCHEMA_ID: &str = "aero-cap023-inference-evidence-v1";
const BUNDLE_DIRECTORY: &str =
    "claim-verification/results/aero_cap023_inference_correctness_918c9222_20260813";
const SCHEMA_PATH: &str =
    "claim-verification/schemas/aero-cap023-inference-evidence-v1.schema.json";
const WORKFLOW_PATH: &str = ".github/workflows/cap023-evidence.yml";
const TOOL_PATH: &str = "tools/cap024_inference_evidence.py";
const CLAIMS_PATH: &str = "claim-verification/claims.json";

const ALLOWED_PATHS: [&str; 10] = [
    "TASK_LEDGER.md",
    WORKFLOW_PATH,
    TOOL_PATH,
    SCHEMA_PATH,
    "claim-verification/results/aero_cap023_inference_correctness_918c9222_20260813/manifest.json",
    "claim-verification/results/aero_cap023_inference_correctness_918c9222_20260813/oracle.json",
    "claim-verification/results/aero_cap023_inference_correctness_918c9222_20260813/REPRODUCE.md",
    CLAIMS_PATH,
    "src/compiler/tests/cap024_claim_verification_contract_tests.rs",
    "src/compiler/tests/cli_status_contract_tests.rs",
];
const REQUIRED_BUNDLE_FILES: [&str; 3] = ["REPRODUCE.md", "manifest.json", "oracle.json"];
const PLATFORM_NAMES: [&str; 2] = ["linux-x86_64", "windows-x86_64"];
const ARTIFACT_NAMES: [&str; 5] = [
    "llvm",
    "bitcode",
    "assembly",
    "executable_o0",
    "executable_o2",
];
const TOOL_NAMES: [&str; 7] = ["cargo", "rustc", "clang", "lld", "opt", "llvm-as", "llc"];
const MANIFEST_FIELDS: [&str; 18] = [
    "authorization_head",
    "claim_id",
    "failures",
    "inputs",
    "limitations",
    "oracle",
    "platforms",
    "replay",
    "reproduce",
    "schema",
    "schema_id",
    "schema_version",
    "scope",
    "subject",
    "support",
    "tool",
    "transport",
    "workflow",
];
const COMMAND_NAMES: [&str; 21] = [
    "clean_before",
    "compiler_build_first",
    "compiler_build_second",
    "aero_build_llvm_first",
    "aero_build_llvm_second",
    "llvm_verify_first",
    "llvm_verify_second",
    "llvm_assemble_first",
    "llvm_assemble_second",
    "machine_verify_first",
    "machine_verify_second",
    "link_o0_first",
    "link_o0_second",
    "link_o2_first",
    "link_o2_second",
    "native_o0_first",
    "native_o0_second",
    "native_o2_first",
    "native_o2_second",
    "public_run",
    "clean_after",
];
const REPLAY_EXCLUSIONS: [&str; 48] = [
    "/platforms/0/observations/runner_image",
    "/platforms/0/observations/kernel",
    "/platforms/0/compiler_executables/first/sha256",
    "/platforms/0/compiler_executables/first/size",
    "/platforms/0/compiler_executables/second/sha256",
    "/platforms/0/compiler_executables/second/size",
    "/platforms/0/commands/aero_build_llvm_first/stdout/base64",
    "/platforms/0/commands/aero_build_llvm_first/stdout/sha256",
    "/platforms/0/commands/aero_build_llvm_first/stdout/size",
    "/platforms/0/commands/aero_build_llvm_first/stderr/base64",
    "/platforms/0/commands/aero_build_llvm_first/stderr/sha256",
    "/platforms/0/commands/aero_build_llvm_first/stderr/size",
    "/platforms/0/commands/aero_build_llvm_second/stdout/base64",
    "/platforms/0/commands/aero_build_llvm_second/stdout/sha256",
    "/platforms/0/commands/aero_build_llvm_second/stdout/size",
    "/platforms/0/commands/aero_build_llvm_second/stderr/base64",
    "/platforms/0/commands/aero_build_llvm_second/stderr/sha256",
    "/platforms/0/commands/aero_build_llvm_second/stderr/size",
    "/platforms/0/commands/public_run/stdout/base64",
    "/platforms/0/commands/public_run/stdout/sha256",
    "/platforms/0/commands/public_run/stdout/size",
    "/platforms/0/commands/public_run/stderr/base64",
    "/platforms/0/commands/public_run/stderr/sha256",
    "/platforms/0/commands/public_run/stderr/size",
    "/platforms/1/observations/runner_image",
    "/platforms/1/observations/kernel",
    "/platforms/1/compiler_executables/first/sha256",
    "/platforms/1/compiler_executables/first/size",
    "/platforms/1/compiler_executables/second/sha256",
    "/platforms/1/compiler_executables/second/size",
    "/platforms/1/commands/aero_build_llvm_first/stdout/base64",
    "/platforms/1/commands/aero_build_llvm_first/stdout/sha256",
    "/platforms/1/commands/aero_build_llvm_first/stdout/size",
    "/platforms/1/commands/aero_build_llvm_first/stderr/base64",
    "/platforms/1/commands/aero_build_llvm_first/stderr/sha256",
    "/platforms/1/commands/aero_build_llvm_first/stderr/size",
    "/platforms/1/commands/aero_build_llvm_second/stdout/base64",
    "/platforms/1/commands/aero_build_llvm_second/stdout/sha256",
    "/platforms/1/commands/aero_build_llvm_second/stdout/size",
    "/platforms/1/commands/aero_build_llvm_second/stderr/base64",
    "/platforms/1/commands/aero_build_llvm_second/stderr/sha256",
    "/platforms/1/commands/aero_build_llvm_second/stderr/size",
    "/platforms/1/commands/public_run/stdout/base64",
    "/platforms/1/commands/public_run/stdout/sha256",
    "/platforms/1/commands/public_run/stdout/size",
    "/platforms/1/commands/public_run/stderr/base64",
    "/platforms/1/commands/public_run/stderr/sha256",
    "/platforms/1/commands/public_run/stderr/size",
];
const CHECKOUT_ACTION: &str = "actions/checkout@11d5960a326750d5838078e36cf38b85af677262";
const UPLOAD_ACTION: &str = "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02";
const DOWNLOAD_ACTION: &str = "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093";
const RUST_VERSION: &str = "1.97.1";
const RUST_COMMIT: &str = "8bab26f4f68e0e26f0bb7960be334d5b520ea452";
const FIXTURE_CARGO_COMMIT: &str = "c980f4866141969fab6254a680546a277789d6f0";
const LLVM_VERSION: &str = "22.1.8";
const EVIDENCE_PIN_ANCHORS: &[&str] = &[
    SUBJECT_COMMIT,
    SUBJECT_TREE,
    SUBJECT_PARENTS[0],
    SUBJECT_PARENTS[1],
    COMPILER_TREE,
    "examples/fixed_int_array_v0/relu_argmax_inference.aero",
    "5d5fe74e4acc351cb4326e85c4d69f320a37f3c6",
    "8244ca26fc90ce708801e12ec6a7192bdedfd01e1a1429c1479d36e233b1bb6c",
    "8224",
    ".github/workflows/rust.yml",
    "888a1d6b699725ebdd8b8fd6c762c1b58cd823a3",
    "32c820df765c6f42025d46a9f95049610fb8c301233f51920c7182fda74a92f5",
    "264585",
    "src/compiler/tests/fixed_int_array_profile_tests.rs",
    "959033d0fd255b947d16aa83efe914b517ced412",
    "6300d3e2a9ef51c270c9ea876a54e70be3fae0e55ccaab5bb81a060a36af5103",
    "257332",
    "src/compiler/Cargo.toml",
    "156dee0fc73aad0bf832c216edbfc9d13fb70012",
    "ee0ab0da24d5706101b37fdf94940fe863e097bcc02b0752b0bccaddf48ab96f",
    "1072",
    "src/compiler/Cargo.lock",
    "24c4729076801853f7bebb4a3269c050f31b3a5a",
    "076d1d4f06ed35627c45a93428aab3705fceafcada5f09ae1597ada6922ff280",
    "26063",
    RUST_VERSION,
    RUST_COMMIT,
    LLVM_VERSION,
    "LLVM-22.1.8-Linux-X64.tar.xz",
    "df0e1ecf16caf3489a272a5eea4eec9b0d82878f6477fa309504f918a0006384",
    "1938859476",
    "clang+llvm-22.1.8-x86_64-pc-windows-msvc.tar.xz",
    "d96c2cc1736f4eb7fa43cb9bbdf56d93551a9ae0a9aadb9c99c3c3b2b712a234",
    "862053924",
    "linux-start.S",
    "b95dbd79fd7b976862149e5635e148b9a9d2bbf20b2c3912a1f8d76c227379bb",
    "205",
    "windows-chkstk.S",
    "b971f9c51534aff82d774c26b6a6f2312a3beeac5e1710a69f3d88bd5671f376",
    "378",
];
const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
const TRACE_BASE64: &str = "ZGlhZ25vc3RpYwo=";
const TRACE_SHA256: &str = "d8bcbfa307f670d5532559fc030ee82ff7be48160c1fff837950e0d7528d0a4c";
const ALT_TRACE_BASE64: &str = "eA==";
const ALT_TRACE_SHA256: &str = "2d711642b726b04401627ca9fbac32f5c8530fb1903cc4db02258717921a4881";

#[derive(Clone, Copy)]
struct FrozenInput {
    path: &'static str,
    blob: &'static str,
    size: u64,
    sha256: &'static str,
}

const FROZEN_INPUTS: [FrozenInput; 5] = [
    FrozenInput {
        path: "examples/fixed_int_array_v0/relu_argmax_inference.aero",
        blob: "5d5fe74e4acc351cb4326e85c4d69f320a37f3c6",
        size: 8_224,
        sha256: "8244ca26fc90ce708801e12ec6a7192bdedfd01e1a1429c1479d36e233b1bb6c",
    },
    FrozenInput {
        path: ".github/workflows/rust.yml",
        blob: "888a1d6b699725ebdd8b8fd6c762c1b58cd823a3",
        size: 264_585,
        sha256: "32c820df765c6f42025d46a9f95049610fb8c301233f51920c7182fda74a92f5",
    },
    FrozenInput {
        path: "src/compiler/tests/fixed_int_array_profile_tests.rs",
        blob: "959033d0fd255b947d16aa83efe914b517ced412",
        size: 257_332,
        sha256: "6300d3e2a9ef51c270c9ea876a54e70be3fae0e55ccaab5bb81a060a36af5103",
    },
    FrozenInput {
        path: "src/compiler/Cargo.toml",
        blob: "156dee0fc73aad0bf832c216edbfc9d13fb70012",
        size: 1_072,
        sha256: "ee0ab0da24d5706101b37fdf94940fe863e097bcc02b0752b0bccaddf48ab96f",
    },
    FrozenInput {
        path: "src/compiler/Cargo.lock",
        blob: "24c4729076801853f7bebb4a3269c050f31b3a5a",
        size: 26_063,
        sha256: "076d1d4f06ed35627c45a93428aab3705fceafcada5f09ae1597ada6922ff280",
    },
];

const ORDINARY: [i32; 20] = [
    2, 3, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
];
const WRAPPING: [i32; 20] = [
    2,
    3,
    2,
    2,
    -3,
    5,
    i32::MAX,
    4,
    -2,
    i32::MIN,
    -1,
    3,
    i32::MAX,
    i32::MAX,
    2,
    7,
    i32::MIN,
    -3,
    13,
    -7,
];
const ACTIVATION: [i32; 20] = [
    2, 3, 2, 1, 1, 1, -1, -1, -1, 1, 0, -1, 2, 0, 1, 2, 3, 4, 5, 4,
];
const TIE: [i32; 20] = [2, 3, 2, 1, 2, 3, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 3, 0, 0, 0];
const MALFORMED_FIRST: [i32; 20] = [
    1, 3, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
];
const MALFORMED_SECOND: [i32; 20] = [
    2, 4, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
];
const MALFORMED_THIRD: [i32; 20] = [
    2, 3, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OracleComputation {
    first_products: [i32; 6],
    raw: [i32; 2],
    biased_hidden: [i32; 2],
    hidden: [i32; 2],
    second_products: [i32; 4],
    raw_logits: [i32; 2],
    logits: [i32; 2],
    result: [i32; 8],
}

fn reference_inference(record: [i32; 20]) -> OracleComputation {
    if record[..3] != [2, 3, 2] {
        return OracleComputation {
            first_products: [0; 6],
            raw: [0; 2],
            biased_hidden: [0; 2],
            hidden: [0; 2],
            second_products: [0; 4],
            raw_logits: [0; 2],
            logits: [0; 2],
            result: [0; 8],
        };
    }
    let input = [record[3], record[4], record[5]];
    let first_weights = [
        record[6], record[7], record[8], record[9], record[10], record[11],
    ];
    let first_bias = [record[12], record[13]];
    let second_weights = [record[14], record[15], record[16], record[17]];
    let second_bias = [record[18], record[19]];
    let first_products = [
        first_weights[0].wrapping_mul(input[0]),
        first_weights[1].wrapping_mul(input[1]),
        first_weights[2].wrapping_mul(input[2]),
        first_weights[3].wrapping_mul(input[0]),
        first_weights[4].wrapping_mul(input[1]),
        first_weights[5].wrapping_mul(input[2]),
    ];
    let raw = [
        first_products[..3]
            .iter()
            .fold(0_i32, |sum, value| sum.wrapping_add(*value)),
        first_products[3..]
            .iter()
            .fold(0_i32, |sum, value| sum.wrapping_add(*value)),
    ];
    let biased_hidden = [
        raw[0].wrapping_add(first_bias[0]),
        raw[1].wrapping_add(first_bias[1]),
    ];
    let hidden = biased_hidden.map(|value| if value > 0 { value } else { 0 });
    let second_products = [
        second_weights[0].wrapping_mul(hidden[0]),
        second_weights[1].wrapping_mul(hidden[1]),
        second_weights[2].wrapping_mul(hidden[0]),
        second_weights[3].wrapping_mul(hidden[1]),
    ];
    let raw_logits = [
        second_products[0].wrapping_add(second_products[1]),
        second_products[2].wrapping_add(second_products[3]),
    ];
    let logits = [
        raw_logits[0].wrapping_add(second_bias[0]),
        raw_logits[1].wrapping_add(second_bias[1]),
    ];
    let class = i32::from(logits[1] > logits[0]);
    OracleComputation {
        first_products,
        raw,
        biased_hidden,
        hidden,
        second_products,
        raw_logits,
        logits,
        result: [
            1, raw[0], raw[1], hidden[0], hidden[1], logits[0], logits[1], class,
        ],
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn git_output(root: &Path, arguments: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .map_err(|error| format!("cannot execute git {arguments:?}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {arguments:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(output.stdout)
}

fn validate_frozen_git_inputs(root: &Path) -> Result<(), String> {
    let commit = String::from_utf8(git_output(root, &["cat-file", "-p", SUBJECT_COMMIT])?)
        .map_err(|error| format!("subject commit is not UTF-8: {error}"))?;
    let mut lines = commit.lines();
    if lines.next() != Some(&format!("tree {SUBJECT_TREE}"))
        || lines.next() != Some(&format!("parent {}", SUBJECT_PARENTS[0]))
        || lines.next() != Some(&format!("parent {}", SUBJECT_PARENTS[1]))
    {
        return Err("accepted subject tree or ordered parents drifted".to_owned());
    }
    let compiler_entry = String::from_utf8(git_output(
        root,
        &["ls-tree", SUBJECT_COMMIT, "src/compiler"],
    )?)
    .map_err(|error| format!("compiler tree entry is not UTF-8: {error}"))?;
    if compiler_entry
        != format!(
            "040000 tree {COMPILER_TREE}\tsrc/compiler{}",
            if cfg!(windows) { "\r\n" } else { "\n" }
        )
        && compiler_entry != format!("040000 tree {COMPILER_TREE}\tsrc/compiler\n")
    {
        return Err(format!(
            "accepted compiler tree drifted: {compiler_entry:?}"
        ));
    }
    for input in FROZEN_INPUTS {
        let bytes = git_output(root, &["cat-file", "blob", input.blob])?;
        if bytes.len() as u64 != input.size || sha256_hex(&bytes) != input.sha256 {
            return Err(format!(
                "canonical Git blob bytes drifted for {}",
                input.path
            ));
        }
        let entry = String::from_utf8(git_output(root, &["ls-tree", SUBJECT_COMMIT, input.path])?)
            .map_err(|error| format!("{} tree entry is not UTF-8: {error}", input.path))?;
        if !entry.contains(&format!("blob {}\t{}", input.blob, input.path)) {
            return Err(format!(
                "{} is not bound to frozen blob {} at the subject",
                input.path, input.blob
            ));
        }
    }
    Ok(())
}

fn extract_source_record_literals(bytes: &[u8]) -> Result<Vec<[i32; 20]>, String> {
    let source =
        std::str::from_utf8(bytes).map_err(|error| format!("source is not UTF-8: {error}"))?;
    let names = [
        "ordinary_record",
        "wrapping_record",
        "activation_record",
        "tie_record",
        "malformed_first_record",
        "malformed_second_record",
        "malformed_third_record",
    ];
    let mut records = Vec::new();
    for name in names {
        let marker = format!("let {name}: [int; 20]");
        let declaration = source
            .split_once(&marker)
            .ok_or_else(|| format!("source omitted {name}"))?
            .1;
        let expression = declaration
            .split_once('=')
            .ok_or_else(|| format!("source {name} omitted ="))?
            .1;
        let body = expression
            .split_once('[')
            .and_then(|(_, value)| value.split_once("];").map(|(value, _)| value))
            .ok_or_else(|| format!("source {name} literal is not closed"))?;
        let values: Vec<i32> = body
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| {
                value
                    .parse::<i32>()
                    .map_err(|error| format!("source {name} lane {value:?}: {error}"))
            })
            .collect::<Result<_, _>>()?;
        records.push(
            values
                .try_into()
                .map_err(|values: Vec<i32>| format!("source {name} has {} lanes", values.len()))?,
        );
    }
    Ok(records)
}

fn validate_scope_paths<'a>(paths: impl IntoIterator<Item = &'a str>) -> Result<(), String> {
    let allowed: BTreeSet<&str> = ALLOWED_PATHS.into_iter().collect();
    let mut seen = BTreeSet::new();
    for path in paths {
        let normalized = path.replace('\\', "/");
        if normalized.starts_with(".codex-remote-attachments/") || normalized.starts_with("tmp/") {
            continue;
        }
        if !allowed.contains(normalized.as_str()) {
            return Err(format!(
                "cumulative CAP-024 diff crossed frozen scope at {normalized}"
            ));
        }
        seen.insert(normalized);
    }
    if !seen.contains("TASK_LEDGER.md")
        || !seen.contains("src/compiler/tests/cap024_claim_verification_contract_tests.rs")
    {
        return Err("cumulative CAP-024 diff omitted its ledger or red contract".to_owned());
    }
    Ok(())
}

fn validate_cumulative_git_scope(root: &Path) -> Result<(), String> {
    let mut paths: Vec<String> = String::from_utf8(git_output(
        root,
        &["diff", "--no-renames", "--name-only", SUBJECT_COMMIT, "--"],
    )?)
    .map_err(|error| format!("git diff paths are not UTF-8: {error}"))?
    .lines()
    .map(str::to_owned)
    .collect();
    paths.extend(
        String::from_utf8(git_output(
            root,
            &["ls-files", "--others", "--exclude-standard"],
        )?)
        .map_err(|error| format!("untracked paths are not UTF-8: {error}"))?
        .lines()
        .map(str::to_owned),
    );
    validate_scope_paths(paths.iter().map(String::as_str))
}

fn parse_json(bytes: &[u8], label: &str) -> Result<Value, String> {
    JsonParser::parse(bytes).map_err(|error| format!("{label} is not strict JSON: {error}"))
}

fn object<'a>(value: &'a Value, label: &str) -> Result<&'a Map, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{label} must be an object"))
}

fn array<'a>(value: &'a Value, label: &str) -> Result<&'a Vec<Value>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{label} must be an array"))
}

fn exact_keys(object: &Map, expected: &[&str], label: &str) -> Result<(), String> {
    let actual: BTreeSet<&str> = object.keys().map(String::as_str).collect();
    let expected: BTreeSet<&str> = expected.iter().copied().collect();
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "{label} keys differ: expected {expected:?}, received {actual:?}"
        ))
    }
}

fn exact_string(object: &Map, key: &str, expected: &str, label: &str) -> Result<(), String> {
    match object.get(key).and_then(Value::as_str) {
        Some(actual) if actual == expected => Ok(()),
        actual => Err(format!(
            "{label}.{key} expected {expected:?}, received {actual:?}"
        )),
    }
}

fn exact_u64(object: &Map, key: &str, expected: u64, label: &str) -> Result<(), String> {
    match object.get(key).and_then(Value::as_u64) {
        Some(actual) if actual == expected => Ok(()),
        actual => Err(format!(
            "{label}.{key} expected {expected}, received {actual:?}"
        )),
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn byte_record(size: u64, sha256: &str, base64: &str) -> Value {
    json!({"base64": base64, "sha256": sha256, "size": size})
}

fn byte_record_from_bytes(bytes: &[u8]) -> Value {
    json!({
        "base64": encode_base64(bytes),
        "sha256": sha256_hex(bytes),
        "size": bytes.len()
    })
}

fn executable_suffix(platform: &str) -> &'static str {
    if platform == "windows-x86_64" {
        ".exe"
    } else {
        ""
    }
}

fn artifact_path(platform: &str, production: &str, artifact: &str) -> String {
    let extension = match artifact {
        "llvm" => "ll",
        "bitcode" => "bc",
        "assembly" => "s",
        "executable_o0" => {
            return format!(
                "${{WORK}}/{platform}/{production}/inference-o0{}",
                executable_suffix(platform)
            );
        }
        "executable_o2" => {
            return format!(
                "${{WORK}}/{platform}/{production}/inference-o2{}",
                executable_suffix(platform)
            );
        }
        _ => panic!("unknown fixture artifact {artifact}"),
    };
    format!("${{WORK}}/{platform}/{production}/inference.{extension}")
}

fn compiler_path(platform: &str, production: &str) -> String {
    format!(
        "${{WORK}}/{platform}/cargo-{production}/release/aero{}",
        executable_suffix(platform)
    )
}

fn artifact_producer(artifact: &str, production: &str) -> String {
    match artifact {
        "llvm" => format!("aero_build_llvm_{production}"),
        "bitcode" => format!("llvm_assemble_{production}"),
        "assembly" => format!("machine_verify_{production}"),
        "executable_o0" => format!("link_o0_{production}"),
        "executable_o2" => format!("link_o2_{production}"),
        _ => panic!("unknown fixture artifact {artifact}"),
    }
}

fn artifact_pair(platform: &str, artifact: &str, seed: &str, size: u64) -> Value {
    let hash = format!("{seed:0<64}");
    let production = |name| {
        json!({
            "path": artifact_path(platform, name, artifact),
            "producer_command": artifact_producer(artifact, name),
            "sha256": hash.clone(),
            "size": size
        })
    };
    json!({"first": production("first"), "pair_equal": true, "second": production("second")})
}

fn command_inheritance(name: &str) -> &'static str {
    if matches!(
        name,
        "compiler_build_first"
            | "compiler_build_second"
            | "aero_build_llvm_first"
            | "aero_build_llvm_second"
            | "public_run"
    ) {
        "runner-substrate-observation-only"
    } else {
        "none"
    }
}

fn command_environment(platform: &str, name: &str) -> Value {
    let suffix = executable_suffix(platform);
    json!({
        "inheritance": command_inheritance(name),
        "overrides": json!({
            "CARGO_NET_OFFLINE": "true",
            "LC_ALL": "C",
            "RUSTC": format!("${{RUST}}/bin/rustc{suffix}"),
            "RUSTFLAGS": "-Awarnings",
            "TEMP": "${WORK}/tmp",
            "TMP": "${WORK}/tmp",
            "TZ": "UTC"
        }),
        "path_prefix": json!([format!("${{RUST}}/bin"), format!("${{LLVM}}/bin")]),
        "selectors": json!({
            "cargo": format!("${{RUST}}/bin/cargo{suffix}"),
            "clang": format!("${{LLVM}}/bin/clang{suffix}"),
            "llc": format!("${{LLVM}}/bin/llc{suffix}"),
            "lld": if platform == "windows-x86_64" {"${LLVM}/bin/lld-link.exe"} else {"${LLVM}/bin/ld.lld"},
            "llvm_as": format!("${{LLVM}}/bin/llvm-as{suffix}"),
            "opt": format!("${{LLVM}}/bin/opt{suffix}"),
            "rustc": format!("${{RUST}}/bin/rustc{suffix}")
        })
    })
}

fn expected_command_spec(
    platform: &str,
    name: &str,
) -> (Vec<String>, Vec<String>, Vec<String>, i64) {
    let suffix = executable_suffix(platform);
    let source = "${SUBJECT}/examples/fixed_int_array_v0/relu_argmax_inference.aero".to_owned();
    let support = if platform == "windows-x86_64" {
        "${WORK}/windows-x86_64/windows-chkstk.S"
    } else {
        "${WORK}/linux-x86_64/linux-start.S"
    }
    .to_owned();
    let cargo = format!("${{RUST}}/bin/cargo{suffix}");
    let clang = format!("${{LLVM}}/bin/clang{suffix}");
    let opt = format!("${{LLVM}}/bin/opt{suffix}");
    let llvm_as = format!("${{LLVM}}/bin/llvm-as{suffix}");
    let llc = format!("${{LLVM}}/bin/llc{suffix}");
    let internal = |phase: &str| vec!["internal".to_owned(), phase.to_owned(), platform.to_owned()];
    match name {
        "clean_before" => (internal("clean-before"), vec![], vec![], 0),
        "clean_after" => (internal("clean-after"), vec![], vec![], 0),
        "compiler_build_first" | "compiler_build_second" => {
            let production = name.strip_prefix("compiler_build_").unwrap();
            let output = compiler_path(platform, production);
            (
                vec![
                    cargo,
                    "build".into(),
                    "--quiet".into(),
                    "--locked".into(),
                    "--offline".into(),
                    "--release".into(),
                    "--bin".into(),
                    "aero".into(),
                    "--manifest-path".into(),
                    "${SUBJECT}/src/compiler/Cargo.toml".into(),
                    "--target-dir".into(),
                    format!("${{WORK}}/{platform}/cargo-{production}"),
                ],
                vec![
                    "${SUBJECT}/src/compiler".into(),
                    "${SUBJECT}/src/compiler/Cargo.toml".into(),
                    "${SUBJECT}/src/compiler/Cargo.lock".into(),
                ],
                vec![output],
                0,
            )
        }
        "aero_build_llvm_first" | "aero_build_llvm_second" => {
            let production = name.strip_prefix("aero_build_llvm_").unwrap();
            let compiler = compiler_path(platform, production);
            let llvm = artifact_path(platform, production, "llvm");
            (
                vec![
                    compiler.clone(),
                    "build".into(),
                    source.clone(),
                    "-o".into(),
                    llvm.clone(),
                    "--require-llvm-verifier".into(),
                    "--language-profile".into(),
                    "exact-i32-array-v0".into(),
                ],
                vec![compiler, source],
                vec![llvm],
                0,
            )
        }
        "llvm_verify_first" | "llvm_verify_second" => {
            let production = name.strip_prefix("llvm_verify_").unwrap();
            let llvm = artifact_path(platform, production, "llvm");
            (
                vec![
                    opt,
                    "-passes=verify".into(),
                    "-disable-output".into(),
                    llvm.clone(),
                ],
                vec![llvm],
                vec![],
                0,
            )
        }
        "llvm_assemble_first" | "llvm_assemble_second" => {
            let production = name.strip_prefix("llvm_assemble_").unwrap();
            let llvm = artifact_path(platform, production, "llvm");
            let bitcode = artifact_path(platform, production, "bitcode");
            (
                vec![llvm_as, llvm.clone(), "-o".into(), bitcode.clone()],
                vec![llvm],
                vec![bitcode],
                0,
            )
        }
        "machine_verify_first" | "machine_verify_second" => {
            let production = name.strip_prefix("machine_verify_").unwrap();
            let llvm = artifact_path(platform, production, "llvm");
            let assembly = artifact_path(platform, production, "assembly");
            (
                vec![
                    llc,
                    "-verify-machineinstrs".into(),
                    "-filetype=asm".into(),
                    llvm.clone(),
                    "-o".into(),
                    assembly.clone(),
                ],
                vec![llvm],
                vec![assembly],
                0,
            )
        }
        "link_o0_first" | "link_o0_second" | "link_o2_first" | "link_o2_second" => {
            let (optimization, production) = if let Some(value) = name.strip_prefix("link_o0_") {
                ("-O0", value)
            } else {
                ("-O2", name.strip_prefix("link_o2_").unwrap())
            };
            let llvm = artifact_path(platform, production, "llvm");
            let artifact = if optimization == "-O0" {
                "executable_o0"
            } else {
                "executable_o2"
            };
            let executable = artifact_path(platform, production, artifact);
            let mut argv = vec![
                clang,
                optimization.into(),
                llvm.clone(),
                support.clone(),
                "-o".into(),
                executable.clone(),
                "-nostdlib".into(),
            ];
            if platform == "windows-x86_64" {
                argv.extend([
                    "--ld-path=${LLVM}/bin/lld-link.exe".into(),
                    "-Wl,/entry:main,/subsystem:console,/nodefaultlib,/brepro".into(),
                ]);
            } else {
                argv.extend([
                    "--ld-path=${LLVM}/bin/ld.lld".into(),
                    "-Wl,-e,_start,--build-id=none".into(),
                ]);
            }
            (argv, vec![llvm, support], vec![executable], 0)
        }
        "native_o0_first" | "native_o0_second" | "native_o2_first" | "native_o2_second" => {
            let parts: Vec<&str> = name.split('_').collect();
            let artifact = format!("executable_{}", parts[1]);
            let executable = artifact_path(platform, parts[2], &artifact);
            (vec![executable.clone()], vec![executable], vec![], 91)
        }
        "public_run" => {
            let compiler = compiler_path(platform, "first");
            (
                vec![
                    compiler.clone(),
                    "run".into(),
                    source.clone(),
                    "--language-profile".into(),
                    "exact-i32-array-v0".into(),
                ],
                vec![compiler, source],
                vec![],
                91,
            )
        }
        _ => panic!("unknown command fixture {name}"),
    }
}

fn command_record(platform: &str, name: &str) -> Value {
    let (argv, consumes, produces, exit_code) = expected_command_spec(platform, name);
    let empty = byte_record(0, EMPTY_SHA256, "");
    let stdout = if name == "public_run" {
        byte_record_from_bytes(b"Aero execution diagnostic\nExit code: 91\n")
    } else if matches!(name, "aero_build_llvm_first" | "aero_build_llvm_second") {
        byte_record_from_bytes(b"diagnostic\n")
    } else {
        empty.clone()
    };
    json!({
        "argv": argv,
        "consumes": consumes,
        "cwd": "${SUBJECT}",
        "env": command_environment(platform, name),
        "exit_code": exit_code,
        "name": name,
        "produces": produces,
        "stderr": empty,
        "stdout": stdout
    })
}

fn fixture_oracle() -> Value {
    let records = [
        ("ordinary", ORDINARY),
        ("wrapping", WRAPPING),
        ("activation_boundary", ACTIVATION),
        ("tie", TIE),
        ("malformed_header_0", MALFORMED_FIRST),
        ("malformed_header_1", MALFORMED_SECOND),
        ("malformed_header_2", MALFORMED_THIRD),
    ];
    let records: Vec<Value> = records
        .into_iter()
        .map(|(name, source)| {
            let expected = reference_inference(source);
            json!({
                "biased_hidden": expected.biased_hidden,
                "first_products": expected.first_products,
                "header_valid": source[..3] == [2, 3, 2],
                "hidden": expected.hidden,
                "lane_count": 20,
                "logits": expected.logits,
                "name": name,
                "raw": expected.raw,
                "raw_logits": expected.raw_logits,
                "result": expected.result,
                "second_products": expected.second_products,
                "source": source,
                "source_after_call": source,
                "source_preserved": true
            })
        })
        .collect();
    json!({
        "arithmetic": "signed-i32-two-complement-wrapping",
        "header": json!([2, 3, 2]),
        "oracle_id": ORACLE_ID,
        "records": records,
        "rules": json!({
            "argmax": "signed-strict-greater-lower-index-tie",
            "header_gate": "exact-[2,3,2]-else-eight-zeros",
            "layout": "row-major-matvec-2x3-then-2x2",
            "logits": "two-wrapping-i32-biased-logits",
            "relu": "strict-positive-else-zero",
            "wrapping": "signed-i32-two-complement-every-mul-add"
        }),
        "sentinel": 91,
        "source": json!({"blob": FROZEN_INPUTS[0].blob, "path": FROZEN_INPUTS[0].path, "sha256": FROZEN_INPUTS[0].sha256, "size": FROZEN_INPUTS[0].size}),
        "source_preservation_lanes": 140,
        "version": 1
    })
}

fn fixture_tool_version_parsed(tool: &str) -> Value {
    match tool {
        "rustc" => json!({
            "banner_kind": "rustc-vv",
            "commit": RUST_COMMIT,
            "version": RUST_VERSION
        }),
        "cargo" => json!({
            "banner_kind": "cargo-vv",
            "version": RUST_VERSION
        }),
        "clang" => json!({
            "banner_kind": "clang",
            "version": LLVM_VERSION
        }),
        "lld" => json!({
            "banner_kind": "lld",
            "version": LLVM_VERSION
        }),
        "opt" | "llvm-as" | "llc" => json!({
            "banner_kind": "llvm",
            "version": LLVM_VERSION
        }),
        _ => panic!("unknown tool fixture {tool}"),
    }
}

fn fixture_tool_version_stdout(platform: &str, tool: &str) -> Vec<u8> {
    let host = if platform == "windows-x86_64" {
        "x86_64-pc-windows-msvc"
    } else {
        "x86_64-unknown-linux-gnu"
    };
    let text = match tool {
        "rustc" => format!(
            "rustc {RUST_VERSION} (8bab26f4f 2026-07-14)\nbinary: rustc\ncommit-hash: {RUST_COMMIT}\ncommit-date: 2026-07-14\nhost: {host}\nrelease: {RUST_VERSION}\nLLVM version: 22.1.6\n"
        ),
        "cargo" => format!(
            "cargo {RUST_VERSION} (c980f4866 2026-06-30)\nrelease: {RUST_VERSION}\ncommit-hash: {FIXTURE_CARGO_COMMIT}\ncommit-date: 2026-06-30\nhost: {host}\n"
        ),
        "clang" => format!(
            "clang version {LLVM_VERSION}\nTarget: {host}\nThread model: posix\nInstalledDir: ${{LLVM}}/bin\n"
        ),
        "lld" if platform == "linux-x86_64" => {
            format!("LLD {LLVM_VERSION} (compatible with GNU linkers)\n")
        }
        "lld" => format!("LLD {LLVM_VERSION}\n"),
        "opt" | "llvm-as" | "llc" => format!(
            "LLVM (https://llvm.org/):\n  LLVM version {LLVM_VERSION}\n  Optimized build.\n  Default target: {host}\n  Host CPU: generic\n"
        ),
        _ => panic!("unknown tool fixture {tool}"),
    };
    text.into_bytes()
}

fn fixture_platform(name: &str) -> Value {
    let mut artifacts = Map::new();
    let artifact_seed_offset = if name == "linux-x86_64" { 1 } else { 9 };
    for (index, artifact) in ARTIFACT_NAMES.into_iter().enumerate() {
        artifacts.insert(
            artifact.to_owned(),
            artifact_pair(
                name,
                artifact,
                &format!("{:x}", index + artifact_seed_offset),
                1_000 + index as u64,
            ),
        );
    }
    let mut commands = Map::new();
    for command in COMMAND_NAMES {
        commands.insert(command.to_owned(), command_record(name, command));
    }
    let tool_path = |tool: &str| match (name, tool) {
        ("windows-x86_64", "cargo" | "rustc") => format!("${{RUST}}/bin/{tool}.exe"),
        ("windows-x86_64", "lld") => "${LLVM}/bin/lld-link.exe".to_owned(),
        ("windows-x86_64", _) => format!("${{LLVM}}/bin/{tool}.exe"),
        (_, "cargo" | "rustc") => format!("${{RUST}}/bin/{tool}"),
        (_, "lld") => "${LLVM}/bin/ld.lld".to_owned(),
        (_, _) => format!("${{LLVM}}/bin/{tool}"),
    };
    let mut tools = Map::new();
    for (index, tool) in TOOL_NAMES.into_iter().enumerate() {
        let version_argv = match tool {
            "cargo" | "rustc" => vec![tool_path(tool), "-Vv".to_owned()],
            _ => vec![tool_path(tool), "--version".to_owned()],
        };
        let parsed_version = fixture_tool_version_parsed(tool);
        let version_stdout = fixture_tool_version_stdout(name, tool);
        tools.insert(
            tool.to_owned(),
            json!({
                "path": tool_path(tool),
                "payload_sha256": format!("{:x}{:0<63}", index + 1, name.len()),
                "payload_size": 10_000 + index,
                "version": json!({
                    "argv": version_argv,
                    "exit_code": 0,
                    "parsed": parsed_version,
                    "stderr": byte_record(0, EMPTY_SHA256, ""),
                    "stdout": byte_record_from_bytes(&version_stdout)
                })
            }),
        );
    }
    let empty = byte_record(0, EMPTY_SHA256, "");
    json!({
        "artifacts": artifacts,
        "commands": commands,
        "compiler_executables": json!({
            "first": json!({"path": compiler_path(name, "first"), "producer_command": "compiler_build_first", "sha256": format!("a{:0<63}", name.len()), "size": 123_456}),
            "second": json!({"path": compiler_path(name, "second"), "producer_command": "compiler_build_second", "sha256": format!("b{:0<63}", name.len()), "size": 123_457})
        }),
        "failures": json!([]),
        "name": name,
        "observations": json!({"kernel": "recorded-kernel", "runner_image": "recorded-image"}),
        "public_semantics": json!({
            "application_stderr": empty.clone(),
            "application_stdout": empty,
            "exit_report_count": 1,
            "reported_exit_code": 91
        }),
        "toolchain": json!({
            "archive_name": if name == "linux-x86_64" {"LLVM-22.1.8-Linux-X64.tar.xz"} else {"clang+llvm-22.1.8-x86_64-pc-windows-msvc.tar.xz"},
            "archive_sha256": if name == "linux-x86_64" {"df0e1ecf16caf3489a272a5eea4eec9b0d82878f6477fa309504f918a0006384"} else {"d96c2cc1736f4eb7fa43cb9bbdf56d93551a9ae0a9aadb9c99c3c3b2b712a234"},
            "archive_size": if name == "linux-x86_64" {1_938_859_476_u64} else {862_053_924_u64},
            "llvm_version": LLVM_VERSION,
            "rust_commit": RUST_COMMIT,
            "rust_version": RUST_VERSION,
            "setup_boundary": "workflow-acquisition-only; every final tool payload and version is verified before capture",
            "tools": tools
        })
    })
}

fn fixture_manifest() -> Value {
    let inputs: Vec<Value> = FROZEN_INPUTS
        .into_iter()
        .map(|input| json!({"blob": input.blob, "path": input.path, "sha256": input.sha256, "size": input.size}))
        .collect();
    json!({
        "authorization_head": AUTHORIZATION_HEAD,
        "claim_id": CLAIM_ID,
        "failures": json!([]),
        "inputs": inputs,
        "limitations": json!([
            "Correctness and reproducibility evidence only; byte sizes are footprint facts.",
            "Target artifacts reproduce only inside their stated platform and pinned-tool boundary.",
            "Runner and kernel identities are observations, not immutable inputs.",
            "No timing, resource-use, ABI, safety, accelerator, or general-inference claim."
        ]),
        "platforms": json!([fixture_platform("linux-x86_64"), fixture_platform("windows-x86_64")]),
        "replay": json!({
            "canonical_projection": "sorted-compact-json-plus-lf-v1",
            "excluded_paths": REPLAY_EXCLUSIONS,
            "fresh_observations": json!({
                "records": json!([]),
                "schema": "platform-plus-exact-pointer-value-records-v1",
                "transport": "temporary-actions-text-only-never-rewrites-accepted"
            })
        }),
        "reproduce": json!({"path": format!("{BUNDLE_DIRECTORY}/REPRODUCE.md"), "sha256": format!("6{:0<63}", 1)}),
        "schema": json!({"path": SCHEMA_PATH, "sha256": format!("5{:0<63}", 1)}),
        "scope": ALLOWED_PATHS,
        "schema_id": SCHEMA_ID,
        "schema_version": 1,
        "subject": json!({
            "clean_after": true,
            "clean_before": true,
            "commit": SUBJECT_COMMIT,
            "compiler_tree": COMPILER_TREE,
            "parents": SUBJECT_PARENTS,
            "tree": SUBJECT_TREE
        }),
        "support": json!({
            "linux": json!({"path": "linux-start.S", "sha256": "b95dbd79fd7b976862149e5635e148b9a9d2bbf20b2c3912a1f8d76c227379bb", "size": 205}),
            "windows": json!({"path": "windows-chkstk.S", "sha256": "b971f9c51534aff82d774c26b6a6f2312a3beeac5e1710a69f3d88bd5671f376", "size": 378})
        }),
        "transport": "temporary-actions-text-only",
        "workflow": json!({"path": WORKFLOW_PATH, "sha256": format!("2{:0<63}", 1)}),
        "tool": json!({"id": TOOL_ID, "path": TOOL_PATH, "sha256": format!("3{:0<63}", 1)}),
        "oracle": json!({"path": format!("{BUNDLE_DIRECTORY}/oracle.json"), "sha256": format!("4{:0<63}", 1)})
    })
}

fn string_schema() -> Value {
    json!({"type": "string"})
}

fn integer_schema() -> Value {
    json!({"minimum": 0, "type": "integer"})
}

fn closed_object_schema(entries: Vec<(&str, Value)>) -> Value {
    let required: Vec<String> = entries.iter().map(|(name, _)| (*name).to_owned()).collect();
    let properties: Map = entries
        .into_iter()
        .map(|(name, schema)| (name.to_owned(), schema))
        .collect();
    json!({
        "additionalProperties": false,
        "properties": properties,
        "required": required,
        "type": "object"
    })
}

fn array_schema(items: Value, minimum: u64, maximum: u64, unique: bool) -> Value {
    json!({
        "items": items,
        "maxItems": maximum,
        "minItems": minimum,
        "type": "array",
        "uniqueItems": unique
    })
}

fn fixture_schema() -> Value {
    let sha256 = json!({"pattern": "^[0-9a-f]{64}$", "type": "string"});
    let byte_record = closed_object_schema(vec![
        (
            "base64",
            json!({"contentEncoding": "base64", "type": "string"}),
        ),
        ("sha256", sha256.clone()),
        ("size", integer_schema()),
    ]);
    let artifact_record = closed_object_schema(vec![
        ("path", string_schema()),
        ("producer_command", string_schema()),
        ("sha256", sha256.clone()),
        ("size", integer_schema()),
    ]);
    let artifact_pair = closed_object_schema(vec![
        ("first", artifact_record.clone()),
        ("pair_equal", json!({"const": true})),
        ("second", artifact_record.clone()),
    ]);
    let command_env = closed_object_schema(vec![
        ("inheritance", string_schema()),
        (
            "overrides",
            closed_object_schema(vec![
                ("CARGO_NET_OFFLINE", string_schema()),
                ("LC_ALL", string_schema()),
                ("RUSTC", string_schema()),
                ("RUSTFLAGS", string_schema()),
                ("TEMP", string_schema()),
                ("TMP", string_schema()),
                ("TZ", string_schema()),
            ]),
        ),
        ("path_prefix", array_schema(string_schema(), 2, 2, true)),
        (
            "selectors",
            closed_object_schema(vec![
                ("cargo", string_schema()),
                ("clang", string_schema()),
                ("llc", string_schema()),
                ("lld", string_schema()),
                ("llvm_as", string_schema()),
                ("opt", string_schema()),
                ("rustc", string_schema()),
            ]),
        ),
    ]);
    let command = closed_object_schema(vec![
        ("argv", array_schema(string_schema(), 1, 64, false)),
        ("consumes", array_schema(string_schema(), 0, 16, true)),
        ("cwd", string_schema()),
        ("env", command_env),
        ("exit_code", json!({"type": "integer"})),
        ("name", string_schema()),
        ("produces", array_schema(string_schema(), 0, 4, true)),
        ("stderr", byte_record.clone()),
        ("stdout", byte_record.clone()),
    ]);
    let parsed_tool_version = json!({
        "oneOf": vec![
            closed_object_schema(vec![
                ("banner_kind", json!({"const": "rustc-vv"})),
                ("commit", json!({"const": RUST_COMMIT})),
                ("version", json!({"const": RUST_VERSION})),
            ]),
            closed_object_schema(vec![
                ("banner_kind", json!({"const": "cargo-vv"})),
                ("version", json!({"const": RUST_VERSION})),
            ]),
            closed_object_schema(vec![
                ("banner_kind", json!({"const": "clang"})),
                ("version", json!({"const": LLVM_VERSION})),
            ]),
            closed_object_schema(vec![
                ("banner_kind", json!({"const": "lld"})),
                ("version", json!({"const": LLVM_VERSION})),
            ]),
            closed_object_schema(vec![
                ("banner_kind", json!({"const": "llvm"})),
                ("version", json!({"const": LLVM_VERSION})),
            ]),
        ]
    });
    let tool_version = closed_object_schema(vec![
        ("argv", array_schema(string_schema(), 2, 2, false)),
        ("exit_code", json!({"const": 0})),
        ("parsed", parsed_tool_version),
        ("stderr", byte_record.clone()),
        ("stdout", byte_record.clone()),
    ]);
    let tool_record = closed_object_schema(vec![
        ("path", string_schema()),
        ("payload_sha256", sha256.clone()),
        ("payload_size", integer_schema()),
        ("version", tool_version),
    ]);
    let toolchain = closed_object_schema(vec![
        ("archive_name", string_schema()),
        ("archive_sha256", sha256.clone()),
        ("archive_size", integer_schema()),
        ("llvm_version", string_schema()),
        ("rust_commit", string_schema()),
        ("rust_version", string_schema()),
        ("setup_boundary", string_schema()),
        (
            "tools",
            closed_object_schema(
                TOOL_NAMES
                    .into_iter()
                    .map(|name| (name, tool_record.clone()))
                    .collect(),
            ),
        ),
    ]);
    let platform = closed_object_schema(vec![
        (
            "artifacts",
            closed_object_schema(
                ARTIFACT_NAMES
                    .into_iter()
                    .map(|name| (name, artifact_pair.clone()))
                    .collect(),
            ),
        ),
        (
            "commands",
            closed_object_schema(
                COMMAND_NAMES
                    .into_iter()
                    .map(|name| (name, command.clone()))
                    .collect(),
            ),
        ),
        (
            "compiler_executables",
            closed_object_schema(vec![
                ("first", artifact_record.clone()),
                ("second", artifact_record),
            ]),
        ),
        ("failures", array_schema(string_schema(), 0, 64, true)),
        ("name", string_schema()),
        (
            "observations",
            closed_object_schema(vec![
                ("kernel", string_schema()),
                ("runner_image", string_schema()),
            ]),
        ),
        (
            "public_semantics",
            closed_object_schema(vec![
                ("application_stderr", byte_record.clone()),
                ("application_stdout", byte_record.clone()),
                ("exit_report_count", json!({"const": 1})),
                ("reported_exit_code", json!({"const": 91})),
            ]),
        ),
        ("toolchain", toolchain),
    ]);
    let manifest_ref =
        closed_object_schema(vec![("path", string_schema()), ("sha256", sha256.clone())]);
    let tool_ref = closed_object_schema(vec![
        ("id", json!({"const": TOOL_ID})),
        ("path", string_schema()),
        ("sha256", sha256.clone()),
    ]);
    let input = closed_object_schema(vec![
        ("blob", string_schema()),
        ("path", string_schema()),
        ("sha256", sha256.clone()),
        ("size", integer_schema()),
    ]);
    let fresh_observation = closed_object_schema(vec![
        ("platform", json!({"enum": PLATFORM_NAMES})),
        ("pointer", string_schema()),
        ("value", json!({})),
    ]);
    let replay = closed_object_schema(vec![
        (
            "canonical_projection",
            json!({"const": "sorted-compact-json-plus-lf-v1"}),
        ),
        (
            "excluded_paths",
            array_schema(string_schema(), 48, 48, true),
        ),
        (
            "fresh_observations",
            closed_object_schema(vec![
                ("records", array_schema(fresh_observation, 0, 48, true)),
                ("schema", string_schema()),
                ("transport", string_schema()),
            ]),
        ),
    ]);
    let subject = closed_object_schema(vec![
        ("clean_after", json!({"const": true})),
        ("clean_before", json!({"const": true})),
        ("commit", json!({"const": SUBJECT_COMMIT})),
        ("compiler_tree", json!({"const": COMPILER_TREE})),
        ("parents", array_schema(string_schema(), 2, 2, true)),
        ("tree", json!({"const": SUBJECT_TREE})),
    ]);
    let support_record = closed_object_schema(vec![
        ("path", string_schema()),
        ("sha256", sha256.clone()),
        ("size", integer_schema()),
    ]);
    let support = closed_object_schema(vec![
        ("linux", support_record.clone()),
        ("windows", support_record),
    ]);
    let root_properties: Map = vec![
        (
            "authorization_head".to_owned(),
            json!({"const": AUTHORIZATION_HEAD}),
        ),
        ("claim_id".to_owned(), json!({"const": CLAIM_ID})),
        (
            "failures".to_owned(),
            array_schema(string_schema(), 0, 64, true),
        ),
        ("inputs".to_owned(), array_schema(input, 5, 5, true)),
        (
            "limitations".to_owned(),
            array_schema(string_schema(), 4, 32, true),
        ),
        ("oracle".to_owned(), manifest_ref.clone()),
        ("platforms".to_owned(), array_schema(platform, 2, 2, true)),
        ("replay".to_owned(), replay),
        ("reproduce".to_owned(), manifest_ref.clone()),
        ("schema".to_owned(), manifest_ref.clone()),
        ("schema_id".to_owned(), json!({"const": SCHEMA_ID})),
        ("schema_version".to_owned(), json!({"const": 1})),
        (
            "scope".to_owned(),
            array_schema(string_schema(), 10, 10, true),
        ),
        ("subject".to_owned(), subject),
        ("support".to_owned(), support),
        ("tool".to_owned(), tool_ref),
        (
            "transport".to_owned(),
            json!({"const": "temporary-actions-text-only"}),
        ),
        ("workflow".to_owned(), manifest_ref),
    ]
    .into_iter()
    .collect();
    let definitions: Map = vec![
        ("artifact_pair".to_owned(), artifact_pair),
        ("byte_record".to_owned(), byte_record),
        ("command".to_owned(), command),
    ]
    .into_iter()
    .collect();
    json!({
        "$defs": definitions,
        "$id": SCHEMA_ID,
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "additionalProperties": false,
        "properties": root_properties,
        "required": MANIFEST_FIELDS,
        "title": "Aero CAP-023 accepted-head inference evidence",
        "type": "object"
    })
}

fn validate_byte_record(value: &Value, label: &str) -> Result<(), String> {
    let record = object(value, label)?;
    exact_keys(record, &["base64", "sha256", "size"], label)?;
    let size = record
        .get("size")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{label}.size must be u64"))?;
    let hash = record
        .get("sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{label}.sha256 must be string"))?;
    let base64 = record
        .get("base64")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{label}.base64 must be string"))?;
    if !valid_sha256(hash) {
        return Err(format!("{label}.sha256 is not canonical lowercase SHA-256"));
    }
    let decoded = decode_base64(base64).map_err(|error| format!("{label}: {error}"))?;
    if decoded.len() as u64 != size {
        return Err(format!(
            "{label}.size {size} does not match {} decoded bytes",
            decoded.len()
        ));
    }
    if sha256_hex(&decoded) != hash {
        return Err(format!("{label}.sha256 does not match decoded bytes"));
    }
    Ok(())
}

fn validate_oracle(value: &Value) -> Result<(), String> {
    let oracle = object(value, "oracle")?;
    exact_keys(
        oracle,
        &[
            "arithmetic",
            "header",
            "oracle_id",
            "records",
            "rules",
            "sentinel",
            "source",
            "source_preservation_lanes",
            "version",
        ],
        "oracle",
    )?;
    exact_string(
        oracle,
        "arithmetic",
        "signed-i32-two-complement-wrapping",
        "oracle",
    )?;
    exact_string(oracle, "oracle_id", ORACLE_ID, "oracle")?;
    exact_u64(oracle, "sentinel", 91, "oracle")?;
    exact_u64(oracle, "source_preservation_lanes", 140, "oracle")?;
    exact_u64(oracle, "version", 1, "oracle")?;
    if oracle.get("header") != Some(&json!([2, 3, 2])) {
        return Err("oracle.header drifted".to_owned());
    }
    if oracle.get("source")
        != Some(&json!({
            "blob": FROZEN_INPUTS[0].blob,
            "path": FROZEN_INPUTS[0].path,
            "sha256": FROZEN_INPUTS[0].sha256,
            "size": FROZEN_INPUTS[0].size
        }))
    {
        return Err("oracle source Git identity drifted".to_owned());
    }
    if oracle.get("rules") != Some(&fixture_oracle()["rules"]) {
        return Err("oracle exact semantic rules drifted".to_owned());
    }
    let records = array(
        oracle.get("records").ok_or("oracle.records missing")?,
        "oracle.records",
    )?;
    if records.len() != 7 {
        return Err(format!(
            "oracle must contain seven records, received {}",
            records.len()
        ));
    }
    let expected = [
        ("ordinary", ORDINARY),
        ("wrapping", WRAPPING),
        ("activation_boundary", ACTIVATION),
        ("tie", TIE),
        ("malformed_header_0", MALFORMED_FIRST),
        ("malformed_header_1", MALFORMED_SECOND),
        ("malformed_header_2", MALFORMED_THIRD),
    ];
    let mut names = BTreeSet::new();
    for (index, ((name, source), actual)) in expected.into_iter().zip(records).enumerate() {
        let label = format!("oracle.records[{index}]");
        let actual = object(actual, &label)?;
        exact_keys(
            actual,
            &[
                "biased_hidden",
                "first_products",
                "header_valid",
                "hidden",
                "lane_count",
                "logits",
                "name",
                "raw",
                "raw_logits",
                "result",
                "second_products",
                "source",
                "source_after_call",
                "source_preserved",
            ],
            &label,
        )?;
        exact_string(actual, "name", name, &label)?;
        if !names.insert(name) {
            return Err(format!("duplicate oracle name {name}"));
        }
        let expected = reference_inference(source);
        for (field, value) in [
            ("source", json!(source)),
            ("source_after_call", json!(source)),
            ("source_preserved", json!(true)),
            ("lane_count", json!(20)),
            ("header_valid", json!(source[..3] == [2, 3, 2])),
            ("first_products", json!(expected.first_products)),
            ("raw", json!(expected.raw)),
            ("biased_hidden", json!(expected.biased_hidden)),
            ("hidden", json!(expected.hidden)),
            ("second_products", json!(expected.second_products)),
            ("raw_logits", json!(expected.raw_logits)),
            ("logits", json!(expected.logits)),
            ("result", json!(expected.result)),
        ] {
            if actual.get(field) != Some(&value) {
                return Err(format!(
                    "{label}.{field} drifted from independent wrapping oracle"
                ));
            }
        }
    }
    Ok(())
}

fn validate_command(value: &Value, platform: &str, expected_name: &str) -> Result<(), String> {
    let command = object(value, "command")?;
    exact_keys(
        command,
        &[
            "argv",
            "consumes",
            "cwd",
            "env",
            "exit_code",
            "name",
            "produces",
            "stderr",
            "stdout",
        ],
        "command",
    )?;
    let name = command
        .get("name")
        .and_then(Value::as_str)
        .ok_or("command.name missing")?;
    if name != expected_name {
        return Err(format!(
            "{platform} command key {expected_name:?} names {name:?}"
        ));
    }
    let (argv, consumes, produces, exit_code) = expected_command_spec(platform, expected_name);
    if command.get("argv") != Some(&json!(argv)) {
        return Err(format!(
            "{platform}.{name} argv drifted from the exact subprocess spec"
        ));
    }
    if command.get("consumes") != Some(&json!(consumes))
        || command.get("produces") != Some(&json!(produces))
    {
        return Err(format!("{platform}.{name} producer/consumer chain drifted"));
    }
    exact_string(command, "cwd", "${SUBJECT}", "command")?;
    if command.get("env") != Some(&command_environment(platform, expected_name)) {
        return Err(format!(
            "{platform}.{name} normalized explicit environment drifted"
        ));
    }
    validate_byte_record(
        command.get("stdout").ok_or("command.stdout missing")?,
        "command.stdout",
    )?;
    validate_byte_record(
        command.get("stderr").ok_or("command.stderr missing")?,
        "command.stderr",
    )?;
    let exit = command
        .get("exit_code")
        .and_then(Value::as_i64)
        .ok_or("command.exit_code missing")?;
    if exit != exit_code {
        return Err(format!(
            "{platform}.{name} exit expected {exit_code}, received {exit}"
        ));
    }
    if name.starts_with("native_o0_") || name.starts_with("native_o2_") {
        if exit != 91
            || command.get("stdout") != Some(&byte_record(0, EMPTY_SHA256, ""))
            || command.get("stderr") != Some(&byte_record(0, EMPTY_SHA256, ""))
        {
            return Err(format!(
                "{platform}.{name} must exit 91 with empty byte-exact streams"
            ));
        }
    }
    for trace_only in [
        "aero_build_llvm_first",
        "aero_build_llvm_second",
        "public_run",
    ] {
        if name == trace_only {
            let stdout_size = command
                .get("stdout")
                .and_then(Value::as_object)
                .and_then(|record| record.get("size"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let stderr_size = command
                .get("stderr")
                .and_then(Value::as_object)
                .and_then(|record| record.get("size"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            if stdout_size + stderr_size == 0 {
                return Err(format!(
                    "{platform}.{name} must retain its raw diagnostic streams"
                ));
            }
        }
    }
    Ok(())
}

fn expected_tool_path(platform: &str, tool: &str) -> String {
    match (platform, tool) {
        ("windows-x86_64", "cargo" | "rustc") => format!("${{RUST}}/bin/{tool}.exe"),
        ("windows-x86_64", "lld") => "${LLVM}/bin/lld-link.exe".to_owned(),
        ("windows-x86_64", _) => format!("${{LLVM}}/bin/{tool}.exe"),
        (_, "cargo" | "rustc") => format!("${{RUST}}/bin/{tool}"),
        (_, "lld") => "${LLVM}/bin/ld.lld".to_owned(),
        (_, _) => format!("${{LLVM}}/bin/{tool}"),
    }
}

fn exact_output_line(text: &str, expected: &str) -> bool {
    text.lines().any(|line| line == expected)
}

fn valid_git_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_tool_record(value: &Value, platform: &str, tool: &str) -> Result<(), String> {
    let record = object(value, "tool record")?;
    exact_keys(
        record,
        &["path", "payload_sha256", "payload_size", "version"],
        "tool record",
    )?;
    let path = expected_tool_path(platform, tool);
    exact_string(record, "path", &path, "tool record")?;
    if !valid_sha256(
        record
            .get("payload_sha256")
            .and_then(Value::as_str)
            .unwrap_or(""),
    ) || record
        .get("payload_size")
        .and_then(Value::as_u64)
        .is_none_or(|size| size == 0)
    {
        return Err(format!("{platform}.{tool} payload identity is incomplete"));
    }
    let version = object(
        record.get("version").ok_or("tool.version missing")?,
        "tool.version",
    )?;
    exact_keys(
        version,
        &["argv", "exit_code", "parsed", "stderr", "stdout"],
        "tool.version",
    )?;
    let expected_argv = if matches!(tool, "cargo" | "rustc") {
        json!([path, "-Vv"])
    } else {
        json!([path, "--version"])
    };
    if version.get("argv") != Some(&expected_argv) || version.get("exit_code") != Some(&json!(0)) {
        return Err(format!("{platform}.{tool} version probe is not exact"));
    }
    validate_byte_record(
        version.get("stdout").ok_or("tool.version.stdout missing")?,
        "tool.version.stdout",
    )?;
    validate_byte_record(
        version.get("stderr").ok_or("tool.version.stderr missing")?,
        "tool.version.stderr",
    )?;
    let parsed = version.get("parsed").ok_or("tool.version.parsed missing")?;
    if parsed != &fixture_tool_version_parsed(tool) {
        return Err(format!("{platform}.{tool} parsed version drifted"));
    }
    let stdout = decode_base64(version["stdout"]["base64"].as_str().unwrap())?;
    let stderr = decode_base64(version["stderr"]["base64"].as_str().unwrap())?;
    if !stderr.is_empty() {
        return Err(format!(
            "{platform}.{tool} version probe unexpectedly wrote stderr"
        ));
    }
    let stdout = String::from_utf8(stdout)
        .map_err(|_| format!("{platform}.{tool} version stdout is not UTF-8"))?;
    let derived = match tool {
        "rustc" => {
            let banner = format!("rustc {RUST_VERSION} (");
            let release = format!("release: {RUST_VERSION}");
            let commit = format!("commit-hash: {RUST_COMMIT}");
            stdout.lines().any(|line| line.starts_with(&banner))
                && exact_output_line(&stdout, &release)
                && exact_output_line(&stdout, &commit)
        }
        "cargo" => {
            let banner = format!("cargo {RUST_VERSION} (");
            let release = format!("release: {RUST_VERSION}");
            let cargo_commit = stdout
                .lines()
                .find_map(|line| line.strip_prefix("commit-hash: "));
            stdout.lines().any(|line| line.starts_with(&banner))
                && exact_output_line(&stdout, &release)
                && cargo_commit.is_some_and(valid_git_commit)
        }
        "clang" => stdout
            .lines()
            .any(|line| line.starts_with(&format!("clang version {LLVM_VERSION}"))),
        "lld" => {
            let banner = format!("LLD {LLVM_VERSION}");
            let gnu_banner = format!("{banner} (");
            stdout
                .lines()
                .any(|line| line == banner.as_str() || line.starts_with(&gnu_banner))
        }
        "opt" | "llvm-as" | "llc" => stdout
            .lines()
            .any(|line| line.trim() == format!("LLVM version {LLVM_VERSION}")),
        _ => false,
    };
    if !derived {
        return Err(format!(
            "{platform}.{tool} parsed version object is not exactly derived from its real raw banner"
        ));
    }
    Ok(())
}

fn validate_public_semantics(platform: &str, commands: &Map, value: &Value) -> Result<(), String> {
    let semantics = object(value, "public_semantics")?;
    exact_keys(
        semantics,
        &[
            "application_stderr",
            "application_stdout",
            "exit_report_count",
            "reported_exit_code",
        ],
        "public_semantics",
    )?;
    let empty = byte_record(0, EMPTY_SHA256, "");
    if semantics.get("application_stdout") != Some(&empty)
        || semantics.get("application_stderr") != Some(&empty)
    {
        return Err(format!(
            "{platform} application streams are not exact empty byte records"
        ));
    }
    exact_u64(semantics, "exit_report_count", 1, "public_semantics")?;
    exact_u64(semantics, "reported_exit_code", 91, "public_semantics")?;
    let public = object(
        commands.get("public_run").ok_or("public_run missing")?,
        "public_run",
    )?;
    let stdout_record = object(public.get("stdout").unwrap(), "public stdout")?;
    let stderr_record = object(public.get("stderr").unwrap(), "public stderr")?;
    let stdout = String::from_utf8(decode_base64(stdout_record["base64"].as_str().unwrap())?)
        .map_err(|_| format!("{platform} public stdout is not UTF-8"))?;
    let stderr = String::from_utf8(decode_base64(stderr_record["base64"].as_str().unwrap())?)
        .map_err(|_| format!("{platform} public stderr is not UTF-8"))?;
    let exit_lines = stdout
        .lines()
        .filter(|line| *line == "Exit code: 91")
        .count();
    let has_other_exit_line = stdout
        .lines()
        .any(|line| line.starts_with("Exit code:") && line != "Exit code: 91");
    let has_application_wrapper = |text: &str| {
        text.lines()
            .any(|line| line.starts_with("Output:") || line.starts_with("Error output:"))
    };
    let stderr_has_wrapper_status = stderr.lines().any(|line| {
        line.starts_with("Exit code:")
            || line.starts_with("Output:")
            || line.starts_with("Error output:")
    });
    if exit_lines != 1
        || has_other_exit_line
        || has_application_wrapper(&stdout)
        || stderr_has_wrapper_status
    {
        return Err(format!(
            "{platform} public semantics are not derivable from stdout-local exact wrapper lines"
        ));
    }
    Ok(())
}

fn validate_manifest(value: &Value) -> Result<(), String> {
    let manifest = object(value, "manifest")?;
    exact_keys(manifest, &MANIFEST_FIELDS, "manifest")?;
    exact_string(
        manifest,
        "authorization_head",
        AUTHORIZATION_HEAD,
        "manifest",
    )?;
    if manifest.get("scope") != Some(&json!(ALLOWED_PATHS)) {
        return Err("CAP-024 exact allowed path scope drifted".to_owned());
    }
    exact_string(manifest, "claim_id", CLAIM_ID, "manifest")?;
    exact_string(manifest, "schema_id", SCHEMA_ID, "manifest")?;
    exact_u64(manifest, "schema_version", 1, "manifest")?;
    exact_string(
        manifest,
        "transport",
        "temporary-actions-text-only",
        "manifest",
    )?;
    let subject = object(
        manifest.get("subject").ok_or("manifest.subject missing")?,
        "manifest.subject",
    )?;
    exact_keys(
        subject,
        &[
            "clean_after",
            "clean_before",
            "commit",
            "compiler_tree",
            "parents",
            "tree",
        ],
        "manifest.subject",
    )?;
    exact_string(subject, "commit", SUBJECT_COMMIT, "manifest.subject")?;
    exact_string(subject, "tree", SUBJECT_TREE, "manifest.subject")?;
    exact_string(subject, "compiler_tree", COMPILER_TREE, "manifest.subject")?;
    if subject.get("parents") != Some(&json!(SUBJECT_PARENTS))
        || subject.get("clean_before") != Some(&Value::Bool(true))
        || subject.get("clean_after") != Some(&Value::Bool(true))
    {
        return Err("manifest subject parents or clean-state proof drifted".to_owned());
    }
    let inputs = array(
        manifest.get("inputs").ok_or("manifest.inputs missing")?,
        "manifest.inputs",
    )?;
    if inputs.len() != FROZEN_INPUTS.len() {
        return Err("manifest input count drifted".to_owned());
    }
    for (index, (actual, expected)) in inputs.iter().zip(FROZEN_INPUTS).enumerate() {
        let label = format!("manifest.inputs[{index}]");
        let actual = object(actual, &label)?;
        exact_keys(actual, &["blob", "path", "sha256", "size"], &label)?;
        exact_string(actual, "path", expected.path, &label)?;
        exact_string(actual, "blob", expected.blob, &label)?;
        exact_string(actual, "sha256", expected.sha256, &label)?;
        exact_u64(actual, "size", expected.size, &label)?;
    }
    for key in ["workflow", "oracle", "reproduce", "schema"] {
        let record = object(
            manifest
                .get(key)
                .ok_or_else(|| format!("manifest.{key} missing"))?,
            &format!("manifest.{key}"),
        )?;
        exact_keys(record, &["path", "sha256"], &format!("manifest.{key}"))?;
        let hash = record
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("manifest.{key}.sha256 missing"))?;
        if !valid_sha256(hash) {
            return Err(format!("manifest.{key}.sha256 malformed"));
        }
    }
    let tool_record = object(
        manifest.get("tool").ok_or("manifest.tool missing")?,
        "manifest.tool",
    )?;
    exact_keys(tool_record, &["id", "path", "sha256"], "manifest.tool")?;
    exact_string(tool_record, "id", TOOL_ID, "manifest.tool")?;
    if !valid_sha256(
        tool_record
            .get("sha256")
            .and_then(Value::as_str)
            .unwrap_or(""),
    ) {
        return Err("manifest.tool.sha256 malformed".to_owned());
    }
    exact_string(
        object(manifest.get("workflow").unwrap(), "workflow")?,
        "path",
        WORKFLOW_PATH,
        "workflow",
    )?;
    exact_string(
        object(manifest.get("tool").unwrap(), "tool")?,
        "path",
        TOOL_PATH,
        "tool",
    )?;
    exact_string(
        object(manifest.get("oracle").unwrap(), "oracle")?,
        "path",
        &format!("{BUNDLE_DIRECTORY}/oracle.json"),
        "oracle",
    )?;
    exact_string(
        object(manifest.get("schema").unwrap(), "schema")?,
        "path",
        SCHEMA_PATH,
        "schema",
    )?;
    exact_string(
        object(manifest.get("reproduce").unwrap(), "reproduce")?,
        "path",
        &format!("{BUNDLE_DIRECTORY}/REPRODUCE.md"),
        "reproduce",
    )?;
    let support = object(
        manifest.get("support").ok_or("manifest.support missing")?,
        "manifest.support",
    )?;
    exact_keys(support, &["linux", "windows"], "manifest.support")?;
    for (name, path, size, hash) in [
        (
            "linux",
            "linux-start.S",
            205,
            "b95dbd79fd7b976862149e5635e148b9a9d2bbf20b2c3912a1f8d76c227379bb",
        ),
        (
            "windows",
            "windows-chkstk.S",
            378,
            "b971f9c51534aff82d774c26b6a6f2312a3beeac5e1710a69f3d88bd5671f376",
        ),
    ] {
        let record = object(
            support.get(name).ok_or("support record missing")?,
            "support",
        )?;
        exact_keys(record, &["path", "sha256", "size"], "support")?;
        exact_string(record, "path", path, "support")?;
        exact_string(record, "sha256", hash, "support")?;
        exact_u64(record, "size", size, "support")?;
    }
    let replay = object(
        manifest.get("replay").ok_or("manifest.replay missing")?,
        "manifest.replay",
    )?;
    exact_keys(
        replay,
        &[
            "canonical_projection",
            "excluded_paths",
            "fresh_observations",
        ],
        "manifest.replay",
    )?;
    exact_string(
        replay,
        "canonical_projection",
        "sorted-compact-json-plus-lf-v1",
        "manifest.replay",
    )?;
    if replay.get("excluded_paths") != Some(&json!(REPLAY_EXCLUSIONS)) {
        return Err("closed replay exclusion list drifted".to_owned());
    }
    if replay.get("fresh_observations")
        != Some(&json!({
            "records": json!([]),
            "schema": "platform-plus-exact-pointer-value-records-v1",
            "transport": "temporary-actions-text-only-never-rewrites-accepted"
        }))
    {
        return Err("fresh replay observation transport shape drifted".to_owned());
    }
    let platforms = array(
        manifest
            .get("platforms")
            .ok_or("manifest.platforms missing")?,
        "manifest.platforms",
    )?;
    if platforms.len() != 2 {
        return Err(format!(
            "manifest requires two platforms, received {}",
            platforms.len()
        ));
    }
    for (platform_index, value) in platforms.iter().enumerate() {
        let platform = object(value, "platform")?;
        exact_keys(
            platform,
            &[
                "artifacts",
                "commands",
                "compiler_executables",
                "failures",
                "name",
                "observations",
                "public_semantics",
                "toolchain",
            ],
            "platform",
        )?;
        let name = platform
            .get("name")
            .and_then(Value::as_str)
            .ok_or("platform.name missing")?;
        if name != PLATFORM_NAMES[platform_index] {
            return Err(format!(
                "platform order must be Linux then Windows, received {name} at {platform_index}"
            ));
        }
        let compiler_executables = object(
            platform
                .get("compiler_executables")
                .ok_or("compiler_executables missing")?,
            "compiler_executables",
        )?;
        exact_keys(
            compiler_executables,
            &["first", "second"],
            "compiler_executables",
        )?;
        for production in ["first", "second"] {
            let record = object(
                compiler_executables.get(production).unwrap(),
                "compiler executable",
            )?;
            exact_keys(
                record,
                &["path", "producer_command", "sha256", "size"],
                "compiler executable",
            )?;
            exact_string(
                record,
                "path",
                &compiler_path(name, production),
                "compiler executable",
            )?;
            exact_string(
                record,
                "producer_command",
                &format!("compiler_build_{production}"),
                "compiler executable",
            )?;
            if !valid_sha256(record.get("sha256").and_then(Value::as_str).unwrap_or(""))
                || record.get("size").and_then(Value::as_u64).is_none()
            {
                return Err(format!(
                    "{name} compiler {production} traceability record malformed"
                ));
            }
        }
        let observations = object(
            platform.get("observations").ok_or("observations missing")?,
            "observations",
        )?;
        exact_keys(observations, &["kernel", "runner_image"], "observations")?;
        for key in ["kernel", "runner_image"] {
            if observations
                .get(key)
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
            {
                return Err(format!("{name}.{key} observation missing"));
            }
        }
        let toolchain = object(
            platform.get("toolchain").ok_or("toolchain missing")?,
            "toolchain",
        )?;
        exact_keys(
            toolchain,
            &[
                "archive_name",
                "archive_sha256",
                "archive_size",
                "llvm_version",
                "rust_commit",
                "rust_version",
                "setup_boundary",
                "tools",
            ],
            "toolchain",
        )?;
        exact_string(toolchain, "llvm_version", LLVM_VERSION, "toolchain")?;
        exact_string(toolchain, "rust_version", RUST_VERSION, "toolchain")?;
        exact_string(toolchain, "rust_commit", RUST_COMMIT, "toolchain")?;
        let (archive_name, archive_size, archive_hash) = if name == "linux-x86_64" {
            (
                "LLVM-22.1.8-Linux-X64.tar.xz",
                1_938_859_476,
                "df0e1ecf16caf3489a272a5eea4eec9b0d82878f6477fa309504f918a0006384",
            )
        } else {
            (
                "clang+llvm-22.1.8-x86_64-pc-windows-msvc.tar.xz",
                862_053_924,
                "d96c2cc1736f4eb7fa43cb9bbdf56d93551a9ae0a9aadb9c99c3c3b2b712a234",
            )
        };
        exact_string(toolchain, "archive_name", archive_name, "toolchain")?;
        exact_u64(toolchain, "archive_size", archive_size, "toolchain")?;
        exact_string(toolchain, "archive_sha256", archive_hash, "toolchain")?;
        exact_string(
            toolchain,
            "setup_boundary",
            "workflow-acquisition-only; every final tool payload and version is verified before capture",
            "toolchain",
        )?;
        let tools = object(
            toolchain.get("tools").ok_or("tool records missing")?,
            "tools",
        )?;
        exact_keys(tools, &TOOL_NAMES, "tools")?;
        for tool in TOOL_NAMES {
            validate_tool_record(tools.get(tool).unwrap(), name, tool)?;
        }
        for (command_name, tool) in [
            ("compiler_build_first", "cargo"),
            ("compiler_build_second", "cargo"),
            ("llvm_verify_first", "opt"),
            ("llvm_verify_second", "opt"),
            ("llvm_assemble_first", "llvm-as"),
            ("llvm_assemble_second", "llvm-as"),
            ("machine_verify_first", "llc"),
            ("machine_verify_second", "llc"),
            ("link_o0_first", "clang"),
            ("link_o0_second", "clang"),
            ("link_o2_first", "clang"),
            ("link_o2_second", "clang"),
        ] {
            let command = object(
                platform["commands"].get(command_name).unwrap(),
                "linked command",
            )?;
            let argv = array(command.get("argv").unwrap(), "linked command argv")?;
            if argv.first() != tools[tool].get("path") {
                return Err(format!(
                    "{name}.{command_name} is not linked to recorded {tool} payload"
                ));
            }
        }
        let lld_selector = format!(
            "--ld-path={}",
            tools["lld"].get("path").unwrap().as_str().unwrap()
        );
        for command_name in [
            "link_o0_first",
            "link_o0_second",
            "link_o2_first",
            "link_o2_second",
        ] {
            let argv = array(
                platform["commands"][command_name].get("argv").unwrap(),
                "link argv",
            )?;
            if !argv.contains(&json!(lld_selector.clone())) {
                return Err(format!(
                    "{name}.{command_name} is not linked to the recorded lld payload"
                ));
            }
        }
        let artifacts = object(
            platform.get("artifacts").ok_or("artifacts missing")?,
            "artifacts",
        )?;
        exact_keys(artifacts, &ARTIFACT_NAMES, "artifacts")?;
        for artifact_name in ARTIFACT_NAMES {
            let pair = object(artifacts.get(artifact_name).unwrap(), "artifact pair")?;
            exact_keys(pair, &["first", "pair_equal", "second"], "artifact pair")?;
            if pair.get("pair_equal") != Some(&Value::Bool(true)) {
                return Err(format!(
                    "{name}.{artifact_name} pair did not declare equality"
                ));
            }
            let first = object(pair.get("first").unwrap(), "first artifact")?;
            let second = object(pair.get("second").unwrap(), "second artifact")?;
            exact_keys(
                first,
                &["path", "producer_command", "sha256", "size"],
                "first artifact",
            )?;
            exact_keys(
                second,
                &["path", "producer_command", "sha256", "size"],
                "second artifact",
            )?;
            for (production, record) in [("first", first), ("second", second)] {
                exact_string(
                    record,
                    "path",
                    &artifact_path(name, production, artifact_name),
                    "artifact",
                )?;
                exact_string(
                    record,
                    "producer_command",
                    &artifact_producer(artifact_name, production),
                    "artifact",
                )?;
            }
            if first.get("sha256") != second.get("sha256")
                || first.get("size") != second.get("size")
                || first
                    .get("size")
                    .and_then(Value::as_u64)
                    .is_none_or(|size| size == 0)
                || !valid_sha256(first.get("sha256").and_then(Value::as_str).unwrap_or(""))
            {
                return Err(format!(
                    "{name}.{artifact_name} productions are not equal nonempty byte records"
                ));
            }
        }
        let commands = object(
            platform.get("commands").ok_or("commands missing")?,
            "commands",
        )?;
        exact_keys(commands, &COMMAND_NAMES, "commands")?;
        for command_name in COMMAND_NAMES {
            validate_command(commands.get(command_name).unwrap(), name, command_name)?;
        }
        let support_path = if name == "windows-x86_64" {
            "${WORK}/windows-x86_64/windows-chkstk.S"
        } else {
            "${WORK}/linux-x86_64/linux-start.S"
        };
        let mut available: BTreeSet<String> = [
            "${SUBJECT}/src/compiler/Cargo.toml",
            "${SUBJECT}/src/compiler/Cargo.lock",
            "${SUBJECT}/src/compiler",
            "${SUBJECT}/examples/fixed_int_array_v0/relu_argmax_inference.aero",
            support_path,
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        for command_name in COMMAND_NAMES {
            let command = object(commands.get(command_name).unwrap(), "command")?;
            for consumed in array(command.get("consumes").unwrap(), "command consumes")? {
                let consumed = consumed.as_str().ok_or("command consume is not a string")?;
                if !available.contains(consumed) {
                    return Err(format!(
                        "{name}.{command_name} consumes unavailable {consumed}"
                    ));
                }
            }
            for produced in array(command.get("produces").unwrap(), "command produces")? {
                let produced = produced.as_str().ok_or("command product is not a string")?;
                if !available.insert(produced.to_owned()) {
                    return Err(format!(
                        "{name}.{command_name} duplicates produced path {produced}"
                    ));
                }
            }
        }
        for production in ["first", "second"] {
            let compiler = object(
                compiler_executables.get(production).unwrap(),
                "compiler executable",
            )?;
            let producer = compiler.get("producer_command").unwrap().as_str().unwrap();
            if !array(
                commands[producer].get("produces").unwrap(),
                "producer outputs",
            )?
            .contains(compiler.get("path").unwrap())
            {
                return Err(format!("{name} compiler producer/path linkage drifted"));
            }
            for artifact_name in ARTIFACT_NAMES {
                let record = object(
                    artifacts[artifact_name].get(production).unwrap(),
                    "artifact",
                )?;
                let producer = record["producer_command"].as_str().unwrap();
                if !array(
                    commands[producer].get("produces").unwrap(),
                    "producer outputs",
                )?
                .contains(record.get("path").unwrap())
                {
                    return Err(format!(
                        "{name}.{artifact_name}.{production} producer/path linkage drifted"
                    ));
                }
            }
        }
        validate_public_semantics(name, commands, platform.get("public_semantics").unwrap())?;
        if platform.get("failures") != Some(&json!([])) {
            return Err(format!("{name} failure record must explicitly be empty"));
        }
    }
    if manifest.get("failures") != Some(&json!([])) {
        return Err("manifest must contain an explicit failure record".to_owned());
    }
    let limitations = array(
        manifest
            .get("limitations")
            .ok_or("manifest.limitations missing")?,
        "manifest.limitations",
    )?;
    if limitations != fixture_manifest()["limitations"].as_array().unwrap() {
        return Err("manifest limitations drifted from the exact no-overclaim boundary".to_owned());
    }
    reject_measurement_fields(value, "manifest")?;
    Ok(())
}

fn validate_manifest_file_hashes(root: &Path, manifest: &Value) -> Result<(), String> {
    for key in ["schema", "oracle", "reproduce", "workflow", "tool"] {
        let record = object(
            manifest
                .get(key)
                .ok_or_else(|| format!("manifest.{key} missing"))?,
            key,
        )?;
        let path = record
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("manifest.{key}.path missing"))?;
        let expected = record
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("manifest.{key}.sha256 missing"))?;
        let bytes = fs::read(root.join(path))
            .map_err(|error| format!("cannot read hash-bound {path}: {error}"))?;
        let actual = sha256_hex(&bytes);
        if actual != expected {
            return Err(format!(
                "manifest.{key} hash mismatch for {path}: expected {expected}, received {actual}"
            ));
        }
    }
    Ok(())
}

fn run_python_contract(root: &Path, arguments: &[&str]) -> Result<(), String> {
    let output = Command::new("python")
        .current_dir(root)
        .arg(TOOL_PATH)
        .args(arguments)
        .output()
        .map_err(|error| {
            format!("cannot execute CAP-024 Python interface {arguments:?}: {error}")
        })?;
    if !output.status.success() {
        return Err(format!(
            "CAP-024 Python interface {arguments:?} failed with {:?}: stdout={:?}, stderr={:?}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let mode = arguments
        .windows(2)
        .find(|pair| pair[0] == "--mode")
        .map(|pair| pair[1])
        .ok_or("CAP-024 Python interface omitted --mode")?;
    let negative_case = arguments
        .windows(2)
        .find(|pair| pair[0] == "--negative-case")
        .map(|pair| pair[1]);
    let expected = if let Some(case) = negative_case {
        format!("{{\"case\":\"{case}\",\"mode\":\"{mode}\",\"ok\":true}}\n")
    } else {
        format!("{{\"mode\":\"{mode}\",\"ok\":true}}\n")
    };
    if output.stdout != expected.as_bytes() || !output.stderr.is_empty() {
        return Err(format!(
            "CAP-024 Python interface {mode} did not return its exact canonical success record"
        ));
    }
    Ok(())
}

fn run_python_contract_rejects(root: &Path, arguments: &[&str]) -> Result<(), String> {
    let output = Command::new("python")
        .current_dir(root)
        .arg(TOOL_PATH)
        .args(arguments)
        .output()
        .map_err(|error| format!("cannot execute negative CAP-024 Python interface: {error}"))?;
    if output.status.success() {
        return Err(format!(
            "CAP-024 Python interface accepted adversarial input {arguments:?}"
        ));
    }
    Ok(())
}

fn read_canonical_value(path: &Path, label: &str) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read {label}: {error}"))?;
    parse_canonical_json_file(&bytes, label)
}

fn write_canonical_value(path: &Path, value: &Value) -> Result<(), String> {
    fs::write(path, canonical_json_file_bytes(value))
        .map_err(|error| format!("cannot write {}: {error}", path.display()))
}

fn copy_bundle(root: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir(destination)
        .map_err(|error| format!("cannot create {}: {error}", destination.display()))?;
    for file in REQUIRED_BUNDLE_FILES {
        fs::copy(
            root.join(BUNDLE_DIRECTORY).join(file),
            destination.join(file),
        )
        .map_err(|error| format!("cannot copy adversarial bundle file {file}: {error}"))?;
    }
    Ok(())
}

fn create_unique_temp_directory(prefix: &str) -> Result<PathBuf, String> {
    for counter in 0..1_000_u32 {
        let candidate =
            std::env::temp_dir().join(format!("{prefix}-{}-{counter}", std::process::id()));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "cannot create temporary CAP-024 directory {}: {error}",
                    candidate.display()
                ));
            }
        }
    }
    Err(format!(
        "cannot allocate unique temporary CAP-024 directory for {prefix}"
    ))
}

fn rebind_bundle_file(bundle: &Path, manifest_key: &str, file: &str) -> Result<(), String> {
    let manifest_path = bundle.join("manifest.json");
    let mut manifest = read_canonical_value(&manifest_path, "adversarial manifest")?;
    let bytes = fs::read(bundle.join(file))
        .map_err(|error| format!("cannot read adversarial {file}: {error}"))?;
    manifest[manifest_key]["sha256"] = json!(sha256_hex(&bytes));
    write_canonical_value(&manifest_path, &manifest)
}

fn prepare_adversarial_bundle(root: &Path, destination: &Path, case: &str) -> Result<(), String> {
    copy_bundle(root, destination)?;
    let manifest_path = destination.join("manifest.json");
    match case {
        "oracle_drift" => {
            let oracle_path = destination.join("oracle.json");
            let mut oracle = read_canonical_value(&oracle_path, "adversarial oracle")?;
            oracle["records"][1]["result"][6] = json!(-1);
            write_canonical_value(&oracle_path, &oracle)?;
            rebind_bundle_file(destination, "oracle", "oracle.json")?;
        }
        "extra_bundle_file" => {
            fs::write(destination.join("extra.txt"), b"not authorized\n")
                .map_err(|error| format!("cannot write extra bundle fixture: {error}"))?;
        }
        "reproduce_drift" => {
            fs::write(
                destination.join("REPRODUCE.md"),
                b"mutated reproduction procedure\n",
            )
            .map_err(|error| format!("cannot write reproduction drift fixture: {error}"))?;
        }
        _ => {
            let mut manifest = read_canonical_value(&manifest_path, "adversarial manifest")?;
            match case {
                "artifact_pair" => {
                    manifest["platforms"][0]["artifacts"]["llvm"]["second"]["sha256"] =
                        json!("9".repeat(64));
                }
                "artifact_path" => {
                    manifest["platforms"][0]["artifacts"]["llvm"]["first"]["path"] =
                        json!("wrong.ll");
                }
                "artifact_producer" => {
                    manifest["platforms"][0]["artifacts"]["llvm"]["first"]["producer_command"] =
                        json!("clean_before");
                }
                "command_argv" => {
                    manifest["platforms"][0]["commands"]["llvm_verify_first"]["argv"][0] =
                        json!("arbitrary");
                }
                "command_env" => {
                    manifest["platforms"][1]["commands"]["llvm_verify_first"]["env"]["inheritance"] =
                        json!("runner-substrate-observation-only");
                }
                "public_missing" => {
                    set_public_raw_streams(&mut manifest, 0, b"Aero execution diagnostic\n", b"")
                }
                "public_duplicate" => {
                    set_public_raw_streams(&mut manifest, 0, b"Exit code: 91\nExit code: 91\n", b"")
                }
                "public_wrong_exit" => {
                    set_public_raw_streams(&mut manifest, 0, b"Exit code: 91\nExit code: 1\n", b"")
                }
                "public_output" => set_public_raw_streams(
                    &mut manifest,
                    0,
                    b"Exit code: 91\nOutput: payload\n",
                    b"",
                ),
                "public_error" => set_public_raw_streams(
                    &mut manifest,
                    0,
                    b"Exit code: 91\nError output: payload\n",
                    b"",
                ),
                "public_prefixed_exit" => {
                    set_public_raw_streams(&mut manifest, 0, b"prefix Exit code: 91\n", b"")
                }
                "public_exit_in_stderr" => set_public_raw_streams(
                    &mut manifest,
                    0,
                    b"Aero execution diagnostic\n",
                    b"Exit code: 91\n",
                ),
                _ => return Err(format!("unknown adversarial bundle case {case}")),
            }
            write_canonical_value(&manifest_path, &manifest)?;
        }
    }
    Ok(())
}

fn fresh_replay_manifest(accepted: &Value) -> Value {
    let mut fresh = accepted.clone();
    for platform in 0..2 {
        fresh["platforms"][platform]["observations"]["runner_image"] =
            json!(format!("fresh-runner-{platform}"));
        fresh["platforms"][platform]["observations"]["kernel"] =
            json!(format!("fresh-kernel-{platform}"));
        for production in ["first", "second"] {
            fresh["platforms"][platform]["compiler_executables"][production]["sha256"] =
                json!("5".repeat(64));
            fresh["platforms"][platform]["compiler_executables"][production]["size"] =
                json!(654_321_u64);
        }
        for command in [
            "aero_build_llvm_first",
            "aero_build_llvm_second",
            "public_run",
        ] {
            let stdout = if command == "public_run" {
                byte_record_from_bytes(b"fresh diagnostic\nExit code: 91\n")
            } else {
                byte_record_from_bytes(b"fresh diagnostic\n")
            };
            fresh["platforms"][platform]["commands"][command]["stdout"] = stdout;
            fresh["platforms"][platform]["commands"][command]["stderr"] =
                byte_record_from_bytes(b"fresh trace\n");
        }
    }
    fresh
}

fn reject_measurement_fields(value: &Value, path: &str) -> Result<(), String> {
    match value {
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                reject_measurement_fields(value, &format!("{path}/{index}"))?;
            }
        }
        Value::Object(values) => {
            for (key, value) in values {
                let normalized = key.to_ascii_lowercase();
                if [
                    "timing",
                    "duration",
                    "elapsed",
                    "throughput",
                    "speedup",
                    "latency",
                    "memory",
                    "energy",
                    "benchmark",
                    "performance",
                ]
                .into_iter()
                .any(|forbidden| normalized.contains(forbidden))
                {
                    return Err(format!(
                        "{path} contains unauthorized measurement field {key:?}"
                    ));
                }
                reject_measurement_fields(value, &format!("{path}/{key}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_claim_index(value: &Value) -> Result<(), String> {
    let index = object(value, "claims index")?;
    exact_keys(
        index,
        &[
            "claims",
            "hardware_summary",
            "repo",
            "repo_commit",
            "schema_version",
            "verified_on",
        ],
        "claims index",
    )?;
    let historical_header = Value::Object(
        index
            .iter()
            .filter(|(key, _)| key.as_str() != "claims")
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    );
    if sha256_hex(canonical_json(&historical_header).as_bytes()) != HISTORICAL_CLAIM_HEADER_SHA256 {
        return Err("historical claim catalog header changed".to_owned());
    }
    let actual = array(index.get("claims").ok_or("claims missing")?, "claims")?;
    if actual.len() != HISTORICAL_CLAIM_SHA256.len() + 1 {
        return Err(
            "claim catalog is not an exact additive append over six historical claims".to_owned(),
        );
    }
    for (index, (claim, expected_hash)) in actual.iter().zip(HISTORICAL_CLAIM_SHA256).enumerate() {
        let actual_hash = sha256_hex(canonical_json(claim).as_bytes());
        if actual_hash != expected_hash {
            return Err(format!(
                "historical claim {index} changed: expected {expected_hash}, received {actual_hash}"
            ));
        }
    }
    let claim = object(actual.last().unwrap(), "CAP-024 claim")?;
    exact_keys(
        claim,
        &["artifacts", "claim", "id", "status"],
        "CAP-024 claim",
    )?;
    exact_string(claim, "id", CLAIM_ID, "CAP-024 claim")?;
    exact_string(claim, "status", CLAIM_STATUS, "CAP-024 claim")?;
    if claim.get("artifacts")
        != Some(&json!([
            format!("{BUNDLE_DIRECTORY}/manifest.json"),
            format!("{BUNDLE_DIRECTORY}/oracle.json"),
            format!("{BUNDLE_DIRECTORY}/REPRODUCE.md")
        ]))
    {
        return Err(
            "CAP-024 claim does not index exactly the three frozen bundle files".to_owned(),
        );
    }
    let text = claim
        .get("claim")
        .and_then(Value::as_str)
        .ok_or("CAP-024 claim text missing")?;
    let expected_claim = format!(
        "Accepted-head CAP-023 correctness and reproducibility record for {SUBJECT_COMMIT}; no performance claim."
    );
    if text != expected_claim {
        return Err("CAP-024 claim text drifted from its exact no-performance boundary".to_owned());
    }
    Ok(())
}

fn validate_closed_schema_nodes(value: &Value, path: &str) -> Result<(), String> {
    match value {
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                validate_closed_schema_nodes(value, &format!("{path}/{index}"))?;
            }
        }
        Value::Object(schema) => {
            if schema.get("type").and_then(Value::as_str) == Some("object") {
                if schema.get("additionalProperties") != Some(&Value::Bool(false)) {
                    return Err(format!(
                        "object schema {path} does not reject unknown properties"
                    ));
                }
                let properties = object(
                    schema
                        .get("properties")
                        .ok_or_else(|| format!("object schema {path} omitted properties"))?,
                    path,
                )?;
                let required = array(
                    schema
                        .get("required")
                        .ok_or_else(|| format!("object schema {path} omitted required"))?,
                    path,
                )?;
                let required: BTreeSet<&str> = required.iter().filter_map(Value::as_str).collect();
                let properties: BTreeSet<&str> = properties.keys().map(String::as_str).collect();
                if required != properties {
                    return Err(format!(
                        "object schema {path} does not require every exact property"
                    ));
                }
            }
            if schema.get("type").and_then(Value::as_str) == Some("array") {
                for key in ["items", "minItems", "maxItems", "uniqueItems"] {
                    if !schema.contains_key(key) {
                        return Err(format!("array schema {path} omitted {key}"));
                    }
                }
            }
            for (key, child) in schema {
                validate_closed_schema_nodes(child, &format!("{path}/{key}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_schema(value: &Value) -> Result<(), String> {
    let schema = object(value, "schema")?;
    exact_keys(
        schema,
        &[
            "$defs",
            "$id",
            "$schema",
            "additionalProperties",
            "properties",
            "required",
            "title",
            "type",
        ],
        "schema",
    )?;
    exact_string(
        schema,
        "$schema",
        "https://json-schema.org/draft/2020-12/schema",
        "schema",
    )?;
    exact_string(schema, "$id", SCHEMA_ID, "schema")?;
    exact_string(schema, "type", "object", "schema")?;
    if schema.get("additionalProperties") != Some(&Value::Bool(false)) {
        return Err("schema root must fail closed on unknown fields".to_owned());
    }
    let required = array(
        schema.get("required").ok_or("schema.required missing")?,
        "schema.required",
    )?;
    let properties = object(
        schema
            .get("properties")
            .ok_or("schema.properties missing")?,
        "schema.properties",
    )?;
    let required: BTreeSet<&str> = required.iter().filter_map(Value::as_str).collect();
    let properties: BTreeSet<&str> = properties.keys().map(String::as_str).collect();
    let expected: BTreeSet<&str> = MANIFEST_FIELDS.into_iter().collect();
    if required != expected || properties != expected {
        return Err("schema root does not freeze every exact manifest property".to_owned());
    }
    let definitions = object(
        schema.get("$defs").ok_or("schema.$defs missing")?,
        "schema.$defs",
    )?;
    exact_keys(
        definitions,
        &["artifact_pair", "byte_record", "command"],
        "schema.$defs",
    )?;
    validate_closed_schema_nodes(value, "schema")?;
    let serialized = canonical_json(value);
    for required in MANIFEST_FIELDS
        .into_iter()
        .chain(ARTIFACT_NAMES)
        .chain(COMMAND_NAMES)
        .chain([
            "sha256",
            "size",
            "base64",
            "minItems",
            "maxItems",
            "uniqueItems",
            "const",
            "additionalProperties",
            "public_semantics",
            "canonical_projection",
            "excluded_paths",
            "fresh_observations",
            "path",
            "producer_command",
            "consumes",
            "produces",
            "payload_sha256",
            "payload_size",
            "version",
            "tools",
        ])
    {
        if !serialized.contains(required) {
            return Err(format!(
                "schema omitted closure keyword or field {required}"
            ));
        }
    }
    if value != &fixture_schema() {
        return Err("schema drifted from the exact closed CAP-024 shape".to_owned());
    }
    Ok(())
}

fn fixture_workflow_text() -> String {
    [
        "name: CAP-023 accepted-head evidence",
        "permissions:\n  contents: read",
        "on:\n  pull_request:\n  push:\n    branches:\n      - master\n  workflow_dispatch:",
        "  capture-linux:\n    if: github.event_name == 'pull_request' || github.event_name == 'workflow_dispatch'",
        "runs-on: ubuntu-24.04",
        "  capture-windows:\n    if: github.event_name == 'pull_request' || github.event_name == 'workflow_dispatch'",
        "runs-on: windows-2025",
        "aggregate:",
        "needs: [capture-linux, capture-windows]",
        "if: always()",
        "upload-linux:\n  if: always()",
        "upload-windows:\n  if: always()",
        "needs.capture-linux.result",
        "needs.capture-windows.result",
        "canonical-failure-record",
        "workflow-acquisition-only",
        "verify final cargo rustc clang lld opt llvm-as llc payloads and versions",
        CHECKOUT_ACTION,
        UPLOAD_ACTION,
        DOWNLOAD_ACTION,
        SUBJECT_COMMIT,
        TOOL_PATH,
        "python tools/cap024_inference_evidence.py --mode capture --platform linux-x86_64",
        "python tools/cap024_inference_evidence.py --mode capture --platform windows-x86_64",
        "python tools/cap024_inference_evidence.py --mode aggregate",
        "      - name: Replay accepted master merge head\n        if: github.event_name == 'push' && github.ref == 'refs/heads/master'\n        run: python tools/cap024_inference_evidence.py --mode replay --bundle claim-verification/results/aero_cap023_inference_correctness_918c9222_20260813 --verify-only",
        "core.autocrlf false",
        "CARGO_NET_OFFLINE",
        "fresh-observations",
        "canonical-text-records-only",
    ]
    .join("\n")
}

fn fixture_tool_text() -> String {
    [
        "import argparse",
        "import base64",
        "import hashlib",
        "import json",
        "import pathlib",
        "import subprocess",
        "import tarfile",
        "import urllib.request",
        CLAIM_ID,
        SCHEMA_ID,
        TOOL_ID,
        ORACLE_ID,
        SUBJECT_COMMIT,
        RUST_VERSION,
        RUST_COMMIT,
        LLVM_VERSION,
        "df0e1ecf16caf3489a272a5eea4eec9b0d82878f6477fa309504f918a0006384",
        "LINUX_LLVM_ARCHIVE_SIZE = 1938859476",
        "d96c2cc1736f4eb7fa43cb9bbdf56d93551a9ae0a9aadb9c99c3c3b2b712a234",
        "WINDOWS_LLVM_ARCHIVE_SIZE = 862053924",
        "linux-start.S",
        "b95dbd79fd7b976862149e5635e148b9a9d2bbf20b2c3912a1f8d76c227379bb",
        "LINUX_START_SIZE = 205",
        "windows-chkstk.S",
        "b971f9c51534aff82d774c26b6a6f2312a3beeac5e1710a69f3d88bd5671f376",
        "WINDOWS_CHKSTK_SIZE = 378",
        "-nostdlib",
        "-verify-machineinstrs",
        "--ld-path=${LLVM}/bin/ld.lld",
        "--ld-path=${LLVM}/bin/lld-link.exe",
        "--build-id=none",
        "/nodefaultlib",
        "/brepro",
        "canonical_projection",
        "fresh_observations",
        "shell=False",
        "explicit signed wrapping arithmetic",
        "Exit code:",
        "Output:",
        "Error output:",
        "exit_report_count",
        "application_stdout",
        "application_stderr",
        "pair_equal",
        "exactly REPRODUCE.md manifest.json oracle.json",
        "--mode self-test",
        "--mode validate --bundle",
        "--mode replay --bundle",
        "--fresh-manifest",
        "--fresh-observations",
        "--negative-case",
        "negative_case",
        "schema_unknown",
        "schema_float",
        "schema_duplicate",
        "oracle_drift",
        "artifact_pair",
        "artifact_path",
        "artifact_producer",
        "command_argv",
        "command_env",
        "public_missing",
        "public_duplicate",
        "public_wrong_exit",
        "public_output",
        "public_error",
        "public_prefixed_exit",
        "public_exit_in_stderr",
        "included_replay_difference",
        "excluded_replay_leaves",
        "extra_bundle_file",
        "reproduce_drift",
        "accepted_manifest_immutable",
        "subprocess_allowlist = PIPELINE_COMMANDS | TOOL_VERSION_PROBES",
        "def run_recorded_subprocess(command_id",
        "def run_tool_version_probe(tool_id",
        "def materialize_command_env(env_spec",
        "env=materialize_command_env(command_spec[\"env\"])",
        "target-byte commands reject ambient inheritance",
        "banner_kind",
        "rustc-vv",
        "cargo-vv",
        "release:",
        "commit-hash:",
        "LLVM version 22.1.8",
        "clang version 22.1.8",
        "LLD 22.1.8",
        "canonical-failure-record",
        "workflow-acquisition-only",
        "runner-substrate-observation-only",
        "\"inheritance\": \"none\"",
        "verify final cargo rustc clang lld opt llvm-as llc payloads and versions",
        "capture exceptions become failure records",
        "sorted-compact-json-plus-lf-v1",
        "{\"mode\":\"self-test\",\"ok\":true}",
        "{\"mode\":\"validate\",\"ok\":true}",
        "{\"mode\":\"replay\",\"ok\":true}",
        "cap024-fresh-observations-v1",
        "accepted_manifest_sha256",
        "pointer",
        "value",
    ]
    .into_iter()
    .chain(EVIDENCE_PIN_ANCHORS.iter().copied())
    .chain(PLATFORM_NAMES)
    .chain(ARTIFACT_NAMES)
    .chain(COMMAND_NAMES)
    .chain(REPLAY_EXCLUSIONS)
    .collect::<Vec<_>>()
    .join("\n")
}

fn fixture_reproduce_text() -> String {
    [
        "Complete third-party target artifact and observable result procedure",
        "initial capture",
        "replay",
        SUBJECT_COMMIT,
        SUBJECT_TREE,
        CLAIM_ID,
        SCHEMA_ID,
        TOOL_ID,
        ORACLE_ID,
        TOOL_PATH,
        "linux-x86_64",
        "windows-x86_64",
        "runner image and kernel observation are not an immutable or reconstructible evidence input",
        "a repeat on another host is new corroboration",
        "failure observations and limitations",
        "no performance claim",
        "parsed Exit code: 91 and no application Output: or Error output:",
        "fresh observations never rewrite accepted observations",
        "core.autocrlf false and canonical Git blob bytes",
        "CARGO_NET_OFFLINE true",
        "Native O0 and O2 exit 91 with empty stdout and stderr",
        "The raw Aero LLVM-build and public-run diagnostic streams are traceability-only replay exclusions",
        "Two same-platform productions must have equal SHA-256 and byte size",
    ]
    .into_iter()
    .chain(EVIDENCE_PIN_ANCHORS.iter().copied())
    .chain(ARTIFACT_NAMES)
    .chain(COMMAND_NAMES)
    .chain(REPLAY_EXCLUSIONS)
    .collect::<Vec<_>>()
    .join("\n")
}

fn reject_measurement_code(text: &str, label: &str) -> Result<(), String> {
    for forbidden in [
        "perf_counter",
        "process_time",
        "import time",
        "Measure-Command",
        "hyperfine",
        "criterion",
        "benchmark_results",
    ] {
        if text.contains(forbidden) {
            return Err(format!(
                "{label} contains unauthorized measurement anchor {forbidden:?}"
            ));
        }
    }
    Ok(())
}

fn reject_affirmative_performance_prose(
    text: &str,
    label: &str,
    strict_prose: bool,
) -> Result<(), String> {
    const PERFORMANCE_WORDS: [&str; 8] = [
        "benchmark",
        "energy",
        "latency",
        "memory",
        "performance",
        "speedup",
        "throughput",
        "timing",
    ];
    const AFFIRMATIVE_WORDS: [&str; 29] = [
        "achieve",
        "achieved",
        "achieves",
        "available",
        "better",
        "efficient",
        "efficiency",
        "excellent",
        "fast",
        "faster",
        "gain",
        "gains",
        "good",
        "high",
        "improved",
        "improves",
        "low",
        "measured",
        "meets",
        "met",
        "reached",
        "reaches",
        "result",
        "results",
        "score",
        "slow",
        "slower",
        "worse",
        "x",
    ];
    const NEGATIVE_WORDS: [&str; 11] = [
        "exclude",
        "excluded",
        "exclusion",
        "forbid",
        "forbidden",
        "limitation",
        "never",
        "no",
        "not",
        "unmeasured",
        "without",
    ];

    for raw_segment in text.split(['\n', '.', ';', '!', '?']) {
        for segment in raw_segment
            .replace(" however ", ";")
            .replace(" but ", ";")
            .replace(" yet ", ";")
            .split(';')
        {
            let words: Vec<String> = segment
                .split(|character: char| !character.is_ascii_alphanumeric())
                .filter(|word| !word.is_empty())
                .map(str::to_ascii_lowercase)
                .collect();
            let performance_positions: Vec<usize> = words
                .iter()
                .enumerate()
                .filter_map(|(index, word)| {
                    PERFORMANCE_WORDS.contains(&word.as_str()).then_some(index)
                })
                .collect();
            if performance_positions.is_empty() {
                continue;
            }
            let contains_number = words
                .iter()
                .any(|word| word.bytes().any(|byte| byte.is_ascii_digit()));
            let contains_affirmative = words
                .iter()
                .any(|word| AFFIRMATIVE_WORDS.contains(&word.as_str()));
            let blanket_negative_boundary = !contains_number
                && !contains_affirmative
                && words.iter().any(|word| word == "claim")
                && words
                    .iter()
                    .any(|word| NEGATIVE_WORDS.contains(&word.as_str()));
            if blanket_negative_boundary {
                continue;
            }
            for position in performance_positions {
                let start = position.saturating_sub(2);
                let end = (position + 3).min(words.len());
                let locally_negative = words[start..end]
                    .iter()
                    .any(|word| NEGATIVE_WORDS.contains(&word.as_str()));
                if !locally_negative
                    && (strict_prose
                        || segment.trim_start().starts_with('#')
                        || contains_number
                        || contains_affirmative
                        || segment.contains(':')
                        || words.get(position + 1).is_some_and(|word| {
                            matches!(word.as_str(), "is" | "was" | "equals" | "of")
                        }))
                {
                    return Err(format!(
                        "{label} contains an unauthorized affirmative performance claim: {:?}",
                        segment.trim()
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_workflow(text: &str) -> Result<(), String> {
    for required in [
        CHECKOUT_ACTION,
        UPLOAD_ACTION,
        DOWNLOAD_ACTION,
        "ubuntu-24.04",
        "windows-2025",
        "on:\n  pull_request:\n  push:\n    branches:\n      - master\n  workflow_dispatch:",
        SUBJECT_COMMIT,
        TOOL_PATH,
        "python tools/cap024_inference_evidence.py --mode capture --platform linux-x86_64",
        "python tools/cap024_inference_evidence.py --mode capture --platform windows-x86_64",
        "--mode aggregate",
        "--mode replay",
        "core.autocrlf false",
        "CARGO_NET_OFFLINE",
        "capture-linux",
        "capture-windows",
        "aggregate",
        "needs: [capture-linux, capture-windows]",
        "if: always()",
        "needs.capture-linux.result",
        "needs.capture-windows.result",
        "canonical-failure-record",
        "workflow-acquisition-only",
        "verify final cargo rustc clang lld opt llvm-as llc payloads and versions",
        "permissions:\n  contents: read",
        "workflow_dispatch:",
        "fresh-observations",
        "canonical-text-records-only",
    ] {
        if !text.contains(required) {
            return Err(format!(
                "CAP-024 workflow omitted exact anchor {required:?}"
            ));
        }
    }
    for forbidden in [
        "actions/checkout@v",
        "actions/upload-artifact@v",
        "actions/download-artifact@v",
        "ubuntu-latest",
        "windows-latest",
        "dtolnay/rust-toolchain",
    ] {
        if text.contains(forbidden) {
            return Err(format!(
                "CAP-024 workflow contains floating anchor {forbidden:?}"
            ));
        }
    }
    if text.matches("if: always()").count() < 3 {
        return Err(
            "CAP-024 workflow must always upload both capture records and always aggregate"
                .to_owned(),
        );
    }
    if text.matches("pull_request:").count() != 1
        || text.matches("workflow_dispatch:").count() != 1
        || text.matches("push:").count() != 1
        || !text.contains(
            "  capture-linux:\n    if: github.event_name == 'pull_request' || github.event_name == 'workflow_dispatch'",
        )
        || !text.contains(
            "  capture-windows:\n    if: github.event_name == 'pull_request' || github.event_name == 'workflow_dispatch'",
        )
        || !text.contains(
            "      - name: Replay accepted master merge head\n        if: github.event_name == 'push' && github.ref == 'refs/heads/master'\n        run: python tools/cap024_inference_evidence.py --mode replay --bundle claim-verification/results/aero_cap023_inference_correctness_918c9222_20260813 --verify-only",
        )
    {
        return Err(
            "CAP-024 workflow must map PR to initial capture and master push to merge-head replay"
                .to_owned(),
        );
    }
    reject_measurement_code(text, "CAP-024 workflow")?;
    reject_affirmative_performance_prose(text, "CAP-024 workflow", false)?;
    Ok(())
}

fn validate_tool(text: &str) -> Result<(), String> {
    for required in [
        "argparse",
        "base64",
        "hashlib",
        "json",
        "subprocess",
        CLAIM_ID,
        SCHEMA_ID,
        TOOL_ID,
        ORACLE_ID,
        SUBJECT_COMMIT,
        RUST_VERSION,
        RUST_COMMIT,
        LLVM_VERSION,
        "linux-start.S",
        "windows-chkstk.S",
        "-nostdlib",
        "-verify-machineinstrs",
        "--ld-path=${LLVM}/bin/ld.lld",
        "--ld-path=${LLVM}/bin/lld-link.exe",
        "--build-id=none",
        "/nodefaultlib",
        "/brepro",
        "canonical_projection",
        "fresh_observations",
        "urllib.request",
        "shell=False",
        "explicit signed wrapping arithmetic",
        "df0e1ecf16caf3489a272a5eea4eec9b0d82878f6477fa309504f918a0006384",
        "LINUX_LLVM_ARCHIVE_SIZE = 1938859476",
        "d96c2cc1736f4eb7fa43cb9bbdf56d93551a9ae0a9aadb9c99c3c3b2b712a234",
        "WINDOWS_LLVM_ARCHIVE_SIZE = 862053924",
        "b95dbd79fd7b976862149e5635e148b9a9d2bbf20b2c3912a1f8d76c227379bb",
        "LINUX_START_SIZE = 205",
        "b971f9c51534aff82d774c26b6a6f2312a3beeac5e1710a69f3d88bd5671f376",
        "WINDOWS_CHKSTK_SIZE = 378",
    ] {
        if !text.contains(required) {
            return Err(format!("CAP-024 tool omitted frozen anchor {required:?}"));
        }
    }
    for exclusion in REPLAY_EXCLUSIONS {
        if !text.contains(exclusion) {
            return Err(format!(
                "CAP-024 tool omitted closed replay exclusion {exclusion:?}"
            ));
        }
    }
    for required in EVIDENCE_PIN_ANCHORS
        .iter()
        .copied()
        .chain(PLATFORM_NAMES)
        .chain(ARTIFACT_NAMES)
        .chain(COMMAND_NAMES)
        .chain([
            "Exit code:",
            "Output:",
            "Error output:",
            "exit_report_count",
            "application_stdout",
            "application_stderr",
            "pair_equal",
            "exactly REPRODUCE.md manifest.json oracle.json",
            "--mode self-test",
            "--mode validate --bundle",
            "--mode replay --bundle",
            "--fresh-manifest",
            "--fresh-observations",
            "--negative-case",
            "negative_case",
            "schema_unknown",
            "schema_float",
            "schema_duplicate",
            "oracle_drift",
            "artifact_pair",
            "artifact_path",
            "artifact_producer",
            "command_argv",
            "command_env",
            "public_missing",
            "public_duplicate",
            "public_wrong_exit",
            "public_output",
            "public_error",
            "public_prefixed_exit",
            "public_exit_in_stderr",
            "included_replay_difference",
            "excluded_replay_leaves",
            "extra_bundle_file",
            "reproduce_drift",
            "accepted_manifest_immutable",
            "subprocess_allowlist = PIPELINE_COMMANDS | TOOL_VERSION_PROBES",
            "def run_recorded_subprocess(command_id",
            "def run_tool_version_probe(tool_id",
            "def materialize_command_env(env_spec",
            "env=materialize_command_env(command_spec[\"env\"])",
            "target-byte commands reject ambient inheritance",
            "banner_kind",
            "rustc-vv",
            "cargo-vv",
            "release:",
            "commit-hash:",
            "LLVM version 22.1.8",
            "clang version 22.1.8",
            "LLD 22.1.8",
            "canonical-failure-record",
            "runner-substrate-observation-only",
            "\"inheritance\": \"none\"",
            "capture exceptions become failure records",
            "sorted-compact-json-plus-lf-v1",
            "{\"mode\":\"self-test\",\"ok\":true}",
            "{\"mode\":\"validate\",\"ok\":true}",
            "{\"mode\":\"replay\",\"ok\":true}",
            "cap024-fresh-observations-v1",
            "accepted_manifest_sha256",
            "pointer",
            "value",
        ])
    {
        if !text.contains(required) {
            return Err(format!(
                "CAP-024 tool omitted exact evidence anchor {required:?}"
            ));
        }
    }
    for forbidden in [
        "import requests",
        "from requests",
        "import jsonschema",
        "from jsonschema",
        "import yaml",
        "from yaml",
        "pip install",
        "cargo add",
        "shell=True",
        "os.system",
        "subprocess.Popen",
        "subprocess.check_call",
        "subprocess.check_output",
        "os.spawn",
        "rustup",
    ] {
        if text.contains(forbidden) {
            return Err(format!(
                "CAP-024 tool is not standard-library only: {forbidden:?}"
            ));
        }
    }
    reject_measurement_code(text, "CAP-024 tool")?;
    reject_affirmative_performance_prose(text, "CAP-024 tool", false)?;
    Ok(())
}

fn validate_reproduce(text: &str) -> Result<(), String> {
    for required in [
        SUBJECT_COMMIT,
        SUBJECT_TREE,
        CLAIM_ID,
        SCHEMA_ID,
        TOOL_ID,
        ORACLE_ID,
        TOOL_PATH,
        "linux-x86_64",
        "windows-x86_64",
        "runner image",
        "kernel",
        "not an immutable",
        "new corroboration",
        "target artifact",
        "observable result",
        "failure",
        "limitation",
        "no performance claim",
        "initial capture",
        "replay",
        "parsed Exit code: 91",
        "no application Output: or Error output:",
        "fresh observations never rewrite accepted observations",
        "canonical Git blob bytes",
        "CARGO_NET_OFFLINE true",
        "Native O0 and O2 exit 91 with empty stdout and stderr",
        "traceability-only replay exclusions",
        "equal SHA-256 and byte size",
    ] {
        if !text
            .to_ascii_lowercase()
            .contains(&required.to_ascii_lowercase())
        {
            return Err(format!("REPRODUCE.md omitted boundary {required:?}"));
        }
    }
    for required in EVIDENCE_PIN_ANCHORS
        .iter()
        .copied()
        .chain(ARTIFACT_NAMES)
        .chain(COMMAND_NAMES)
        .chain(REPLAY_EXCLUSIONS)
    {
        if !text.contains(required) {
            return Err(format!(
                "REPRODUCE.md omitted exact pin or procedure anchor {required:?}"
            ));
        }
    }
    reject_measurement_code(text, "REPRODUCE.md")?;
    reject_affirmative_performance_prose(text, "REPRODUCE.md", true)?;
    Ok(())
}

fn mutate_at_pointer(value: &mut Value, pointer: &str, replacement: Value) {
    let slot = value
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("fixture pointer {pointer} missing"));
    *slot = replacement;
}

fn set_public_raw_streams(manifest: &mut Value, platform: usize, stdout: &[u8], stderr: &[u8]) {
    manifest["platforms"][platform]["commands"]["public_run"]["stdout"] =
        byte_record_from_bytes(stdout);
    manifest["platforms"][platform]["commands"]["public_run"]["stderr"] =
        byte_record_from_bytes(stderr);
}

fn replay_projection(value: &Value) -> Result<String, String> {
    let mut projected = value.clone();
    for excluded in REPLAY_EXCLUSIONS {
        let slot = projected
            .pointer_mut(excluded)
            .ok_or_else(|| format!("closed replay exclusion {excluded} did not resolve"))?;
        *slot = Value::String("<trace-only-observation>".to_owned());
    }
    String::from_utf8(canonical_json_file_bytes(&projected))
        .map_err(|error| format!("canonical replay projection is not UTF-8: {error}"))
}

fn replay_pointer_platform(pointer: &str) -> Result<&'static str, String> {
    if pointer.starts_with("/platforms/0/") {
        Ok(PLATFORM_NAMES[0])
    } else if pointer.starts_with("/platforms/1/") {
        Ok(PLATFORM_NAMES[1])
    } else {
        Err(format!(
            "fresh observation pointer is outside the exact platform projection: {pointer}"
        ))
    }
}

fn fixture_fresh_observations(accepted_manifest: &Value, fresh_manifest: &Value) -> Value {
    let records: Vec<Value> = REPLAY_EXCLUSIONS
        .into_iter()
        .map(|pointer| {
            json!({
                "platform": replay_pointer_platform(pointer).unwrap(),
                "pointer": pointer,
                "value": fresh_manifest.pointer(pointer).unwrap().clone()
            })
        })
        .collect();
    json!({
        "accepted_manifest_sha256": sha256_hex(&canonical_json_file_bytes(accepted_manifest)),
        "records": records,
        "schema_id": "cap024-fresh-observations-v1"
    })
}

fn validate_fresh_observations(
    value: &Value,
    accepted_manifest: &Value,
    fresh_manifest: &Value,
) -> Result<(), String> {
    validate_manifest(accepted_manifest)
        .map_err(|error| format!("accepted manifest for replay is invalid: {error}"))?;
    validate_manifest(fresh_manifest)
        .map_err(|error| format!("fresh manifest for replay is invalid: {error}"))?;
    if replay_projection(accepted_manifest)? != replay_projection(fresh_manifest)? {
        return Err("fresh manifest differs in a claim-bearing replay field".to_owned());
    }
    let root = object(value, "fresh observations")?;
    exact_keys(
        root,
        &["accepted_manifest_sha256", "records", "schema_id"],
        "fresh observations",
    )?;
    exact_string(
        root,
        "schema_id",
        "cap024-fresh-observations-v1",
        "fresh observations",
    )?;
    let expected_manifest_hash = sha256_hex(&canonical_json_file_bytes(accepted_manifest));
    if root.get("accepted_manifest_sha256").and_then(Value::as_str)
        != Some(expected_manifest_hash.as_str())
    {
        return Err(
            "fresh observations do not bind the canonical accepted-manifest bytes".to_owned(),
        );
    }
    let records = array(
        root.get("records")
            .ok_or("fresh observation records missing")?,
        "fresh observation records",
    )?;
    if records.len() != REPLAY_EXCLUSIONS.len() {
        return Err("fresh observations must contain exactly 48 closed trace leaves".to_owned());
    }
    for (index, (record, expected_pointer)) in records.iter().zip(REPLAY_EXCLUSIONS).enumerate() {
        let record = object(record, "fresh observation record")?;
        exact_keys(
            record,
            &["platform", "pointer", "value"],
            "fresh observation record",
        )?;
        exact_string(
            record,
            "pointer",
            expected_pointer,
            &format!("fresh observations[{index}]"),
        )?;
        exact_string(
            record,
            "platform",
            replay_pointer_platform(expected_pointer)?,
            &format!("fresh observations[{index}]"),
        )?;
        if record.get("value") != fresh_manifest.pointer(expected_pointer) {
            return Err(format!(
                "fresh observations[{index}] does not equal the fresh manifest leaf {expected_pointer}"
            ));
        }
    }
    Ok(())
}

fn original_claim_index_fixture() -> Value {
    let mut value = parse_json(
        include_bytes!("../../../claim-verification/claims.json"),
        "checked-in historical claims",
    )
    .expect("parse historical claim fixture");
    value["claims"]
        .as_array_mut()
        .expect("historical claims array")
        .truncate(HISTORICAL_CLAIM_SHA256.len());
    value
}

fn additive_claim_index_fixture() -> Value {
    let original = original_claim_index_fixture();
    let mut result = original.clone();
    result["claims"].as_array_mut().unwrap().push(json!({
        "artifacts": json!([
            format!("{BUNDLE_DIRECTORY}/manifest.json"),
            format!("{BUNDLE_DIRECTORY}/oracle.json"),
            format!("{BUNDLE_DIRECTORY}/REPRODUCE.md")
        ]),
        "claim": format!("Accepted-head CAP-023 correctness and reproducibility record for {SUBJECT_COMMIT}; no performance claim."),
        "id": CLAIM_ID,
        "status": CLAIM_STATUS
    }));
    result
}

#[test]
fn frozen_oracle_is_independent_and_exact() {
    validate_oracle(&fixture_oracle()).expect("canonical oracle fixture");
    let source = git_output(
        &repository_root(),
        &["cat-file", "blob", FROZEN_INPUTS[0].blob],
    )
    .expect("read frozen source blob");
    assert_eq!(
        extract_source_record_literals(&source).expect("extract seven source literals"),
        vec![
            ORDINARY,
            WRAPPING,
            ACTIVATION,
            TIE,
            MALFORMED_FIRST,
            MALFORMED_SECOND,
            MALFORMED_THIRD
        ]
    );
    assert_eq!(
        reference_inference(ORDINARY).result,
        [1, 122, 167, 135, 181, 4940, 5573, 1]
    );
    assert_eq!(
        reference_inference(WRAPPING).result,
        [1, -24, 18, 2_147_483_623, 0, -37, 2_147_483_641, 1]
    );
    assert_eq!(
        reference_inference(ACTIVATION).result,
        [1, -3, 0, 0, 0, 5, 4, 0]
    );
    assert_eq!(reference_inference(TIE).result, [1, 1, 2, 1, 2, 3, 3, 0]);
    for malformed in [MALFORMED_FIRST, MALFORMED_SECOND, MALFORMED_THIRD] {
        assert_eq!(reference_inference(malformed).result, [0; 8]);
    }
}

#[test]
fn pure_manifest_validator_rejects_every_frozen_mutation_class() {
    validate_manifest(&fixture_manifest()).expect("canonical manifest fixture");
    let cases: &[(&str, &str, Value)] = &[
        ("subject", "/subject/commit", json!("0".repeat(40))),
        ("tree", "/subject/tree", json!("0".repeat(40))),
        ("source", "/inputs/0/sha256", json!("0".repeat(64))),
        (
            "platform count",
            "/platforms",
            json!([fixture_platform("linux-x86_64")]),
        ),
        ("platform name", "/platforms/1/name", json!("darwin-x86_64")),
        (
            "malformed hash",
            "/platforms/0/artifacts/llvm/first/sha256",
            json!("ABC"),
        ),
        (
            "size",
            "/platforms/0/artifacts/llvm/second/size",
            json!(9_999),
        ),
        (
            "pair",
            "/platforms/0/artifacts/llvm/second/sha256",
            json!("9".repeat(64)),
        ),
        (
            "arbitrary argv",
            "/platforms/0/commands/llvm_verify_first/argv/0",
            json!("arbitrary"),
        ),
        (
            "machine verifier",
            "/platforms/0/commands/machine_verify_first/argv/1",
            json!("-filetype=asm"),
        ),
        (
            "target command ambient inheritance",
            "/platforms/1/commands/llvm_verify_first/env/inheritance",
            json!("runner-substrate-observation-only"),
        ),
        (
            "producer/path mismatch",
            "/platforms/0/artifacts/llvm/first/path",
            json!("wrong.ll"),
        ),
        (
            "result",
            "/platforms/0/commands/native_o0_first/exit_code",
            json!(1),
        ),
        (
            "lossless command bytes",
            "/platforms/0/commands/public_run/stdout/sha256",
            json!("0".repeat(64)),
        ),
        (
            "parsed application stream",
            "/platforms/0/public_semantics/application_stdout/base64",
            json!("eA=="),
        ),
        ("missing failure", "/failures", Value::Null),
        ("missing limitation", "/limitations", json!([])),
        (
            "replay exclusions",
            "/replay/excluded_paths",
            json!([REPLAY_EXCLUSIONS[0]]),
        ),
        ("tool identity", "/tool/id", json!("unfrozen-tool")),
        (
            "reproduction hash binding",
            "/reproduce/sha256",
            json!("not-a-sha256"),
        ),
        (
            "pinned tool",
            "/platforms/0/toolchain/rust_version",
            json!("stable"),
        ),
        (
            "allowed scope",
            "/scope/0",
            json!("src/compiler/src/lib.rs"),
        ),
    ];
    for (label, pointer, replacement) in cases {
        let mut mutated = fixture_manifest();
        mutate_at_pointer(&mut mutated, pointer, replacement.clone());
        assert!(
            validate_manifest(&mutated).is_err(),
            "{label} mutation unexpectedly passed"
        );
    }
    let mut wrong_tool_version = fixture_manifest();
    wrong_tool_version["platforms"][0]["toolchain"]["tools"]["cargo"]["version"]["parsed"]["version"] =
        json!("1.97.2");
    assert!(validate_manifest(&wrong_tool_version).is_err());

    let mut wrong_rustc_commit = fixture_manifest();
    let rustc_stdout = String::from_utf8(fixture_tool_version_stdout("linux-x86_64", "rustc"))
        .unwrap()
        .replace(RUST_COMMIT, "0000000000000000000000000000000000000000");
    wrong_rustc_commit["platforms"][0]["toolchain"]["tools"]["rustc"]["version"]["stdout"] =
        byte_record_from_bytes(rustc_stdout.as_bytes());
    assert!(validate_manifest(&wrong_rustc_commit).is_err());

    let mut malformed_cargo_commit = fixture_manifest();
    let cargo_stdout = String::from_utf8(fixture_tool_version_stdout("windows-x86_64", "cargo"))
        .unwrap()
        .replace(FIXTURE_CARGO_COMMIT, "not-a-git-commit");
    malformed_cargo_commit["platforms"][1]["toolchain"]["tools"]["cargo"]["version"]["stdout"] =
        byte_record_from_bytes(cargo_stdout.as_bytes());
    assert!(validate_manifest(&malformed_cargo_commit).is_err());

    let mut wrong_clang_banner = fixture_manifest();
    wrong_clang_banner["platforms"][1]["toolchain"]["tools"]["clang"]["version"]["stdout"] =
        byte_record_from_bytes(b"clang version 22.1.9\nTarget: x86_64-pc-windows-msvc\n");
    assert!(validate_manifest(&wrong_clang_banner).is_err());

    let mut wrong_llvm_version = fixture_manifest();
    wrong_llvm_version["platforms"][0]["toolchain"]["tools"]["opt"]["version"]["stdout"] =
        byte_record_from_bytes(b"LLVM (https://llvm.org/):\n  LLVM version 22.1.9\n");
    assert!(validate_manifest(&wrong_llvm_version).is_err());

    let mut wrong_lld_banner = fixture_manifest();
    wrong_lld_banner["platforms"][0]["toolchain"]["tools"]["lld"]["version"]["stdout"] =
        byte_record_from_bytes(b"GNU ld 2.44\n");
    assert!(validate_manifest(&wrong_lld_banner).is_err());

    for (label, stdout, stderr) in [
        (
            "missing public exit line",
            b"Aero execution diagnostic\n".as_slice(),
            b"".as_slice(),
        ),
        (
            "duplicate public exit line",
            b"Exit code: 91\nExit code: 91\n".as_slice(),
            b"".as_slice(),
        ),
        (
            "wrong additional public exit line",
            b"Exit code: 91\nExit code: 1\n".as_slice(),
            b"".as_slice(),
        ),
        (
            "public Output line",
            b"Exit code: 91\nOutput: payload\n".as_slice(),
            b"".as_slice(),
        ),
        (
            "public Error output line",
            b"Exit code: 91\nError output: payload\n".as_slice(),
            b"".as_slice(),
        ),
        (
            "prefixed non-whole exit line",
            b"prefix Exit code: 91\n".as_slice(),
            b"".as_slice(),
        ),
        (
            "exit line moved to stderr",
            b"Aero execution diagnostic\n".as_slice(),
            b"Exit code: 91\n".as_slice(),
        ),
    ] {
        let mut manifest = fixture_manifest();
        set_public_raw_streams(&mut manifest, 0, stdout, stderr);
        assert!(
            validate_manifest(&manifest).is_err(),
            "{label} unexpectedly passed"
        );
    }
    let mut unknown = fixture_manifest();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unknown".to_owned(), Value::Null);
    assert!(validate_manifest(&unknown).is_err());
    let mut equal_cross_os_hashes = fixture_manifest();
    for artifact in ARTIFACT_NAMES {
        for production in ["first", "second"] {
            for field in ["sha256", "size"] {
                let linux = equal_cross_os_hashes
                    .pointer(&format!(
                        "/platforms/0/artifacts/{artifact}/{production}/{field}"
                    ))
                    .unwrap()
                    .clone();
                mutate_at_pointer(
                    &mut equal_cross_os_hashes,
                    &format!("/platforms/1/artifacts/{artifact}/{production}/{field}"),
                    linux,
                );
            }
        }
    }
    validate_manifest(&equal_cross_os_hashes)
        .expect("cross-OS equality is permitted, never required");
    let mut repeated_kind_hash = fixture_manifest();
    for production in ["first", "second"] {
        for field in ["sha256", "size"] {
            let llvm = repeated_kind_hash
                .pointer(&format!("/platforms/0/artifacts/llvm/{production}/{field}"))
                .unwrap()
                .clone();
            mutate_at_pointer(
                &mut repeated_kind_hash,
                &format!("/platforms/0/artifacts/bitcode/{production}/{field}"),
                llvm,
            );
        }
    }
    validate_manifest(&repeated_kind_hash)
        .expect("distinct artifact kinds may coincidentally share a digest and size");
}

#[test]
fn replay_projection_excludes_only_the_closed_traceability_observations() {
    let accepted = fixture_manifest();
    let accepted_projection = replay_projection(&accepted).expect("accepted projection");
    let mut replay = accepted.clone();
    for platform in 0..2 {
        mutate_at_pointer(
            &mut replay,
            &format!("/platforms/{platform}/observations/runner_image"),
            json!("fresh-image"),
        );
        mutate_at_pointer(
            &mut replay,
            &format!("/platforms/{platform}/observations/kernel"),
            json!("fresh-kernel"),
        );
        for production in ["first", "second"] {
            mutate_at_pointer(
                &mut replay,
                &format!("/platforms/{platform}/compiler_executables/{production}/sha256"),
                json!("5".repeat(64)),
            );
            mutate_at_pointer(
                &mut replay,
                &format!("/platforms/{platform}/compiler_executables/{production}/size"),
                json!(654_321_u64),
            );
        }
        for command in [
            "aero_build_llvm_first",
            "aero_build_llvm_second",
            "public_run",
        ] {
            let stdout = if command == "public_run" {
                byte_record_from_bytes(b"fresh diagnostic\nExit code: 91\n")
            } else {
                byte_record(1, ALT_TRACE_SHA256, ALT_TRACE_BASE64)
            };
            mutate_at_pointer(
                &mut replay,
                &format!("/platforms/{platform}/commands/{command}/stdout"),
                stdout,
            );
            mutate_at_pointer(
                &mut replay,
                &format!("/platforms/{platform}/commands/{command}/stderr"),
                byte_record(11, TRACE_SHA256, TRACE_BASE64),
            );
        }
    }
    validate_manifest(&replay).expect("fresh trace observations remain structurally valid");
    assert_eq!(
        replay_projection(&replay).expect("replay projection"),
        accepted_projection,
        "closed trace-only differences must project away"
    );
    let fresh_observations = fixture_fresh_observations(&accepted, &replay);
    validate_fresh_observations(&fresh_observations, &accepted, &replay)
        .expect("exact fresh-observation document");
    let mut missing_fresh_leaf = fresh_observations.clone();
    missing_fresh_leaf["records"].as_array_mut().unwrap().pop();
    assert!(validate_fresh_observations(&missing_fresh_leaf, &accepted, &replay).is_err());
    let mut wrong_manifest_hash = fresh_observations.clone();
    wrong_manifest_hash["accepted_manifest_sha256"] = json!("0".repeat(64));
    assert!(validate_fresh_observations(&wrong_manifest_hash, &accepted, &replay).is_err());
    let mut wrong_order = fresh_observations.clone();
    wrong_order["records"].as_array_mut().unwrap().swap(0, 1);
    assert!(validate_fresh_observations(&wrong_order, &accepted, &replay).is_err());
    let mut arbitrary_value = fresh_observations.clone();
    arbitrary_value["records"][0]["value"] = json!("invented-observation");
    assert!(validate_fresh_observations(&arbitrary_value, &accepted, &replay).is_err());
    let mut wrong_platform = fresh_observations.clone();
    wrong_platform["records"][0]["platform"] = json!(PLATFORM_NAMES[1]);
    assert!(validate_fresh_observations(&wrong_platform, &accepted, &replay).is_err());
    let mut included_difference = replay.clone();
    for production in ["first", "second"] {
        included_difference["platforms"][0]["artifacts"]["llvm"][production]["sha256"] =
            json!("8".repeat(64));
    }
    assert!(
        validate_fresh_observations(
            &fixture_fresh_observations(&accepted, &included_difference),
            &accepted,
            &included_difference,
        )
        .is_err()
    );

    for pointer in REPLAY_EXCLUSIONS {
        let mut one_leaf = accepted.clone();
        mutate_at_pointer(&mut one_leaf, pointer, json!("changed-trace-leaf"));
        assert_eq!(
            replay_projection(&one_leaf).unwrap(),
            accepted_projection,
            "excluded leaf {pointer} remained claim-bearing"
        );
    }

    for (label, pointer, replacement) in [
        (
            "command exit",
            "/platforms/0/commands/public_run/exit_code",
            json!(1),
        ),
        (
            "parsed public semantics",
            "/platforms/0/public_semantics/reported_exit_code",
            json!(1),
        ),
        (
            "pinned tool",
            "/platforms/0/toolchain/llvm_version",
            json!("22.1.9"),
        ),
        (
            "target artifact",
            "/platforms/0/artifacts/llvm/first/sha256",
            json!("8".repeat(64)),
        ),
        (
            "artifact path",
            "/platforms/0/artifacts/llvm/first/path",
            json!("different.ll"),
        ),
        (
            "artifact producer",
            "/platforms/0/artifacts/llvm/first/producer_command",
            json!("clean_before"),
        ),
        (
            "compiler path",
            "/platforms/0/compiler_executables/first/path",
            json!("different-aero"),
        ),
        (
            "tool payload",
            "/platforms/0/toolchain/tools/opt/payload_sha256",
            json!("8".repeat(64)),
        ),
        (
            "public application bytes",
            "/platforms/0/public_semantics/application_stdout/size",
            json!(1),
        ),
    ] {
        let mut changed = accepted.clone();
        mutate_at_pointer(&mut changed, pointer, replacement);
        assert_ne!(
            replay_projection(&changed).expect("claim-bearing projection"),
            accepted_projection,
            "{label} must remain claim-bearing"
        );
    }
}

#[test]
fn pure_oracle_and_claim_validators_reject_drift_and_non_additive_indexing() {
    let mut oracle = fixture_oracle();
    mutate_at_pointer(&mut oracle, "/records/1/result/6", json!(-1));
    assert!(validate_oracle(&oracle).is_err());
    let mut wrong_oracle_id = fixture_oracle();
    mutate_at_pointer(&mut wrong_oracle_id, "/oracle_id", json!("unfrozen-oracle"));
    assert!(validate_oracle(&wrong_oracle_id).is_err());
    let mut duplicate = fixture_oracle();
    mutate_at_pointer(&mut duplicate, "/records/1/name", json!("ordinary"));
    assert!(validate_oracle(&duplicate).is_err());
    let mut unpreserved = fixture_oracle();
    mutate_at_pointer(
        &mut unpreserved,
        "/records/0/source_preserved",
        json!(false),
    );
    assert!(validate_oracle(&unpreserved).is_err());
    let mut source_drift = fixture_oracle();
    mutate_at_pointer(&mut source_drift, "/source/blob", json!("0".repeat(40)));
    assert!(validate_oracle(&source_drift).is_err());
    let additive = additive_claim_index_fixture();
    validate_claim_index(&additive).expect("canonical additive claim index");
    let mut rewritten = additive.clone();
    rewritten["claims"][0]["status"] = json!("rewritten");
    assert!(validate_claim_index(&rewritten).is_err());
    let mut omitted = additive;
    omitted["claims"].as_array_mut().unwrap().remove(0);
    assert!(validate_claim_index(&omitted).is_err());
}

#[test]
fn pure_schema_workflow_tool_and_reproduction_validators_fail_closed() {
    assert!(parse_json(br#"{"duplicate":1,"duplicate":2}"#, "duplicate-key fixture").is_err());
    assert!(parse_json(b"1.0", "floating-number fixture").is_err());
    assert!(parse_json(b"{} trailing", "trailing-byte fixture").is_err());
    let canonical = canonical_json_file_bytes(&fixture_oracle());
    parse_canonical_json_file(&canonical, "canonical oracle").expect("canonical JSON bytes");
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(&canonical);
    assert!(parse_canonical_json_file(&bom, "BOM fixture").is_err());
    assert!(
        parse_canonical_json_file(&canonical[..canonical.len() - 1], "missing LF fixture").is_err()
    );
    let mut crlf = canonical[..canonical.len() - 1].to_vec();
    crlf.extend_from_slice(b"\r\n");
    assert!(parse_canonical_json_file(&crlf, "CRLF fixture").is_err());
    let pretty = canonical_json(&fixture_oracle()).replace(',', ", ");
    assert!(parse_canonical_json_file(format!("{pretty}\n").as_bytes(), "pretty fixture").is_err());
    assert!(
        validate_byte_record(
            &json!({"base64": "Zh==", "sha256": EMPTY_SHA256, "size": 1}),
            "noncanonical-base64 fixture",
        )
        .is_err()
    );

    let schema = fixture_schema();
    validate_schema(&schema).expect("canonical strict schema fixture");
    let mut open_schema = schema.clone();
    mutate_at_pointer(&mut open_schema, "/additionalProperties", json!(true));
    assert!(validate_schema(&open_schema).is_err());
    let mut missing_property = schema;
    missing_property["required"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    assert!(validate_schema(&missing_property).is_err());

    let workflow = fixture_workflow_text();
    validate_workflow(&workflow).expect("canonical workflow fixture");
    assert!(validate_workflow(&workflow.replace(CHECKOUT_ACTION, "actions/checkout@v4")).is_err());
    assert!(validate_workflow(&format!("{workflow}\nimport time")).is_err());
    assert!(validate_workflow(&format!("{workflow}\n# throughput: 2 results/s")).is_err());

    let tool = fixture_tool_text();
    validate_tool(&tool).expect("canonical standard-library tool fixture");
    assert!(validate_tool(&tool.replace(REPLAY_EXCLUSIONS[0], "/too-broad")).is_err());
    assert!(validate_tool(&format!("{tool}\nimport requests")).is_err());
    assert!(validate_tool(&format!("{tool}\n# speedup: 3x")).is_err());
    assert!(validate_tool(&format!("{tool}\n# performance available")).is_err());

    let reproduce = fixture_reproduce_text();
    validate_reproduce(&reproduce).expect("canonical reproduction fixture");
    assert!(
        validate_reproduce(&reproduce.replace("new corroboration", "identical host proof"))
            .is_err()
    );
    assert!(validate_reproduce(&format!("{reproduce}\nperf_counter")).is_err());
    assert!(
        validate_reproduce(&format!(
            "{reproduce}\nLatency: 1 ms; throughput: 2/s; speedup: 3x."
        ))
        .is_err()
    );
    assert!(validate_reproduce(&format!("{reproduce}\nThroughput available.")).is_err());
    validate_reproduce(&format!(
        "{reproduce}\nNo timing, throughput, latency, memory, energy, benchmark, speedup, or performance claim."
    ))
    .expect("explicit negative performance boundary remains permitted");
    validate_scope_paths([
        "TASK_LEDGER.md",
        "src/compiler/tests/cap024_claim_verification_contract_tests.rs",
        "tmp/user-file",
    ])
    .expect("authorized cumulative scope plus preserved user tmp");
    assert!(validate_scope_paths(["TASK_LEDGER.md", "src/compiler/src/lib.rs"]).is_err());
}

#[test]
fn cap024_repository_contract_is_complete() {
    let root = repository_root();
    let mut failures = Vec::new();
    if let Err(error) = validate_frozen_git_inputs(&root) {
        failures.push(error);
    }
    if let Err(error) = validate_cumulative_git_scope(&root) {
        failures.push(error);
    }
    let required_files = [
        SCHEMA_PATH,
        WORKFLOW_PATH,
        TOOL_PATH,
        &format!("{BUNDLE_DIRECTORY}/manifest.json"),
        &format!("{BUNDLE_DIRECTORY}/oracle.json"),
        &format!("{BUNDLE_DIRECTORY}/REPRODUCE.md"),
        CLAIMS_PATH,
    ];
    for relative in required_files {
        if !root.join(relative).is_file() {
            failures.push(format!("missing CAP-024 contract file {relative}"));
        }
    }
    let bundle = root.join(BUNDLE_DIRECTORY);
    if bundle.is_dir() {
        match fs::read_dir(&bundle) {
            Ok(entries) => {
                let mut actual: Vec<String> = entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| entry.file_name().into_string().ok())
                    .collect();
                actual.sort();
                if actual != REQUIRED_BUNDLE_FILES {
                    failures.push(format!(
                        "bundle must contain exactly {REQUIRED_BUNDLE_FILES:?}, received {actual:?}"
                    ));
                }
            }
            Err(error) => failures.push(format!("cannot enumerate bundle: {error}")),
        }
    }
    let validations: &[(&str, fn(&[u8]) -> Result<(), String>)] = &[
        (SCHEMA_PATH, |bytes| {
            validate_schema(&parse_canonical_json_file(bytes, "schema")?)
        }),
        (&format!("{BUNDLE_DIRECTORY}/manifest.json"), |bytes| {
            validate_manifest(&parse_canonical_json_file(bytes, "manifest")?)
        }),
        (&format!("{BUNDLE_DIRECTORY}/oracle.json"), |bytes| {
            validate_oracle(&parse_canonical_json_file(bytes, "oracle")?)
        }),
    ];
    for (relative, validate) in validations {
        if let Ok(bytes) = fs::read(root.join(relative)) {
            if let Err(error) = validate(&bytes) {
                failures.push(format!("{relative}: {error}"));
            }
        }
    }
    let manifest_path = root.join(format!("{BUNDLE_DIRECTORY}/manifest.json"));
    if let Ok(bytes) = fs::read(&manifest_path) {
        match parse_canonical_json_file(&bytes, "manifest hash bindings") {
            Ok(manifest) => {
                if let Err(error) = validate_manifest_file_hashes(&root, &manifest) {
                    failures.push(error);
                }
            }
            Err(error) => failures.push(error),
        }
    }
    if let Ok(bytes) = fs::read(root.join(format!("{BUNDLE_DIRECTORY}/oracle.json"))) {
        match parse_canonical_json_file(&bytes, "oracle source literals") {
            Ok(oracle) => match git_output(&root, &["cat-file", "blob", FROZEN_INPUTS[0].blob])
                .and_then(|source| extract_source_record_literals(&source))
            {
                Ok(literals) => {
                    if let Some(records) = oracle["records"].as_array() {
                        let recorded: Vec<Value> = records
                            .iter()
                            .map(|record| record["source"].clone())
                            .collect();
                        if recorded
                            != literals
                                .into_iter()
                                .map(|record| json!(record))
                                .collect::<Vec<_>>()
                        {
                            failures.push("oracle records do not equal the seven statically extracted source literals".to_owned());
                        }
                    } else {
                        failures.push("oracle records are not an array".to_owned());
                    }
                }
                Err(error) => failures.push(error),
            },
            Err(error) => failures.push(error),
        }
    }
    if let Ok(bytes) = fs::read(root.join(CLAIMS_PATH)) {
        match parse_json(&bytes, "claim index") {
            Ok(index) => {
                if let Err(error) = validate_claim_index(&index) {
                    failures.push(format!("{CLAIMS_PATH}: {error}"));
                }
            }
            Err(error) => failures.push(error),
        }
    }
    for (relative, validate) in [
        (
            WORKFLOW_PATH,
            validate_workflow as fn(&str) -> Result<(), String>,
        ),
        (TOOL_PATH, validate_tool),
        (
            &format!("{BUNDLE_DIRECTORY}/REPRODUCE.md"),
            validate_reproduce,
        ),
    ] {
        if let Ok(text) = fs::read_to_string(root.join(relative)) {
            if let Err(error) = validate(&text) {
                failures.push(format!("{relative}: {error}"));
            }
        }
    }
    if root.join(TOOL_PATH).is_file() && root.join(BUNDLE_DIRECTORY).is_dir() {
        let accepted_manifest_path = root.join(BUNDLE_DIRECTORY).join("manifest.json");
        let accepted_manifest_before = fs::read(&accepted_manifest_path).ok();
        for arguments in [
            vec!["--mode", "self-test"],
            vec!["--mode", "validate", "--bundle", BUNDLE_DIRECTORY],
            vec![
                "--mode",
                "replay",
                "--bundle",
                BUNDLE_DIRECTORY,
                "--verify-only",
            ],
        ] {
            if let Err(error) = run_python_contract(&root, &arguments) {
                failures.push(error);
            }
        }
        for case in ["schema_unknown", "schema_float", "schema_duplicate"] {
            if let Err(error) =
                run_python_contract(&root, &["--mode", "self-test", "--negative-case", case])
            {
                failures.push(format!(
                    "CAP-024 Python canonical-schema negative {case} failed: {error}"
                ));
            }
        }
        match create_unique_temp_directory("aero-cap024-contract-negative") {
            Ok(adversarial_root) => {
                for case in [
                    "oracle_drift",
                    "artifact_pair",
                    "artifact_path",
                    "artifact_producer",
                    "command_argv",
                    "command_env",
                    "public_missing",
                    "public_duplicate",
                    "public_wrong_exit",
                    "public_output",
                    "public_error",
                    "public_prefixed_exit",
                    "public_exit_in_stderr",
                    "extra_bundle_file",
                    "reproduce_drift",
                ] {
                    let bundle = adversarial_root.join(case);
                    match prepare_adversarial_bundle(&root, &bundle, case) {
                        Ok(()) => match bundle.to_str() {
                            Some(bundle) => {
                                if let Err(error) = run_python_contract_rejects(
                                    &root,
                                    &["--mode", "validate", "--bundle", bundle],
                                ) {
                                    failures
                                        .push(format!("CAP-024 Python accepted {case}: {error}"));
                                }
                            }
                            None => failures
                                .push(format!("adversarial bundle path for {case} is not UTF-8")),
                        },
                        Err(error) => failures.push(error),
                    }
                }

                match accepted_manifest_before
                    .as_deref()
                    .ok_or_else(|| "cannot read accepted manifest before replay".to_owned())
                    .and_then(|bytes| parse_canonical_json_file(bytes, "accepted replay manifest"))
                {
                    Ok(accepted) => {
                        let fresh = fresh_replay_manifest(&accepted);
                        let observations = fixture_fresh_observations(&accepted, &fresh);
                        let fresh_path = adversarial_root.join("fresh-manifest.json");
                        let observations_path = adversarial_root.join("fresh-observations.json");
                        if let Err(error) = write_canonical_value(&fresh_path, &fresh)
                            .and_then(|()| write_canonical_value(&observations_path, &observations))
                        {
                            failures.push(error);
                        } else if let (Some(fresh_path), Some(observations_path)) =
                            (fresh_path.to_str(), observations_path.to_str())
                        {
                            if let Err(error) = run_python_contract(
                                &root,
                                &[
                                    "--mode",
                                    "replay",
                                    "--bundle",
                                    BUNDLE_DIRECTORY,
                                    "--fresh-manifest",
                                    fresh_path,
                                    "--fresh-observations",
                                    observations_path,
                                    "--verify-only",
                                ],
                            ) {
                                failures.push(format!(
                                    "CAP-024 Python rejected closed excluded replay leaves: {error}"
                                ));
                            }

                            let mut included = fresh.clone();
                            for production in ["first", "second"] {
                                included["platforms"][0]["artifacts"]["llvm"][production]["sha256"] =
                                    json!("8".repeat(64));
                            }
                            let included_observations =
                                fixture_fresh_observations(&accepted, &included);
                            let included_path = adversarial_root.join("included-manifest.json");
                            let included_observations_path =
                                adversarial_root.join("included-observations.json");
                            if let Err(error) = write_canonical_value(&included_path, &included)
                                .and_then(|()| {
                                    write_canonical_value(
                                        &included_observations_path,
                                        &included_observations,
                                    )
                                })
                            {
                                failures.push(error);
                            } else if let (Some(included_path), Some(included_observations_path)) =
                                (included_path.to_str(), included_observations_path.to_str())
                                && let Err(error) = run_python_contract_rejects(
                                    &root,
                                    &[
                                        "--mode",
                                        "replay",
                                        "--bundle",
                                        BUNDLE_DIRECTORY,
                                        "--fresh-manifest",
                                        included_path,
                                        "--fresh-observations",
                                        included_observations_path,
                                        "--verify-only",
                                    ],
                                )
                            {
                                failures.push(format!(
                                    "CAP-024 Python accepted a claim-bearing replay difference: {error}"
                                ));
                            }

                            for (case, mutate) in [
                                ("wrong-accepted-hash", 0_u8),
                                ("wrong-record-order", 1),
                                ("arbitrary-record-value", 2),
                                ("wrong-record-platform", 3),
                            ] {
                                let mut changed = observations.clone();
                                match mutate {
                                    0 => {
                                        changed["accepted_manifest_sha256"] = json!("0".repeat(64));
                                    }
                                    1 => changed["records"].as_array_mut().unwrap().swap(0, 1),
                                    2 => {
                                        changed["records"][0]["value"] = json!("arbitrary");
                                    }
                                    3 => {
                                        changed["records"][0]["platform"] =
                                            json!(PLATFORM_NAMES[1]);
                                    }
                                    _ => unreachable!(),
                                }
                                let changed_path = adversarial_root
                                    .join(format!("fresh-observations-{case}.json"));
                                if let Err(error) = write_canonical_value(&changed_path, &changed) {
                                    failures.push(error);
                                } else if let Some(changed_path) = changed_path.to_str()
                                    && let Err(error) = run_python_contract_rejects(
                                        &root,
                                        &[
                                            "--mode",
                                            "replay",
                                            "--bundle",
                                            BUNDLE_DIRECTORY,
                                            "--fresh-manifest",
                                            fresh_path,
                                            "--fresh-observations",
                                            changed_path,
                                            "--verify-only",
                                        ],
                                    )
                                {
                                    failures
                                        .push(format!("CAP-024 Python accepted {case}: {error}"));
                                }
                            }
                        } else {
                            failures
                                .push("fresh replay fixture paths are not valid UTF-8".to_owned());
                        }
                    }
                    Err(error) => failures.push(error),
                }

                if let Err(error) = fs::remove_dir_all(&adversarial_root) {
                    failures.push(format!(
                        "cannot remove adversarial CAP-024 tree {}: {error}",
                        adversarial_root.display()
                    ));
                }
            }
            Err(error) => failures.push(error),
        }
        if let Some(before) = accepted_manifest_before {
            match fs::read(&accepted_manifest_path) {
                Ok(after) if after == before => {}
                Ok(_) => failures.push(
                    "CAP-024 Python interface rewrote immutable accepted manifest bytes".to_owned(),
                ),
                Err(error) => failures.push(format!(
                    "cannot reread accepted manifest after behavioral checks: {error}"
                )),
            }
        }
    }
    assert!(
        failures.is_empty(),
        "CAP-024 repository evidence contract is intentionally red until the authorized schema/tool/workflow/bundle/index implementation exists:\n{}",
        failures.join("\n")
    );
}
