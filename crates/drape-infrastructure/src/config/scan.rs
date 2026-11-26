use super::{MAX_DEPTH, MAX_DOCUMENT_BYTES, MAX_STRING_BYTES, MAX_VALUES};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct TooLarge;

enum Frame {
    Object { keys: HashSet<String>, key: bool },
    Array,
}

/// Why a document cannot be rewritten without loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Scan {
    Clear,
    /// Repeated property; the parser still accepts it.
    Duplicate,
    /// Content the parser may reject or reinterpret, so a parse failure proves nothing.
    Opaque,
}

pub(super) fn scan(bytes: &[u8]) -> Result<Scan, TooLarge> {
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(TooLarge);
    }
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    let mut stack = Vec::new();
    let mut values = 0;
    let mut found = Scan::Clear;
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'{' | b'[' => {
                count_value(&mut values)?;
                if stack.len() >= MAX_DEPTH {
                    return Err(TooLarge);
                }
                stack.push(if byte == b'{' {
                    Frame::Object {
                        keys: HashSet::new(),
                        key: true,
                    }
                } else {
                    Frame::Array
                });
                index += 1;
            }
            b'}' | b']' => {
                let _ = stack.pop();
                index += 1;
            }
            b',' => {
                if let Some(Frame::Object { key, .. }) = stack.last_mut() {
                    *key = true;
                }
                index += 1;
            }
            b'"' => {
                let start = index;
                let (end, unpaired) = string_end(bytes, start)?;
                if unpaired {
                    found = Scan::Opaque;
                }
                let is_key = matches!(stack.last(), Some(Frame::Object { key: true, .. }));
                if is_key && let Some(Frame::Object { keys, key }) = stack.last_mut() {
                    // Valid objects have fewer keys than the document's value budget.
                    if keys.len() >= MAX_VALUES {
                        return Err(TooLarge);
                    }
                    if let Ok(decoded) = serde_json::from_slice::<String>(&bytes[start..end]) {
                        if decoded.starts_with("$serde_json::private::") {
                            found = Scan::Opaque;
                        }
                        if !keys.insert(decoded) {
                            found = found.max(Scan::Duplicate);
                        }
                    }
                    *key = false;
                } else if !is_key {
                    count_value(&mut values)?;
                }
                index = end;
            }
            b':' | b' ' | b'\t' | b'\r' | b'\n' => index += 1,
            _ => {
                count_value(&mut values)?;
                index += 1;
                while bytes.get(index).is_some_and(|byte| {
                    !matches!(
                        byte,
                        b'{' | b'['
                            | b'}'
                            | b']'
                            | b','
                            | b':'
                            | b'"'
                            | b' '
                            | b'\t'
                            | b'\r'
                            | b'\n'
                    )
                }) {
                    index += 1;
                }
            }
        }
    }
    Ok(found)
}

fn count_value(values: &mut usize) -> Result<(), TooLarge> {
    if *values >= MAX_VALUES {
        return Err(TooLarge);
    }
    *values += 1;
    Ok(())
}

fn string_end(bytes: &[u8], start: usize) -> Result<(usize, bool), TooLarge> {
    let mut index = start + 1;
    let mut high = false;
    let mut ambiguous = false;
    while let Some(&byte) = bytes.get(index) {
        if index - start - 1 > MAX_STRING_BYTES {
            return Err(TooLarge);
        }
        if byte == b'"' {
            return Ok((index + 1, ambiguous || high));
        }
        if byte == b'\\' {
            if let Some(unit) = unicode_escape(bytes, index) {
                match unit {
                    0xd800..=0xdbff => {
                        ambiguous |= high;
                        high = true;
                    }
                    0xdc00..=0xdfff => {
                        ambiguous |= !high;
                        high = false;
                    }
                    _ => {
                        ambiguous |= high;
                        high = false;
                    }
                }
                index += 6;
                continue;
            }
            ambiguous |= high;
            high = false;
            index += if index + 1 < bytes.len() { 2 } else { 1 };
        } else {
            ambiguous |= high;
            high = false;
            index += 1;
        }
    }
    if index - start - 1 > MAX_STRING_BYTES {
        return Err(TooLarge);
    }
    Ok((index, ambiguous || high))
}

