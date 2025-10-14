use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hdlr {
    pub handler: FourCC,
    pub name: String,
}

impl Default for Hdlr {
    fn default() -> Self {
        Hdlr {
            handler: FourCC::new(b"none"),
            name: String::new(),
        }
    }
}

impl AtomExt for Hdlr {
    type Ext = ();
    const KIND_EXT: FourCC = FourCC::new(b"hdlr");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        u32::decode(buf)?; // pre-defined
        let handler = FourCC::decode(buf)?;

        <[u8; 12]>::decode(buf)?; // reserved

        let name = String::decode(buf)?;

        // Skip any trailing padding
        if buf.has_remaining() {
            tracing::warn!("Skipped {} extra trailing bytes in hdlr", buf.remaining());
            buf.advance(buf.remaining());
        }

        Ok(Hdlr { handler, name })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        0u32.encode(buf)?; // pre-defined
        self.handler.encode(buf)?;

        // 12 bytes reserved
        [0u8; 12].encode(buf)?;

        self.name.as_str().encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdlr() {
        let expected = Hdlr {
            handler: FourCC::new(b"vide"),
            name: String::from("VideoHandler"),
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Hdlr::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_hdlr_empty() {
        let expected = Hdlr {
            handler: FourCC::new(b"vide"),
            name: String::new(),
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let decoded = Hdlr::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_hdlr_with_trailing_bytes() {
        // Test that we can decode hdlr boxes with extra padding bytes after the name
        // Some encoders add extra null bytes or padding at the end of the box

        let mut buf = Vec::new();

        // hdlr box header
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // size (will be fixed later)
        buf.extend_from_slice(b"hdlr");

        // version and flags
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // pre-defined
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // handler type
        buf.extend_from_slice(b"vide");

        // reserved (12 bytes)
        buf.extend_from_slice(&[0x00; 12]);

        // name (null-terminated string)
        buf.extend_from_slice(b"VideoHandler\0");

        // Add extra trailing bytes that should be skipped
        buf.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // 4 extra bytes

        // Fix the size
        let size = buf.len() as u32;
        buf[0..4].copy_from_slice(&size.to_be_bytes());

        // Decode
        let mut cursor = std::io::Cursor::new(&buf);
        let decoded = Hdlr::decode(&mut cursor).expect("failed to decode hdlr with trailing bytes");

        // Verify the decoded data (trailing bytes should be ignored)
        assert_eq!(decoded.handler, FourCC::new(b"vide"));
        assert_eq!(decoded.name, "VideoHandler");

        // Verify we've consumed all the data
        assert_eq!(cursor.position(), buf.len() as u64);
    }

    #[test]
    fn test_hdlr_with_multiple_trailing_nulls() {
        // Test with multiple null bytes at the end (common in some encoders)

        let mut buf = Vec::new();

        // hdlr box header
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // size
        buf.extend_from_slice(b"hdlr");

        // version and flags
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // pre-defined
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // handler type
        buf.extend_from_slice(b"soun");

        // reserved (12 bytes)
        buf.extend_from_slice(&[0x00; 12]);

        // name
        buf.extend_from_slice(b"SoundHandler\0");

        // Add multiple null bytes at the end
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // Fix the size
        let size = buf.len() as u32;
        buf[0..4].copy_from_slice(&size.to_be_bytes());

        // Decode
        let mut cursor = std::io::Cursor::new(&buf);
        let decoded = Hdlr::decode(&mut cursor).expect("failed to decode hdlr with trailing nulls");

        // Verify
        assert_eq!(decoded.handler, FourCC::new(b"soun"));
        assert_eq!(decoded.name, "SoundHandler");
        assert_eq!(cursor.position(), buf.len() as u64);
    }

    #[test]
    fn test_hdlr_roundtrip_with_trailing_bytes() {
        // Test that our encoder doesn't add trailing bytes,
        // but our decoder can handle them

        let original = Hdlr {
            handler: FourCC::new(b"meta"),
            name: "MetaHandler".to_string(),
        };

        // Encode (should not have trailing bytes)
        let mut encoded = Vec::new();
        original.encode(&mut encoded).unwrap();

        // Decode the clean version
        let mut cursor = std::io::Cursor::new(&encoded);
        let decoded = Hdlr::decode(&mut cursor).expect("failed to decode clean hdlr");
        assert_eq!(decoded, original);

        // Now manually add trailing bytes to the encoded data
        let box_end = cursor.position() as usize;
        let mut encoded_with_trash = encoded[..box_end].to_vec();

        // Add some trash bytes
        encoded_with_trash.extend_from_slice(&[0xFF, 0xEE, 0xDD]);

        // Update the size in the header
        let new_size = encoded_with_trash.len() as u32;
        encoded_with_trash[0..4].copy_from_slice(&new_size.to_be_bytes());

        // Decode again - should still work and ignore trash
        let mut cursor2 = std::io::Cursor::new(&encoded_with_trash);
        let decoded2 = Hdlr::decode(&mut cursor2).expect("failed to decode hdlr with added trash");
        assert_eq!(decoded2, original);
        assert_eq!(cursor2.position(), encoded_with_trash.len() as u64);
    }
}
