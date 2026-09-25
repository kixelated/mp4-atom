use crate::*;

/// Photo - JPEG ('jpeg'), QuickTime's Motion JPEG visual sample entry.
///
/// Unlike most video codecs, JPEG has no separate decoder configuration box:
/// the quantization/Huffman tables and everything else needed to decode are
/// self-contained within each JPEG-coded sample (baseline JPEG per
/// [ITU-T T.81](https://www.itu.int/rec/T-REC-T.81-199209-I/en) | ISO/IEC 10918-1, the
/// core JPEG bitstream standard), so this is just a [`Visual`] sample entry
/// plus the usual optional extension boxes.
///
/// `'jpeg'` itself is a QuickTime-registered codec, not part of the ISO
/// base media file format -- see its
/// [MP4RA registration](https://mp4ra.org/registered-types/codecs) (spec:
/// "QT") and Apple's QuickTime documentation on the
/// [Photo Compressor](https://developer.apple.com/Mac/library/documentation/QuickTime/RM/CompressDecompress/ImageComprMgr/E-Chapter/5CompressorsSupplied.html),
/// which each sample's JPEG bitstream conforms to.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Jpeg {
    pub visual: Visual,
    pub btrt: Option<Btrt>,
    pub colr: Option<Colr>,
    pub pasp: Option<Pasp>,
    pub fiel: Option<Fiel>,
}

impl Atom for Jpeg {
    const KIND: FourCC = FourCC::new(b"jpeg");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        let visual = Visual::decode(buf)?;

        let mut btrt = None;
        let mut colr = None;
        let mut pasp = None;
        let mut fiel = None;
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Btrt(atom) => btrt = atom.into(),
                Any::Colr(atom) => colr = atom.into(),
                Any::Pasp(atom) => pasp = atom.into(),
                Any::Fiel(atom) => fiel = atom.into(),
                unknown => Self::decode_unknown(&unknown)?,
            }
        }
        skip_trailing_padding(buf);

        Ok(Self {
            visual,
            btrt,
            colr,
            pasp,
            fiel,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.visual.encode(buf)?;
        self.btrt.encode(buf)?;
        self.colr.encode(buf)?;
        self.pasp.encode(buf)?;
        self.fiel.encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jpeg_roundtrip() {
        let expected = Jpeg {
            visual: Visual {
                data_reference_index: 1,
                width: 1920,
                height: 1080,
                compressor: "Photo - JPEG".into(),
                ..Default::default()
            },
            btrt: Some(Btrt {
                buffer_size_db: 0,
                max_bitrate: 5_000_000,
                avg_bitrate: 2_000_000,
            }),
            colr: None,
            pasp: Some(Pasp {
                h_spacing: 1,
                v_spacing: 1,
            }),
            fiel: None,
        };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let decoded = Jpeg::decode(&mut buf.as_slice()).expect("failed to decode jpeg");
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_jpeg_minimal() {
        // No optional boxes at all -- just the bare VisualSampleEntry.
        let expected = Jpeg {
            visual: Visual {
                data_reference_index: 1,
                width: 640,
                height: 480,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let decoded = Jpeg::decode(&mut buf.as_slice()).expect("failed to decode jpeg");
        assert_eq!(decoded, expected);
    }
}
