mod ilst;
mod iref;
mod pitm;
pub use ilst::*;
pub use iref::*;
pub use pitm::*;

use crate::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Meta {
    pub hdlr: Hdlr,
    pub pitm: Option<Pitm>,
    pub iref: Option<Iref>,
    pub ilst: Option<Ilst>,
    pub unknown: Vec<crate::Any>,
}

impl Eq for Meta {}

impl AtomExt for Meta {
    type Ext = ();

    const KIND_EXT: FourCC = FourCC::new(b"meta");

    fn decode_body_ext<B: Buf>(buf: &mut B, _ext: ()) -> Result<Self> {
        let hdlr = Hdlr::decode(buf)?;
        let mut pitm = None;
        let mut iref = None;
        let mut ilst = None;
        let mut unknown_boxes = vec![];
        while let Some(atom) = Any::decode_maybe(buf)? {
            match atom {
                Any::Pitm(atom) => pitm = Some(atom),
                Any::Iref(atom) => iref = Some(atom),
                Any::Ilst(atom) => ilst = Some(atom),
                _ => {
                    unknown_boxes.push(atom);
                }
            }
        }

        Ok(Self {
            hdlr,
            pitm,
            iref,
            ilst,
            unknown: unknown_boxes,
        })
    }

    fn encode_body_ext<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        self.hdlr.encode(buf)?;
        if self.pitm.is_some() {
            self.pitm.encode(buf)?;
        }
        if self.ilst.is_some() {
            self.ilst.encode(buf)?;
        }
        for atom in &self.unknown {
            atom.encode(buf)?;
        }
        if self.iref.is_some() {
            self.iref.encode(buf)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_empty() {
        let expected = Meta {
            hdlr: Hdlr {
                handler: b"fake".into(),
                name: "".into(),
            },
            pitm: None,
            iref: None,
            ilst: None,
            unknown: vec![],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Meta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_meta_mdir() {
        let expected = Meta {
            hdlr: Hdlr {
                handler: b"mdir".into(),
                name: "".into(),
            },
            pitm: Some(Pitm { item_id: 3 }),
            iref: Some(Iref {
                references: vec![Reference {
                    reference_type: b"cdsc".into(),
                    from_item_id: 2,
                    to_item_ids: vec![1, 3],
                }],
            }),
            ilst: Some(Ilst::default()),
            unknown: vec![],
        };
        let mut buf = Vec::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.as_ref();
        let output = Meta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }
}
