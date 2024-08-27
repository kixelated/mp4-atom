use crate::*;

// Combine the version and flags into a single struct
// We use a special trait to ensure it's always a u32
pub trait Ext: Default {
    fn encode(&self) -> Result<[u8; 4]>;
    fn decode(v: [u8; 4]) -> Result<Self>;
}

// Rather than encoding/decoding the header in every atom, use this trait.
pub trait AtomExt: Sized {
    const KIND: FourCC;

    // One day default associated types will be a thing, then this can be ()
    type Ext: Ext;

    fn encode_atom(&self, buf: &mut BufMut) -> Result<Self::Ext>;
    fn decode_atom(buf: &mut Buf, ext: Self::Ext) -> Result<Self>;
}

impl<T: AtomExt> Atom for T {
    const KIND: FourCC = AtomExt::KIND;

    fn decode_atom(buf: &mut Buf) -> Result<Self> {
        let ext = AtomExt::Ext::decode(buf.u32()?)?;
        AtomExt::decode_atom(buf, ext)
    }

    fn encode_atom(&self, buf: &mut BufMut) -> Result<()> {
        // Here's the magic, we reserve space for the version/flags first
        let start = buf.len();
        buf.u32(0)?;

        // That way we can return them as part of the trait, avoiding boilerplate
        let ext = AtomExt::encode_atom(self, buf)?;

        // Go back and update the version/flags
        buf.u32_at(ext.encode(), start)
    }
}

// Some atoms don't have any version/flags, so we provide a default implementation
impl Ext for () {
    fn encode(&self) -> Result<[u8; 4]> {
        Ok([0, 0, 0, 0])
    }

    fn decode(_: [u8; 4]) -> Result<Self> {
        Ok(Self)
    }
}

// Here's a macro to make life easier:
/* input:
ext! {
    name: Tfdt,
    versions: [0, 1],
    flags: {
        base_data_offset = 0,
        sample_description_index = 1,
        default_sample_duration = 3,
        default_sample_size = 4,
        default_sample_flags = 5,
        duration_is_empty = 16,
        default_base_is_moof = 17,
    },
}

output:
enum TfdtVersion {
    V0 = 0,
    V1 = 1,
}

struct TfdtExt {
    pub version: TfdtVersion,
    pub base_data_offset: bool,
    pub sample_description_index: bool,
    pub default_sample_duration: bool,
    pub default_sample_size: bool,
    pub default_sample_flags: bool,
    pub duration_is_empty: bool,
    pub default_base_is_moof: bool,
}
*/

macro_rules! ext {
	(name: $name:ident, versions: [$($version:expr),*], flags: { $($flag:ident = $bit:expr,)* }) => {
		paste::paste! {
			#[derive(Debug, Clone, PartialEq, Eq)]
			enum [<$name Version>] {
				$(
					[<V $version>] = $version,
				)*
			}

			impl From<[<$name Version>]> for u8 {
				fn from(v: [<$name Version>]) -> u8 {
					v as u8
				}
			}

			impl TryFrom<u8> for [<$name Version>] {
				type Error = Error;

				fn try_from(v: u8) -> Result<Self> {
					match v {
						$(
							$version => Ok(Self::[<V $version>]),
						)*
						_ => Err(Error::UnknownVersion(v)),
					}
				}
			}

			impl Default for [<$name Version>] {
				fn default() -> Self {
					Self::[<V $($version)*>]
				}
			}

			#[derive(Debug, Clone, PartialEq, Eq, Default)]
			struct [<$name Ext>] {
				pub version: [<$name Version>],
				$(
					pub $flag: bool,
				)*
			}

			impl Ext for [<$name Ext>] {
				fn encode(&self) -> Result<[u8; 4]>{
					let mut v = [0u8; 4];
					v[0] = self.version.into();

					$(
						if self.$flag {
							v[$bit / 8] |= 1 << ($bit % 8);
						}
					)*

					Ok(v)
				}

				fn decode(v: [u8; 4]) -> Result<Self> {
					let version = v[0].try_into()?;
					Ok([<$name Ext>] {
						version,
						$(
							$flag: v[$bit / 8] & (1 << ($bit % 8)) != 0,
						)*
					})
				}
			}

			// Helper when there are no flags
			impl From<[<$name Version>]> for [<$name Ext>] {
				fn from(version: [<$name Version>]) -> Self {
					Self {
						version,
						..Default::default()
					}
				}
			}
		}
	};
}

pub(crate) use ext;
