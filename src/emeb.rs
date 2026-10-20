use crate::*;

/// `EventMessageEmptyBox` ('emeb'), a sample carried by an Event Message
/// track (the [`Evte`](crate::Evte) sample entry) to signal that no event is
/// active for the enclosing sample's duration.
///
/// See ISO/IEC 23001-18:2022 Section 6.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Emeb;

impl Atom for Emeb {
    const KIND: FourCC = FourCC::new(b"emeb");

    fn decode_body<B: Buf>(_buf: &mut B) -> Result<Self> {
        Ok(Self)
    }

    fn encode_body<B: BufMut>(&self, _buf: &mut B) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emeb_roundtrip() {
        // A real `emeb` sample: header only, no body.
        const ENCODED_EMEB: &[u8] = &[0x00, 0x00, 0x00, 0x08, 0x65, 0x6d, 0x65, 0x62];

        let emeb =
            Emeb::decode(&mut std::io::Cursor::new(ENCODED_EMEB)).expect("failed to decode emeb");
        assert_eq!(emeb, Emeb);

        let mut buf = Vec::new();
        emeb.encode(&mut buf).unwrap();
        assert_eq!(buf.as_slice(), ENCODED_EMEB);
    }
}
