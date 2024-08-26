use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Moof {
    //pub mfhd: Mfhd,
    //pub traf: Vec<Traf>,
    pub unknown: Vec<Unknown>,
}

impl Atom for Moof {
    const KIND: FourCC = FourCC::new(b"moof");

    fn decode_inner<B: Buf>(buf: &mut B) -> Result<Self> {
        /*
        let mut mfhd = None;
        let mut traf = Vec::new();
        */
        let mut unknown = Vec::new();

        while buf.has_remaining() {
            match Any::decode(buf)? {
                /*
                Any::Mfhd(atom) => {
                    mfhd = Some(atom);
                }
                Any::Traf(atom) => {
                    traf.push(atom);
                }
                */
                Any::Unknown(atom) => unknown.push(atom),
                other => return Err(Error::UnexpectedBox(other.kind())),
            }
        }

        Ok(Moof {
            //mfhd: mfhd.ok_or(Error::MissingBox("mfhd".into()))?,
            //traf,
            unknown,
        })
    }

    fn encode_inner<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        /*
        self.mfhd.encode(buf)?;
        for traf in &self.traf {
            traf.encode(buf)?;
        }
        */
        for unknown in &self.unknown {
            unknown.encode(buf)?;
        }
        Ok(())
    }

    fn encode_inner_size(&self) -> usize {
        /*self.mfhd.encode_size() + self.traf.encode_size() + */
        self.unknown.encode_size()
    }
}
