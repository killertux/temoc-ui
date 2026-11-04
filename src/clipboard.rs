//! Copying to the clipboard through the terminal, with OSC 52.

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// What to write to the terminal to put `text` on the clipboard.
#[must_use]
pub fn copy_sequence(text: &str) -> Vec<u8> {
    format!("\x1b]52;c;{}\x07", base64(text.as_bytes())).into_bytes()
}

fn base64(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let group = chunk
            .iter()
            .chain([0, 0].iter())
            .take(3)
            .fold(0_u32, |group, byte| (group << 8) | u32::from(*byte));
        for position in 0..4 {
            if position <= chunk.len() {
                let sextet = (group >> (18 - 6 * position)) & 0x3f;
                encoded.extend(
                    usize::try_from(sextet)
                        .ok()
                        .and_then(|sextet| BASE64.get(sextet))
                        .map(|letter| char::from(*letter)),
                );
            } else {
                encoded.push('=');
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_every_length_of_final_group() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"a"), "YQ==");
        assert_eq!(base64(b"ab"), "YWI=");
        assert_eq!(base64(b"abc"), "YWJj");
        assert_eq!(base64(b"abcd"), "YWJjZA==");
    }

    #[test]
    fn encodes_every_bit_of_a_byte() {
        assert_eq!(base64(&[0xff, 0xff, 0xff]), "////");
        assert_eq!(base64(&[0xfb, 0xef, 0xbe]), "++++");
    }

    #[test]
    fn a_sequence_carries_utf8() {
        assert_eq!(
            copy_sequence("• Getting started"),
            b"\x1b]52;c;4oCiIEdldHRpbmcgc3RhcnRlZA==\x07"
        );
    }
}
