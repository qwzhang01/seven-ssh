use encoding_rs::Encoding;

pub fn get_encoding(charset: &str) -> Option<&'static Encoding> {
    let normalized = charset.to_uppercase();
    match normalized.as_str() {
        "UTF-8" | "UTF8" => None, // No conversion needed
        _ => Encoding::for_label(charset.as_bytes()),
    }
}

/// Decode bytes from the given charset to UTF-8.
/// Returns the original bytes if charset is UTF-8 or unknown.
pub fn decode_to_utf8(bytes: &[u8], charset: &str) -> Vec<u8> {
    match get_encoding(charset) {
        Some(encoding) => {
            let (cow, _, _) = encoding.decode(bytes);
            cow.as_bytes().to_vec()
        }
        None => bytes.to_vec(),
    }
}

/// Encode UTF-8 bytes to the target charset.
/// Returns the original bytes if charset is UTF-8 or unknown.
pub fn encode_from_utf8(bytes: &[u8], charset: &str) -> Vec<u8> {
    match get_encoding(charset) {
        Some(encoding) => {
            let input = String::from_utf8_lossy(bytes);
            let (cow, _, _) = encoding.encode(&input);
            cow.to_vec()
        }
        None => bytes.to_vec(),
    }
}
