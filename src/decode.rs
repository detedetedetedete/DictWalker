use encoding::all::WINDOWS_1257;
use encoding::all::UTF_16BE;
use encoding::all::UTF_16LE;
use encoding::DecoderTrap;
use encoding::types::Encoding;

pub fn decode_windows_1257(bytes: &[u8]) -> Result<String, String> {
    match WINDOWS_1257.decode(bytes, DecoderTrap::Strict) {
        Ok(v) => Ok(v),
        Err(e) => return Err(format!("Failed to read bytes as Windows 1257: \"{}\"", e))
    }
}

pub fn decode_utf16_le(bytes: &[u8]) -> Result<String, String> {
    match UTF_16LE.decode(bytes, DecoderTrap::Strict) {
        Ok(v) => Ok(v),
        Err(e) => return Err(format!("Failed to read bytes as UTF_16LE: \"{}\"", e))
    }
}

pub fn decode_utf16_be(bytes: &[u8]) -> Result<String, String> {
    match UTF_16BE.decode(bytes, DecoderTrap::Strict) {
        Ok(v) => Ok(v),
        Err(e) => return Err(format!("Failed to read bytes as UTF_16BE: \"{}\"", e))
    }
}