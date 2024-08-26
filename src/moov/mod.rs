use crate::*;

//mod mvex;
//mod mvhd;
//mod trak;
//mod udta;

//pub use mvex::*;
//pub use mvhd::*;
//pub use trak::*;
//pub use udta::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Moov {
    //pub mvhd: Mvhd,
    /*
    pub meta: Option<Meta>,
    pub mvex: Option<Mvex>,
    pub traks: Vec<Trak>,
    pub udta: Option<Udta>,
    */
    pub unknown: Vec<Unknown>,
}

impl Atom for Moov {
    const KIND: FourCC = FourCC::new(b"moov");

    fn encode_inner_size(&self) -> usize {
        //let mut size = self.mvhd.encode_size();
        /*
        for trak in self.traks.iter() {
            size += trak.encode_size();
        }
        if let Some(meta) = &self.meta {
            size += meta.encode_size();
        }
        if let Some(udta) = &self.udta {
            size += udta.encode_size();
        }
        */
        let size = self.unknown.encode_size();
        size
    }

    fn decode_inner<B: Buf>(mut buf: &mut B) -> Result<Self> {
        /*
        let mut mvhd = None;
        let mut meta = None;
        let mut udta = None;
        let mut mvex = None;
        let mut trak = Vec::new();
        */
        let mut unknown = Vec::new();

        while let Some(any) = buf.decode()? {
            match any {
                /*
                Any::Mvhd(atom) => {
                    mvhd.replace(atom);
                }
                Any::Meta(atom) => {
                    meta.replace(atom);
                }
                Any::Mvex(atom) => {
                    mvex.replace(atom);
                }
                Any::Trak(atom) => {
                    traks.push(atom);
                }
                Any::Udta(atom) => {
                    udta.replace(atom);
                }
                */
                Any::Unknown(atom) => {
                    unknown.push(atom);
                }
                _ => {
                    return Err(Error::UnexpectedBox(any.kind()));
                }
            }
        }

        Ok(Moov {
            //mvhd: mvhd.ok_or(Error::MissingBox("mvhd"))?,
            /*
            meta,
            udta,
            mvex,
            trak,
            */
            unknown,
        })
    }

    fn encode_inner<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        //self.mvhd.encode(buf)?;

        /*
        if let Some(meta) = &self.meta {
            meta.encode(buf)?;
        }
        if let Some(mvex) = &self.mvex {
            mvex.encode(buf)?;
        }
        for trak in self.traks.iter() {
            trak.encode(buf)?;
        }
        if let Some(udta) = &self.udta {
            udta.encode(buf)?;
        }
        */

        self.unknown.encode(buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /*
    #[test]
    fn test_moov() {
        let src_box = Moov {
            mvhd: Mvhd::default(),
            mvex: None, // XXX mvex is not written currently
            traks: vec![],
            meta: Some(Meta::default()),
            udta: Some(Udta::default()),
        };

        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::MoovBox);
        assert_eq!(header.size, src_box.box_size());

        let dst_box = MoovBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(dst_box, src_box);
    }

    #[test]
    fn test_moov_empty() {
        let src_box = MoovBox::default();

        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::MoovBox);
        assert_eq!(header.size, src_box.box_size());

        let dst_box = MoovBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(dst_box, src_box);
    }
    */
}
