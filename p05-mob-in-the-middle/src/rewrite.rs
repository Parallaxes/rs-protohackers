use regex::bytes::Regex;

pub const TONY_ADDRESS: &[u8] = b"7YWHMfk9JZe0LM0g1ZauHuiSxhI";
lazy_static::lazy_static! {
    pub static ref BOGUSCOIN_REGEX: Regex =
        Regex::new(r"\b7[a-zA-Z0-9]{25,34}\b").unwrap();
}

pub fn rewrite_boguscoin(input: &[u8]) -> Vec<u8> {
    BOGUSCOIN_REGEX
        .replace_all(input, |caps: &regex::bytes::Captures| {
            // Check boundaries manually
            let m = caps.get(0).unwrap();
            let start = m.start();
            let end = m.end();
            let before = if start == 0 { b' ' } else { input[start - 1] };
            let after = if end == input.len() { b' ' } else { input[end] };
            let valid_before = start == 0 || before == b' ' || before == b'\n';
            let valid_after = end == input.len() || after == b' ' || after == b'\n';
            if valid_before && valid_after {
                TONY_ADDRESS.to_vec()
            } else {
                m.as_bytes().to_vec()
            }
        })
        .to_vec()
}