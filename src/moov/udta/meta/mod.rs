mod ilst;
pub use ilst::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Meta {
    Mdir { ilst: Option<Ilst> },
}

impl Default for Meta {
    fn default() -> Self {
        Self::Mdir { ilst: None }
    }
}

const MDIR: FourCC = FourCC::new(b"mdir");

impl AtomExt for Meta {
    type Ext = ();
    const KIND_EXT: FourCC = FourCC::new(b"meta");

    fn decode_atom_ext(buf: &mut Bytes, _ext: ()) -> Result<Self> {
        let hdlr = Hdlr::decode(buf)?;

        match hdlr.handler_type {
            MDIR => {
                let ilst = Option::<Ilst>::decode(buf)?;
                Ok(Meta::Mdir { ilst })
            }
            _ => todo!("unsupported handler type: {:?}", hdlr.handler_type),
        }
    }

    fn encode_atom_ext(&self, buf: &mut BytesMut) -> Result<()> {
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

        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let output = Meta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_meta_mdir() {
        let expected = Meta::Mdir {
            ilst: Some(Ilst::default()),
        };

        let mut buf = BytesMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.freeze();
        let output = Meta::decode(&mut buf).unwrap();
        assert_eq!(output, expected);
    }
}
