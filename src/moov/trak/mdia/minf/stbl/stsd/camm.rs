use crate::*;

/// CameraMotionMetadataSampleEntry ('camm'), used for camera motion metadata tracks
/// (e.g. gyroscope/accelerometer samples).
///
/// See [Google's "Camera Motion Metadata Track Design" specification](https://developers.google.com/streetview/publish/camm-spec).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Camm {
    pub metadata: MetaData,
}

impl Atom for Camm {
    const KIND: FourCC = FourCC::new(b"camm");

    fn decode_body<B: Buf>(buf: &mut B) -> Result<Self> {
        Ok(Self {
            metadata: MetaData::decode(buf)?,
        })
    }

    fn encode_body<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.metadata.encode(buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camm_roundtrip() {
        let camm = Camm {
            metadata: MetaData {
                data_reference_index: 1,
            },
        };

        let mut buf = Vec::new();
        camm.encode(&mut buf).unwrap();

        let decoded = Camm::decode(&mut buf.as_slice()).expect("failed to decode camm");
        assert_eq!(decoded, camm);
    }
}
