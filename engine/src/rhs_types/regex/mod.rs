use cfg_if::cfg_if;
use lex::{expect, span, Lex, LexErrorKind, LexResult};
use serde::{Serialize, Serializer};
use std::fmt::{self, Debug, Formatter};

cfg_if! {
    if #[cfg(feature = "regex")] {
        mod imp_real;
        pub use self::imp_real::*;
    } else {
        mod imp_stub;
        pub use self::imp_stub::*;
    }
}

impl PartialEq for Regex {
    fn eq(&self, other: &Regex) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Regex {}

impl Debug for Regex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'i> Lex<'i> for Regex {
    fn lex(input: &str) -> LexResult<'_, Self> {
        let input = expect(input, "\"")?;
        let mut regex_buf = String::new();
        let mut in_char_class = false;
        let (regex_str, input) = {
            let mut iter = input.chars();
            loop {
                let before_char = iter.as_str();
                match iter
                    .next()
                    .ok_or_else(|| (LexErrorKind::MissingEndingQuote, input))?
                {
                    '\\' => {
                        if let Some(c) = iter.next() {
                            if in_char_class || c != '"' {
                                regex_buf.push('\\');
                            }
                            regex_buf.push(c);
                        }
                    }
                    '"' if !in_char_class => {
                        break (span(input, before_char), iter.as_str());
                    }
                    '[' if !in_char_class => {
                        in_char_class = true;
                        regex_buf.push('[');
                    }
                    ']' if in_char_class => {
                        in_char_class = false;
                        regex_buf.push(']');
                    }
                    c => {
                        regex_buf.push(c);
                    }
                };
            }
        };
        match Regex::new(&regex_buf) {
            Ok(regex) => Ok((regex, input)),
            Err(err) => Err((LexErrorKind::ParseRegex(err), regex_str)),
        }
    }
}

impl Serialize for Regex {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(ser)
    }
}

#[test]
fn test() {
    use serde_json::{json, to_value as json};

    let expr = assert_ok!(
        Regex::lex(r#""[a-z"\]]+\d{1,10}\"";"#),
        Regex::new(r#"[a-z"\]]+\d{1,10}""#).unwrap(),
        ";"
    );

    assert_eq!(json(&expr).unwrap(), json!(r#"[a-z"\]]+\d{1,10}""#));

    assert_err!(
        Regex::lex(r#""abcd\"#),
        LexErrorKind::MissingEndingQuote,
        "abcd\\"
    );
}