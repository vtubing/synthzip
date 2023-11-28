use crate::prelude::*;
use std::io::SeekFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DataDescriptor {
  pub signature: Option<[u8; 4]>,
  pub crc32_of_uncompressed_data: u32,
  pub compressed_size: u32,
  pub uncompressed_size: u32,
}

impl ExpectedSize for DataDescriptor {
  fn expected_size(&self) -> u32 {
    match self.signature {
      Some(_) => 16,
      None => 12,
    }
  }
}

impl DataDescriptor {
  const SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x07, 0x08];

  pub fn read_from_end_with_signature<R: Read + Seek>(reader: &mut R) -> Result<Self> {
    let initial_stream_position = reader.stream_position()?;
    reader.seek(SeekFrom::End(-16))?;
    let value = Self::read(reader)?;
    reader.seek(SeekFrom::Start(initial_stream_position))?;
    Ok(value)
  }

  pub fn read_from_end_without_signature<R: Read + Seek>(reader: &mut R) -> Result<Self> {
    let initial_stream_position = reader.stream_position()?;
    reader.seek(SeekFrom::End(-12))?;
    let value = Self::read(reader)?;
    reader.seek(SeekFrom::Start(initial_stream_position))?;
    Ok(value)
  }

  pub fn read_from_end<R: Read + Seek>(reader: &mut R) -> Result<Self> {
    let value = Self::read_from_end_with_signature(reader)?;
    if value.signature.is_some() {
      Ok(value)
    } else {
      Self::read_from_end_without_signature(reader)
    }
  }

  pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
    reader.trace(|reader| {
      let signature = {
        let mut signature = [0u8; 4];

        reader.read_exact(&mut signature)?;

        if signature == Self::SIGNATURE {
          Some(signature)
        } else {
          None
        }
      };

      let crc32_of_uncompressed_data = reader.read_u32::<LittleEndian>()?;
      let compressed_size = reader.read_u32::<LittleEndian>()?;
      let uncompressed_size = reader.read_u32::<LittleEndian>()?;

      let value = Self {
        signature,
        crc32_of_uncompressed_data,
        compressed_size,
        uncompressed_size,
      };

      Ok(value)
    })
  }

  pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
    writer.trace(self.expected_size(), |writer| {
      let Self {
        signature,
        crc32_of_uncompressed_data: crc32,
        compressed_size,
        uncompressed_size,
      } = self;

      if let Some(signature) = signature {
        writer.write_all(signature)?;
      };

      writer.write_u32::<LittleEndian>(*crc32)?;
      writer.write_u32::<LittleEndian>(*compressed_size)?;
      writer.write_u32::<LittleEndian>(*uncompressed_size)?;

      Ok(())
    })
  }
}
