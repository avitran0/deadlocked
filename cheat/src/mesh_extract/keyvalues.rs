/// minimal KeyValues (VDF) parser; returns a flat (key, value) list since
/// KeyValues allows duplicate keys at the same level
pub enum Value {
    Str(String),
    Block(Vec<(String, Value)>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            Value::Block(_) => None,
        }
    }

    pub fn as_block(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Block(b) => Some(b),
            Value::Str(_) => None,
        }
    }
}

enum Token {
    Str(String),
    Open,
    Close,
}

fn tokenize(text: &str) -> Vec<Token> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'{' => {
                tokens.push(Token::Open);
                i += 1;
            }
            b'}' => {
                tokens.push(Token::Close);
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'"' => {
                let mut buf = Vec::new();
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        buf.push(bytes[i + 1]);
                        i += 2;
                    } else {
                        buf.push(bytes[i]);
                        i += 1;
                    }
                }
                i += 1;
                tokens.push(Token::Str(String::from_utf8_lossy(&buf).into_owned()));
            }
            _ => i += 1,
        }
    }
    tokens
}

fn parse_block(tokens: &[Token], mut i: usize) -> (Vec<(String, Value)>, usize) {
    let mut pairs = Vec::new();
    while i < tokens.len() {
        match &tokens[i] {
            Token::Close => return (pairs, i + 1),
            Token::Str(key) => {
                let key = key.clone();
                i += 1;
                match tokens.get(i) {
                    Some(Token::Open) => {
                        let (block, next) = parse_block(tokens, i + 1);
                        pairs.push((key, Value::Block(block)));
                        i = next;
                    }
                    Some(Token::Str(value)) => {
                        pairs.push((key, Value::Str(value.clone())));
                        i += 1;
                    }
                    _ => break,
                }
            }
            Token::Open => i += 1,
        }
    }
    (pairs, i)
}

pub fn parse(text: &str) -> Vec<(String, Value)> {
    let tokens = tokenize(text);
    parse_block(&tokens, 0).0
}
