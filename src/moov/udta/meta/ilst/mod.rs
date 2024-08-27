mod data;
use std::collections::HashMap;

pub use data::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ilst {
    pub items: HashMap<MetadataKey, IlstItem>,
}

impl Atom for Ilst {
    const KIND: FourCC = FourCC::new(b"ilst");

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        let mut items = HashMap::new();

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Name(atom) => {
                    items.insert(MetadataKey::Title, IlstItem::read_box(reader, s)?);
                }
                Any::DayBox => {
                    items.insert(MetadataKey::Year, IlstItem::read_box(reader, s)?);
                }
                Any::Covr(atom) => {
                    items.insert(MetadataKey::Poster, IlstItem::read_box(reader, s)?);
                }
                Any::Desc(atom) => {
                    items.insert(MetadataKey::Summary, IlstItem::read_box(reader, s)?);
                }
                atom => return Error::UnexpectedBox(atom.kind()),
            }
        }

        Ok(Ilst { items })
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        for (key, value) in &self.items {
            let name = match key {
                MetadataKey::Title => Any::Name,
                MetadataKey::Year => Any::DayBox,
                MetadataKey::Poster => Any::Covr,
                MetadataKey::Summary => Any::Desc,
            };
            BoxHeader::new(name, value.get_size()).write(writer)?;
            value.data.write_box(writer)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IlstItem {
    pub data: Data,
}

impl<R: Read + Seek> Read<&mut R> for IlstItem {
    fn decode_inner(buf: &mut Buf) -> Result<Self> {
        let mut data = buf.decode()?;

        while let Some(atom) = buf.decode()? {
            match atom {
                Any::Data(atom) => data.replace(atom),
                atom => return Error::UnexpectedBox(atom.kind()),
            }
        }

        Ok(IlstItem {
            data: data.ok_or(Error::MissingBox(Data::KIND)),
        })
    }
}

impl<'a> Metadata<'a> for Ilst {
    fn title(&self) -> Option<Cow<str>> {
        self.items.get(&MetadataKey::Title).map(item_to_str)
    }

    fn year(&self) -> Option<u32> {
        self.items.get(&MetadataKey::Year).and_then(item_to_u32)
    }

    fn poster(&self) -> Option<&[u8]> {
        self.items.get(&MetadataKey::Poster).map(item_to_bytes)
    }

    fn summary(&self) -> Option<Cow<str>> {
        self.items.get(&MetadataKey::Summary).map(item_to_str)
    }
}

fn item_to_bytes(item: &IlstItem) -> &[u8] {
    &item.data.data
}

fn item_to_str(item: &IlstItem) -> Cow<str> {
    String::from_utf8_lossy(&item.data.data)
}

fn item_to_u32(item: &IlstItem) -> Option<u32> {
    match item.data.data_type {
        DataType::Binary if item.data.data.len() == 4 => Some(BigEndian::read_u32(&item.data.data)),
        DataType::Text => String::from_utf8_lossy(&item.data.data).parse::<u32>().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ilst() {
        let src_year = IlstItem {
            data: Data {
                data_type: DataType::Text,
                data: b"test_year".to_vec(),
            },
        };
        let expected = Ilst {
            items: [
                (MetadataKey::Title, IlstItem::default()),
                (MetadataKey::Year, src_year),
                (MetadataKey::Poster, IlstItem::default()),
                (MetadataKey::Summary, IlstItem::default()),
            ]
            .into(),
        };
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Ilst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_ilst_empty() {
        let expected = Ilst::default();
        let mut buf = BufMut::new();
        expected.encode(&mut buf).unwrap();

        let mut buf = buf.filled();
        let decoded = Ilst::decode(&mut buf).unwrap();
        assert_eq!(decoded, expected);
    }
}
