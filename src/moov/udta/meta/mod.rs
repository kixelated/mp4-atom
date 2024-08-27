mod ilst;
pub use ilst::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Meta {
    Mdir { ilst: Option<Ilst> },
}

const MDIR: FourCC = FourCC::new(b"mdir");

impl AtomExt for Meta {
    type Ext = ();
    const KIND: FourCC = FourCC::new(b"meta");

    fn decode_atom(buf: &mut Buf, _ext: ()) -> Result<Self> {
        let hdlr = Hdlr::decode(buf)?;

        let mut ilst = None;

        match hdlr.handler_type {
            MDIR => {
                let ilst = buf.decode()?;
                Ok(Meta::Mdir { ilst })
            }
        }
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        let hldr = match self {
            Self::Mdir { .. } => Hdlr {
                handler_type: MDIR,
                ..Default::default()
            },
        };

        hldr.encode(buf)?;

        match self {
            Self::Mdir { ilst } => {
                ilst.encode(buf)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_mdir_empty() {
        let expected = Meta::Mdir { ilst: None };

        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let output = Meta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_meta_mdir() {
        let expected = Meta::Mdir {
            ilst: Some(Ilst::default()),
        };

        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let output = Meta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_meta_hdrl_non_first() {
        const ENCODED: &[u8] = b"\x00\x00\x00\x7fmeta\x00\x00\x00\x00\x00\x00\x00Qilst\x00\x00\x00I\xa9too\x00\x00\x00Adata\x00\x00\x00\x01\x00\x00\x00\x00TMPGEnc Video Mastering Works 7 Version 7.0.15.17\x00\x00\x00\"hdlr\x00\x00\x00\x00\x00\x00\x00\x00mdirappl\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";

        let mut buf = Buf::new(ENCODED);
        let meta_box = Meta::decode(&mut buf).unwrap();

        // this contains \xa9too box in the ilst
        // it designates the tool that created the file, but is not yet supported by this crate
        assert_eq!(
            meta_box,
            Meta::Mdir {
                ilst: Some(Ilst::default())
            }
        );
    }

    #[test]
    fn test_meta_unknown() {
        let src_hdlr = Hdlr {
            handler_type: FourCC::from(*b"test"),
            ..Default::default()
        };
        let src_data = (Any::UnknownBox(0x42494241), b"123".to_vec());
        let expected = Meta::Unknown {
            hdlr: src_hdlr,
            data: vec![src_data],
        };

        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let output = Meta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }
}
