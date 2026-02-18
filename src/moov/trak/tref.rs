use crate::*;

/// Track reference type box.
///
/// We treat this as Atom-like, but in a distinct namespace, consistent
/// with the way MP4RA handles it. Avoids collisions between real boxes,
/// and reference item labels
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrackReferenceTypeBox {
    pub reference_type: FourCC,
    pub track_ids: Vec<u32>,
}

impl TrackReferenceTypeBox {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        if self.track_ids.len() * 4 > (u32::MAX as usize) - 8 {
            // its ludicrous
            return Err(Error::InvalidSize);
        }
        let size: u32 = 4u32 + 4u32 + (self.track_ids.len() as u32) * 4;
        size.encode(buf)?;
        self.reference_type.encode(buf)?;
        self.track_ids.encode(buf)
    }

    fn decode<B: Buf>(buf: &mut B) -> Result<TrackReferenceTypeBox> {
        let header = Header::decode(buf)?;
        let size = header.size.unwrap_or(buf.remaining());
        if size > buf.remaining() {
            return Err(Error::InvalidSize);
        }
        let num_entries = size / 4; // since its all u32
        let mut track_ids = Vec::with_capacity(num_entries.min(16));
        for _ in 0..num_entries {
            track_ids.push(u32::decode(buf)?);
        }
        // ignore any residual bytes unless we're in strict mode
        if cfg!(feature = "strict") && (size % 4 != 0) {
            return Err(Error::InvalidSize);
        }
        buf.advance(size % 4);
        Ok(TrackReferenceTypeBox {
            reference_type: header.kind,
            track_ids,
        })
    }
}

/// Track reference box
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tref {
    pub track_reference_type_boxes: Vec<TrackReferenceTypeBox>,
}

impl Atom for Tref {
    const KIND: FourCC = FourCC::new(b"tref");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let mut track_reference_type_boxes = vec![];
        while buf.has_remaining() {
            let reference = TrackReferenceTypeBox::decode(buf)?;
            track_reference_type_boxes.push(reference);
        }
        Ok(Self {
            track_reference_type_boxes,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        for b in &self.track_reference_type_boxes {
            b.encode(buf)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // From the 01_simple.mp4 sample in MPEG's file format conformance set
    const ENCODED_TREF: &[u8] = &[
        0x00, 0x00, 0x00, 0x24, 0x74, 0x72, 0x65, 0x66, 0x00, 0x00, 0x00, 0x0c, 0x73, 0x79, 0x6e,
        0x63, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x10, 0x6d, 0x70, 0x6f, 0x64, 0x00, 0x00,
        0x00, 0xc9, 0x00, 0x00, 0x00, 0x65,
    ];

    #[test]
    fn test_tref_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_TREF);

        let tref = Tref::decode(buf).expect("failed to decode tref");

        assert_eq!(
            tref,
            Tref {
                track_reference_type_boxes: vec![
                    TrackReferenceTypeBox {
                        reference_type: b"sync".into(),
                        track_ids: vec![1]
                    },
                    TrackReferenceTypeBox {
                        reference_type: b"mpod".into(),
                        track_ids: vec![201, 101]
                    }
                ]
            }
        );
    }

    #[test]
    fn test_tref_encode() {
        let tref = Tref {
            track_reference_type_boxes: vec![
                TrackReferenceTypeBox {
                    reference_type: b"sync".into(),
                    track_ids: vec![1],
                },
                TrackReferenceTypeBox {
                    reference_type: b"mpod".into(),
                    track_ids: vec![201, 101],
                },
            ],
        };

        let mut buf = Vec::new();
        tref.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_TREF);
    }
}
