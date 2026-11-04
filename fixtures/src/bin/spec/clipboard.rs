//! Reading the OSC 52 clipboard sequences written to the terminal.

const START: &[u8] = b"\x1b]52;c;";
const END: u8 = 0x07;

/// The base64 payload of the last clipboard sequence in `output`.
pub fn last_payload(output: &[u8]) -> Option<&[u8]> {
    let start = output
        .windows(START.len())
        .rposition(|window| window == START)?
        .checked_add(START.len())?;
    let rest = output.get(start..)?;
    let end = rest.iter().position(|byte| *byte == END)?;
    rest.get(..end)
}

/// Writes a sequence the way the specification spells it.
pub fn describe(payload: &[u8]) -> String {
    format!("ESC]52;c;{}BEL", String::from_utf8_lossy(payload))
}

/// Decodes a base64 payload into the text it carries.
pub fn decode(payload: &[u8]) -> Result<String, String> {
    let mut bytes = Vec::with_capacity(payload.len());
    let mut bits: u32 = 0;
    let mut bit_count = 0;
    for byte in payload.iter().copied().filter(|byte| *byte != b'=') {
        let value = sextet(byte).ok_or_else(|| format!("invalid base64 byte {byte}"))?;
        bits = (bits << 6) | value;
        bit_count += 6;
        if bit_count >= 8 {
            bit_count -= 8;
            bytes.extend(u8::try_from((bits >> bit_count) & 0xff).ok());
        }
    }
    String::from_utf8(bytes).map_err(|error| error.to_string())
}

fn sextet(byte: u8) -> Option<u32> {
    let value = match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => return None,
    };
    Some(u32::from(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_text_with_and_without_padding() {
        assert_eq!(
            decode(b"UnVuIHRoZSBpbnN0YWxsZXI=").unwrap(),
            "Run the installer"
        );
        assert_eq!(decode(b"RG9jcw==").unwrap(), "Docs");
        assert_eq!(decode(b"YWJj").unwrap(), "abc");
    }

    #[test]
    fn decodes_utf8() {
        assert_eq!(
            decode("4oCiIEdldHRpbmcgc3RhcnRlZA==".as_bytes()).unwrap(),
            "• Getting started"
        );
    }

    #[test]
    fn rejects_invalid_base64() {
        assert!(decode(b"%%%").is_err());
    }

    #[test]
    fn finds_the_last_sequence_among_other_output() {
        let output = b"\x1b[2J\x1b]52;c;QQ==\x07text\x1b]52;c;Qg==\x07";
        assert_eq!(last_payload(output), Some(&b"Qg=="[..]));
    }

    #[test]
    fn ignores_output_without_a_sequence() {
        assert_eq!(last_payload(b"\x1b[2J"), None);
        assert_eq!(last_payload(b"\x1b]52;c;QQ=="), None);
    }

    #[test]
    fn describes_control_characters_by_name() {
        assert_eq!(describe(b"QQ=="), "ESC]52;c;QQ==BEL");
    }
}
