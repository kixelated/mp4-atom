use crate::*;

// PixelInformationProperty, ISO/IEC 23008-12 Section 6.5.6
// Number and bit depth of the colour components in the reconstructed image
// for the image item this property is associated with.

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pixi {
    pub bits_per_channel: Vec<u8>,
}

impl AtomExt for Pixi {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"pixi");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let num_channels = u8::decode(buf)?;
        let mut bits_per_channel = Vec::with_capacity(num_channels as usize);
        for _ in 0..num_channels {
            bits_per_channel.push(u8::decode(buf)?);
        }
        Ok(Pixi { bits_per_channel })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let num_channels = self.bits_per_channel.len() as u8;
        num_channels.encode(buf)?;
        for bpc in &self.bits_per_channel {
            bpc.encode(buf)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixi() {
        let expected = Pixi {
            bits_per_channel: vec![8, 7, 6],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        assert_eq!(
            buf,
            [0, 0, 0, 16, b'p', b'i', b'x', b'i', 0, 0, 0, 0, 3, 8, 7, 6]
        );
        let decoded = Pixi::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
