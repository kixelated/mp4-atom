mod elst;
pub use elst::*;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Edts {
    pub elst: Option<Elst>,
}

impl Atom for Edts {
    const KIND: FourCC = FourCC::new(b"edts");

    nested! {
        required: [],
        optional: [ Elst ],
        multiple: [],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A container decoded through the `nested!` macro must tolerate a sub-header
    // padding remainder (e.g. a QuickTime zero terminator) after its children.
    #[test]
    fn test_edts_trailing_padding() {
        let mut buf = Vec::new();
        Edts::default().encode(&mut buf).unwrap();
        buf.extend_from_slice(&[0, 0, 0, 0]);
        let size = (buf.len() as u32).to_be_bytes();
        buf[0..4].copy_from_slice(&size);

        let edts = Edts::decode(&mut buf.as_slice()).expect("trailing padding must be tolerated");
        assert_eq!(edts, Edts::default());
    }
}