fn unicode_escape(bytes: &[u8], index: usize) -> Option<u16> {
    let escape = bytes.get(index..index + 6)?;
    if !escape.starts_with(b"\\u") {
        return None;
    }
    escape[2..].iter().copied().try_fold(0_u16, |value, byte| {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return None,
        };
        Some(value * 16 + u16::from(digit))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_keys_are_scoped_to_their_object() {
        assert_eq!(scan(br#"{"a":0,"\u0061":1}"#), Ok(Scan::Duplicate));
        assert_eq!(scan(br#"{"a":{"a":0},"b":{"a":1}}"#), Ok(Scan::Clear));
        assert_eq!(
            scan(br#"{"$serde_json::private::Number":0}"#),
            Ok(Scan::Opaque)
        );
        assert_eq!(
            scan(br#"{"\u0024serde_json::private::RawValue":0}"#),
            Ok(Scan::Opaque)
        );
        assert_eq!(
            scan(br#"["$serde_json::private::Number"]"#),
            Ok(Scan::Clear)
        );
    }

    #[test]
    fn surrogate_pairs_and_escaped_backslashes_are_distinguished() {
        let unpaired: [&[u8]; 5] = [
            br#""\ud800""#,
            br#""\udfff""#,
            br#""\ud800x""#,
            br#""\ud800\ud800\udc00""#,
            br#"{"\ud800":0}"#,
        ];
        for bytes in unpaired {
            assert_eq!(scan(bytes), Ok(Scan::Opaque));
        }
        let paired: [&[u8]; 4] = [
            br#""\ud800\udfff""#,
            br#""\\ud800""#,
            br#""\ud800\udc00\udbff\udfff""#,
            br#""quote\"slash\\""#,
        ];
        for bytes in paired {
            assert_eq!(scan(bytes), Ok(Scan::Clear));
        }
    }

    #[test]
    fn exact_depth_string_and_document_limits_are_allowed() {
        let nested = format!("{}{}", "[".repeat(MAX_DEPTH), "]".repeat(MAX_DEPTH));
        assert_eq!(scan(nested.as_bytes()), Ok(Scan::Clear));
        assert_eq!(scan(format!("[{nested}]").as_bytes()), Err(TooLarge));
        let string = format!("\"{}\"", "a".repeat(MAX_STRING_BYTES));
        assert_eq!(scan(string.as_bytes()), Ok(Scan::Clear));
        assert_eq!(
            scan(format!("\"{}\"", "a".repeat(MAX_STRING_BYTES + 1)).as_bytes()),
            Err(TooLarge)
        );
        let document = vec![b' '; MAX_DOCUMENT_BYTES].into_boxed_slice();
        assert_eq!(scan(&document), Ok(Scan::Clear));
        let oversized = vec![b' '; MAX_DOCUMENT_BYTES + 1].into_boxed_slice();
        assert_eq!(scan(&oversized), Err(TooLarge));
    }

    #[test]
    fn keys_do_not_consume_the_value_budget() {
        let object = format!(
            "{{{}}}",
            (0..MAX_VALUES - 1)
                .map(|n| format!("\"{n}\":null"))
                .collect::<Vec<_>>()
                .join(",")
        );
        assert_eq!(scan(object.as_bytes()), Ok(Scan::Clear));
        let array = format!("[{}0]", "0,".repeat(MAX_VALUES));
        assert_eq!(scan(array.as_bytes()), Err(TooLarge));
    }

    #[test]
    fn malformed_bytes_and_bom_are_safe() {
        assert_eq!(scan(b"\xef\xbb\xbf{\"a\":true}"), Ok(Scan::Clear));
        let malformed: [&[u8]; 7] = [
            b"",
            b"]}{[",
            b"\"\\",
            b"\"\\uD8",
            b"\xff\"\xfe",
            b"{\"a\"",
            b"[true,false,null,-1.2e3]",
        ];
        for bytes in malformed {
            assert!(scan(bytes).is_ok());
        }
        assert_eq!(
            scan(format!("\"{}\\u1234\"", "a".repeat(MAX_STRING_BYTES - 5)).as_bytes()),
            Err(TooLarge)
        );
    }
}
