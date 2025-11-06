use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct JSONError(pub(crate) String);

#[macro_export]
macro_rules! jsonerr {
    ($($arg:tt)*) => {
        JSONError(format!($($arg)*))
    };
}

impl<E> From<E> for JSONError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(e: E) -> Self {
        Self(e.to_string())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Token {
    LBrace,
    RBrace,
    LAngle,
    RAngle,
    Comma,
    Colon,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    // basic types
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    // compund types
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn to_json(&self) -> String {
        match self {
            Value::String(s) => Self::string_repr(s),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_json()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", Self::string_repr(k), v.to_json()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
        }
    }

    fn string_repr(s: &str) -> String {
        let mut buf = "\"".to_string();
        for c in s.chars() {
            match c {
                '"' | '\\' | '/' => {
                    buf.push('\\');
                    buf.push(c);
                }
                '\x08' => {
                    buf.push_str("\\b");
                }
                '\x0c' => {
                    buf.push_str("\\f");
                }
                '\n' => {
                    buf.push_str("\\n");
                }
                '\r' => {
                    buf.push_str("\\r");
                }
                '\t' => {
                    buf.push_str("\\t");
                }
                _ if c.is_ascii_control() => {
                    panic!("control characters not allowed")
                }
                _ if c.is_ascii() => {
                    buf.push(c);
                }
                _ => {
                    buf.push_str(&format!("\\u{:04X}", c as u32));
                }
            }
        }

        buf.push('"');

        buf
    }
}
