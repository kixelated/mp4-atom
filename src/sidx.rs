use crate::*;

// SegmentIndexBox, ISO/IEC 14496-12 Section 8.16.3
// This is called out in CMAF (23000-19) and DASH (23009-1).

ext! {
    name: Sidx,
    versions: [0, 1],
    flags: {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentReference {
    pub reference_type: bool,
    pub reference_size: u32,
    pub subsegment_duration: u32,
    pub starts_with_sap: bool,
    pub sap_type: u8,
    pub sap_delta_time: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Sidx {
    pub reference_id: u32,
    pub timescale: u32,
    pub earliest_presentation_time: u64,
    pub first_offset: u64,
    pub references: Vec<SegmentReference>,
}

impl AtomExt for Sidx {
    type Ext = SidxExt;

    const KIND_EXT: FourCC = FourCC::new(b"sidx");

    fn decode_body_ext<B: Buf>(buf: &mut B, ext: SidxExt) -> Result<Self> {
        let reference_id = u32::decode(buf)?;
        let timescale = u32::decode(buf)?;
        let (earliest_presentation_time, first_offset) = if ext.version == SidxVersion::V0 {
            (u32::decode(buf)?.into(), u32::decode(buf)?.into())
        } else {
            (u64::decode(buf)?, u64::decode(buf)?)
        };
        let _reserved = u16::decode(buf)?;
        let reference_count = u16::decode(buf)?;
        let mut references = Vec::with_capacity(std::cmp::min(reference_count as usize, 128));
        for _ in 0..reference_count {
            let reference_type_and_size = u32::decode(buf)?;
            let reference_type = (reference_type_and_size & 0x8000_0000) == 0x8000_0000;
            let reference_size = reference_type_and_size & 0x7FFF_FFFF;
            let subsegment_duration = u32::decode(buf)?;
            let sap_flag_and_type_and_delta_time = u32::decode(buf)?;
            let starts_with_sap = (sap_flag_and_type_and_delta_time & 0x8000_0000) == 0x8000_0000;
            let sap_type = ((sap_flag_and_type_and_delta_time >> 28) & 0b111) as u8;
            let sap_delta_time = sap_flag_and_type_and_delta_time & 0x0FFF_FFFF;
            let reference = SegmentReference {
                reference_type,
                reference_size,
                subsegment_duration,
                starts_with_sap,
                sap_type,
                sap_delta_time,
            };
            references.push(reference);
        }

        Ok(Sidx {
            reference_id,
            timescale,
            earliest_presentation_time,
            first_offset,
            references,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<SidxExt> {
        self.reference_id.encode(buf)?;
        self.timescale.encode(buf)?;
        let version = match (
            u32::try_from(self.earliest_presentation_time),
            u32::try_from(self.first_offset),
        ) {
            (Ok(earliest_presentation_time), Ok(first_offset)) => {
                earliest_presentation_time.encode(buf)?;
                first_offset.encode(buf)?;
                SidxVersion::V0
            }
            _ => {
                self.earliest_presentation_time.encode(buf)?;
                self.first_offset.encode(buf)?;
                SidxVersion::V1
            }
        };
        0u16.encode(buf)?; // reserved
        let reference_count: u16 = self
            .references
            .len()
            .try_into()
            .map_err(|_| Error::TooLarge(Self::KIND))?;
        reference_count.encode(buf)?;
        for reference in &self.references {
            let reference_type_and_size: u32 = match reference.reference_type {
                true => 0x8000_0000 | reference.reference_size,
                false => reference.reference_size,
            };
            reference_type_and_size.encode(buf)?;
            reference.subsegment_duration.encode(buf)?;
            let sap_flag_and_type_and_delta_time = match reference.starts_with_sap {
                true => {
                    0x8000_0000
                        | ((reference.sap_type as u32 & 0b111) << 28)
                        | (reference.sap_delta_time & 0x0FFF_FFFF)
                }
                false => {
                    ((reference.sap_type as u32 & 0b111) << 28)
                        | (reference.sap_delta_time & 0x0FFF_FFFF)
                }
            };
            sap_flag_and_type_and_delta_time.encode(buf)?;
        }
        Ok(SidxExt { version })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // From MPEG File Format Conformance suite: 21_segment.mp4
    const ENCODED_SIDX: &[u8] = &[
        0x00, 0x00, 0x00, 0x2c, 0x73, 0x69, 0x64, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x04, 0xfc, 0x80, 0x00, 0x00, 0x13, 0x80, 0x90, 0x00, 0x00, 0x00,
    ];

    // Decoded values per 21_segment_gpac.json
    /*
    "SegmentIndexBox": {
      "@Size": "44",
      "@Type": "sidx",
      "@Version": "0",
      "@Flags": "0",
      "@Specification": "p12",
      "@Container": "file",
      "@reference_ID": "1",
      "@timescale": "100",
      "@earliest_presentation_time": "0",
      "@first_offset": "0",
      "Reference": {
        "@type": "0",
        "@size": "326784",
        "@duration": "4992",
        "@startsWithSAP": "1",
        "@SAP_type": "1",
        "@SAPDeltaTime": "0"
      }
      */

    #[test]
    fn test_sidx_v0_decode() {
        let buf: &mut std::io::Cursor<&&[u8]> = &mut std::io::Cursor::new(&ENCODED_SIDX);
        let sidx = Sidx::decode(buf).expect("failed to decode sidx");
        assert_eq!(
            sidx,
            Sidx {
                reference_id: 1,
                timescale: 100,
                earliest_presentation_time: 0,
                first_offset: 0,
                references: vec![SegmentReference {
                    reference_type: false,
                    reference_size: 326784,
                    subsegment_duration: 4992,
                    starts_with_sap: true,
                    sap_type: 1,
                    sap_delta_time: 0,
                }],
            }
        );
    }

    #[test]
    fn test_sidx_v0_encode() {
        let mut buf = Vec::new();
        let sidx = Sidx {
            reference_id: 1,
            timescale: 100,
            earliest_presentation_time: 0,
            first_offset: 0,
            references: vec![SegmentReference {
                reference_type: false,
                reference_size: 326784,
                subsegment_duration: 4992,
                starts_with_sap: true,
                sap_type: 1,
                sap_delta_time: 0,
            }],
        };
        sidx.encode(&mut buf).unwrap();

        assert_eq!(buf.as_slice(), ENCODED_SIDX);
    }
}
