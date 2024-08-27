mod mvex;
mod mvhd;
mod trak;
mod udta;

pub use mvex::*;
pub use mvhd::*;
pub use trak::*;
pub use udta::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Moov {
    pub mvhd: Mvhd,
    pub meta: Option<Meta>,
    pub mvex: Option<Mvex>,
    pub traks: Vec<Trak>,
    pub udta: Option<Udta>,
}

impl Atom for Moov {
    const KIND: FourCC = FourCC::new(b"moov");

    fn decode_atom(mut buf: &mut Buf) -> Result<Self> {
        let mut mvhd = None;
        let mut meta = None;
        let mut udta = None;
        let mut mvex = None;
        let mut traks = Vec::new();

        while let Some(any) = buf.decode()? {
            match any {
                Any::Mvhd(atom) => mvhd.replace(atom),
                Any::Meta(atom) => meta.replace(atom),
                Any::Mvex(atom) => mvex.replace(atom),
                Any::Trak(atom) => traks.push(atom),
                Any::Udta(atom) => udta.replace(atom),
                _ => return Err(Error::UnexpectedBox(any.kind())),
            }
        }

        Ok(Moov {
            mvhd: mvhd.ok_or(Error::MissingBox("mvhd"))?,
            meta,
            udta,
            mvex,
            traks,
        })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        self.mvhd.encode(buf)?;
        self.meta.encode(buf)?;
        self.mvex.encode(buf)?;
        self.traks.encode(buf)?;
        self.udta.encode(buf)?;

        Ok(())
    }
}
