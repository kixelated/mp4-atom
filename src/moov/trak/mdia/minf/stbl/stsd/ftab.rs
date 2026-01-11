use crate::*;

/// Font table for tx3g
///
/// 3GPP TS 26.245 or ETSI TS 126 245 Section 5.16
/// See https://www.etsi.org/deliver/etsi_ts/126200_126299/126245/18.00.00_60/ts_126245v180000p.pdf

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FontEntry {
    pub font_id: u16,
    pub font: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ftab {
    pub font_entries: Vec<FontEntry>,
}

impl Atom for Ftab {
    const KIND: FourCC = FourCC::new(b"ftab");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let entry_count = u16::decode(buf)?;
        let mut font_entries = Vec::with_capacity(std::cmp::min(8, entry_count as usize));
        for _ in 0..entry_count {
            let font_id = u16::decode(buf)?;
            let font_name_length = u8::decode(buf)?;
            let font_bytes = Vec::decode_exact(buf, font_name_length as usize)?;
            let font = if font_bytes.len() >= 4 && font_bytes[0] == 0xFF && font_bytes[1] == 0xFE {
                // UTF-16 little endian
                if font_name_length % 2 != 0 {
                    return Err(Error::InvalidSize);
                }
                let utf_16_len: usize = ((font_name_length - 2) / 2) as usize;
                let mut utf_16 = Vec::with_capacity(utf_16_len);
                for i in 1..=utf_16_len {
                    let bytes = [font_bytes[i * 2], font_bytes[i * 2 + 1]];
                    utf_16.push(u16::from_le_bytes(bytes));
                }
                String::from_utf16(&utf_16).map_err(|_| {
                    Error::InvalidString("Failed to parse UTF-16 LE font id in ftab".into())
                })?
            } else if font_bytes.len() >= 4 && font_bytes[0] == 0xFE && font_bytes[1] == 0xFF {
                // UTF-16 big endian
                if font_name_length % 2 != 0 {
                    return Err(Error::InvalidSize);
                }
                let utf_16_len: usize = ((font_name_length - 2) / 2) as usize;
                let mut utf_16 = Vec::with_capacity(utf_16_len);
                for i in 1..=utf_16_len {
                    let bytes = [font_bytes[i * 2], font_bytes[i * 2 + 1]];
                    utf_16.push(u16::from_be_bytes(bytes));
                }
                String::from_utf16(&utf_16).map_err(|_| {
                    Error::InvalidString("Failed to parse UTF-16 BE font id in ftab".into())
                })?
            } else {
                String::from_utf8(font_bytes)
                    .map_err(|_| Error::InvalidString("Failed to parse font id in ftab".into()))?
            };
            let font_entry = FontEntry { font_id, font };
            font_entries.push(font_entry);
        }
        Ok(Ftab { font_entries })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let entry_count: u16 = self
            .font_entries
            .len()
            .try_into()
            .map_err(|_| Error::TooLarge(Self::KIND))?;
        entry_count.encode(buf)?;
        for font_entry in &self.font_entries {
            font_entry.font_id.encode(buf)?;
            // We always encode as UTF-8, so there won't be round tripping of UTF-16. That is OK.
            let font_bytes = font_entry.font.as_bytes();
            let font_name_length: u8 = font_bytes
                .len()
                .try_into()
                .map_err(|_| Error::TooLarge(Self::KIND))?;
            font_name_length.encode(buf)?;
            font_bytes.encode(buf)?;
        }

        Ok(())
    }
}
