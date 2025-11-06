use std::{collections::HashMap, str};

use crate::{jsonerr, types::*};

struct Parser<'a> {
    buf: &'a [u8],
    pos: usize,
    cache: Option<Result<Token, JSONError>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            buf: input,
            pos: 0,
            cache: None,
        }
    }

    fn current(&self) -> Option<u8> {
        if self.pos < self.buf.len() {
            return Some(self.buf[self.pos]);
        }
        None
    }

    fn read_byte(&mut self) -> Option<u8> {
        if self.pos < self.buf.len() {
            let b = self.buf[self.pos];
            self.pos += 1;
            return Some(b);
        }
        None
    }

    fn is_whitespace(b: u8) -> bool {
        matches!(b, b' ' | b'\n' | b'\r' | b'\t')
    }

    fn is_delimiter(b: u8) -> bool {
        matches!(b, b'{' | b'}' | b'[' | b']' | b':' | b',')
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.current() {
            if !Self::is_whitespace(b) {
                break;
            }
            self.read_byte();
        }
    }

    fn read_string(&mut self) -> Result<String, JSONError> {
        let mut s = String::new();
        loop {
            match self.read_byte().ok_or(jsonerr!("EOF"))? {
                b'"' => break,
                b'\\' => match self.read_byte().ok_or(jsonerr!("EOF"))? {
                    b if matches!(b, b'"' | b'\\' | b'/') => {
                        s.push(b as char);
                    }
                    b'b' => s.push('\x08'),
                    b'f' => s.push('\x0c'),
                    b'n' => s.push('\n'),
                    b'r' => s.push('\r'),
                    b't' => s.push('\t'),
                    b'u' => {
                        let mut hex = Vec::new();

                        for _ in 0..4 {
                            hex.push(self.read_byte().ok_or(jsonerr!("EOF"))?);
                        }

                        let code = u32::from_str_radix(str::from_utf8(&hex)?, 16)?;
                        let ch =
                            char::from_u32(code).ok_or(jsonerr!("invalid unicode code point"))?;

                        s.push(ch);
                    }
                    b => return Err(jsonerr!("expected \", \\, /, b, f, n, r, t or u, got {b}")),
                },
                x => {
                    s.push(x as char);
                }
            }
        }

        Ok(s)
    }

    fn read_object(&mut self) -> Result<HashMap<String, Value>, JSONError> {
        let mut d = HashMap::new();

        loop {
            match self.peek_token() {
                Ok(Token::RBrace) => {
                    _ = self.read_token();
                    break;
                }
                Err(err) => return Err(err.clone()),
                _ => {}
            }

            let k = match self.read_token()? {
                Token::String(s) => s,
                x => return Err(jsonerr!("expected string, got {x:?}")),
            };

            match self.read_token()? {
                Token::Colon => {
                    // ok
                }
                x => return Err(jsonerr!("expected colon, got {x:?}")),
            }

            let v = self.read_value()?;

            d.insert(k, v);

            match self.read_token()? {
                Token::Comma => {
                    // ok
                }
                Token::RBrace => break,
                x => return Err(jsonerr!("expected comma or }}, got {x:?}")),
            }
        }

        Ok(d)
    }

    fn read_array(&mut self) -> Result<Vec<Value>, JSONError> {
        let mut arr = Vec::new();

        loop {
            match self.peek_token() {
                Ok(Token::RAngle) => {
                    _ = self.read_token();
                    break;
                }
                Err(err) => return Err(err.clone()),
                _ => {}
            }

            arr.push(self.read_value()?);
            match self.read_token()? {
                Token::Comma => {}
                Token::RAngle => break,
                x => return Err(jsonerr!("expected comma or ], got {x:?}")),
            }
        }

        Ok(arr)
    }

    fn read_term(&mut self) -> Result<Token, JSONError> {
        let start = self.pos;
        while let Some(b) = self.current() {
            if Self::is_whitespace(b) || Self::is_delimiter(b) {
                break;
            }
            self.read_byte();
        }
        let end = self.pos;

        match &self.buf[start..end] {
            b"true" => Ok(Token::Boolean(true)),
            b"false" => Ok(Token::Boolean(false)),
            b"null" => Ok(Token::Null),
            x => Ok(Token::Number(str::from_utf8(x)?.parse()?)),
        }
    }

    fn peek_token(&mut self) -> &Result<Token, JSONError> {
        let t = self.read_token();
        self.cache = Some(t);

        match &self.cache {
            Some(t) => t,
            None => unreachable!(),
        }
    }

    fn read_token(&mut self) -> Result<Token, JSONError> {
        if let Some(t) = self.cache.take() {
            return t;
        }
        self.skip_whitespace();

        match self.read_byte().ok_or(jsonerr!("EOF"))? {
            b'"' => Ok(Token::String(self.read_string()?)),
            b'[' => Ok(Token::LAngle),
            b']' => Ok(Token::RAngle),
            b'{' => Ok(Token::LBrace),
            b'}' => Ok(Token::RBrace),
            b',' => Ok(Token::Comma),
            b':' => Ok(Token::Colon),
            _ => {
                self.pos -= 1;
                self.read_term()
            }
        }
    }

    fn read_value(&mut self) -> Result<Value, JSONError> {
        match self.read_token()? {
            Token::LBrace => Ok(Value::Object(self.read_object()?)),
            Token::LAngle => Ok(Value::Array(self.read_array()?)),
            Token::String(s) => Ok(Value::String(s)),
            Token::Number(n) => Ok(Value::Number(n)),
            Token::Boolean(b) => Ok(Value::Boolean(b)),
            Token::Null => Ok(Value::Null),
            x => Err(jsonerr!("unexpected token {x:?}")),
        }
    }
}

pub fn deserialize(json: &[u8]) -> Result<Value, JSONError> {
    Parser::new(json).read_value()
}

pub fn serialize(val: Value) -> String {
    val.to_json()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json() {
        let input = r#"{
          "string": "Hello 🌍",
          "escaped": "\"quoted\"\\\\\\n\\b\\u00F6",
          "empty_string": "",
          "number_int": 11,
          "number_float": -3.111111111111,
          "number_exp": 6.022e23,
          "true": true,
          "false": false,
          "null_value": null,
          "array_mixed": [1, "two", null, true, {"nested": []}],
          "object_nested": {
            "a": 1,
            "b": {
              "c": {
                "d": "deep"
              }
            }
          },
          "unicode_key_üñîçødê": "works!",
          "empty_array": [],
          "empty_array_nested": [[[],[],[[]]]],
          "empty_object": {}
        }
"#;

        assert!(deserialize(input.as_bytes()).is_ok())
    }
}
