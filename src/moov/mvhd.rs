use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MvhdVersion {
    V0,
    V1,
}

impl Decode for MvhdVersion {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {
        let v = buf.decode()?;
        match v {
            0u8 => Ok(MvhdVersion::V0),
            1u8 => Ok(MvhdVersion::V1),
            _ => Err(Error::UnknownVersion(v)),
        }
    }
}

impl Encode for MvhdVersion {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let v = match self {
            MvhdVersion::V0 => 0u8,
            MvhdVersion::V1 => 1u8,
        };
        v.encode(buf)
    }

    fn encode_size(&self) -> usize {
        1
    }
}

impl Encode for MvhdVersion {
    fn encode<B: BufMut>(&self, buf: &mut B) -> Result<()> {
        let version = match self {
            MvhdVersion::V0 => 0,
            MvhdVersion::V1 => 1,
        };
        version.encode(buf)
    }

    fn encode_size(&self) -> usize {
        1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mvhd {
    pub version: MvhdVersion,
    pub flags: [u8; 3],
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,

    pub rate: Ratio<U16>,
    pub volume: Ratio<U8>,

    pub matrix: tkhd::Matrix,
    pub next_track_id: u32,
}

impl Atom for Mvhd {
    const KIND: FourCC = FourCC::unchecked("mvhd");

    fn decode_inner<B: Buf>(buf: &mut B) -> Result<Self> {
        let version = buf.decode()?;
        let flags = buf.decode()?;

        let (creation_time, modification_time, timescale, duration) = match version {
            MvhdVersion::V1 => (
                u64::decode(buf)?,
                u64::decode(buf)?,
                u32::decode(buf)?,
                u64::decode(buf)?,
            ),
            MvhdVersion::V0 => (
                u32::decode(buf)? as u64,
                u32::decode(buf)? as u64,
                u32::decode(buf)?,
                u32::decode(buf)? as u64,
            ),
        };

        let rate = buf.decode()?;
        let volume = buf.decode()?;

        u16::decode(buf)?; // reserved = 0
        u64::decode(buf)?; // reserved = 0

        let matrix = tkhd::Matrix {
            a: reader.read_i32::<BigEndian>()?,
            b: reader.read_i32::<BigEndian>()?,
            u: reader.read_i32::<BigEndian>()?,
            c: reader.read_i32::<BigEndian>()?,
            d: reader.read_i32::<BigEndian>()?,
            v: reader.read_i32::<BigEndian>()?,
            x: reader.read_i32::<BigEndian>()?,
            y: reader.read_i32::<BigEndian>()?,
            w: reader.read_i32::<BigEndian>()?,
        };

        skip_bytes(reader, 24)?; // pre_defined = 0

        let next_track_id = reader.read_u32::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(MvhdBox {
            version,
            flags,
            creation_time,
            modification_time,
            timescale,
            duration,
            rate,
            volume,
            matrix,
            next_track_id,
        })
    }
}

impl Default for Mvhd {
    fn default() -> Self {
        Mvhd {
            version: MvhdVersion::V0,
            flags: 0,
            creation_time: 0,
            modification_time: 0,
            timescale: 1000,
            duration: 0,
            rate: FixedPointU16::new(1),
            matrix: tkhd::Matrix::default(),
            volume: FixedPointU8::new(1),
            next_track_id: 1,
        }
    }
}

impl Decode for Mvhd {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self> {}
}

impl<W: Write> WriteBox<&mut W> for MvhdBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.creation_time)?;
            writer.write_u64::<BigEndian>(self.modification_time)?;
            writer.write_u32::<BigEndian>(self.timescale)?;
            writer.write_u64::<BigEndian>(self.duration)?;
        } else if self.version == 0 {
            writer.write_u32::<BigEndian>(self.creation_time as u32)?;
            writer.write_u32::<BigEndian>(self.modification_time as u32)?;
            writer.write_u32::<BigEndian>(self.timescale)?;
            writer.write_u32::<BigEndian>(self.duration as u32)?;
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
        }
        writer.write_u32::<BigEndian>(self.rate.raw_value())?;

        writer.write_u16::<BigEndian>(self.volume.raw_value())?;

        writer.write_u16::<BigEndian>(0)?; // reserved = 0

        writer.write_u64::<BigEndian>(0)?; // reserved = 0

        writer.write_i32::<BigEndian>(self.matrix.a)?;
        writer.write_i32::<BigEndian>(self.matrix.b)?;
        writer.write_i32::<BigEndian>(self.matrix.u)?;
        writer.write_i32::<BigEndian>(self.matrix.c)?;
        writer.write_i32::<BigEndian>(self.matrix.d)?;
        writer.write_i32::<BigEndian>(self.matrix.v)?;
        writer.write_i32::<BigEndian>(self.matrix.x)?;
        writer.write_i32::<BigEndian>(self.matrix.y)?;
        writer.write_i32::<BigEndian>(self.matrix.w)?;

        write_zeros(writer, 24)?; // pre_defined = 0

        writer.write_u32::<BigEndian>(self.next_track_id)?;

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mp4box::BoxHeader;
    use std::io::Cursor;

    #[test]
    fn test_mvhd32() {
        let src_box = MvhdBox {
            version: 0,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            timescale: 1000,
            duration: 634634,
            rate: FixedPointU16::new(1),
            volume: FixedPointU8::new(1),
            matrix: tkhd::Matrix::default(),
            next_track_id: 1,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::MvhdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = MvhdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_mvhd64() {
        let src_box = MvhdBox {
            version: 1,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            timescale: 1000,
            duration: 634634,
            rate: FixedPointU16::new(1),
            volume: FixedPointU8::new(1),
            matrix: tkhd::Matrix::default(),
            next_track_id: 1,
        };
        let mut buf = Vec::new();
        src_box.write_box(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.box_size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = BoxHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, BoxType::MvhdBox);
        assert_eq!(src_box.box_size(), header.size);

        let dst_box = MvhdBox::read_box(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
