use super::{DataDescriptor, LocalFileHeader};
use crate::prelude::*;
use flate2::read::DeflateDecoder;

#[derive(derivative::Derivative, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derivative(Debug)]
pub struct Entry {
  pub header: LocalFileHeader,
  #[derivative(Debug = "ignore")]
  pub data: Vec<u8>,
  pub data_descriptor: Option<DataDescriptor>,
}

impl ExpectedSize for Entry {
  fn expected_size(&self) -> u32 {
    self.header.expected_size()
      + match self.data_descriptor {
        Some(data_descriptor) => data_descriptor.compressed_size + data_descriptor.expected_size(),
        None => self.header.compressed_size,
      }
  }
}

impl Entry {
  pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
    reader.trace(|reader| {
      let header = LocalFileHeader::read(reader)?;

      let data = if header.indicates_data_descriptor_is_present() {
        let data_descriptor = DataDescriptor::read_from_end(reader)?;
        let mut data = Vec::with_capacity(data_descriptor.compressed_size.try_into()?);
        for _ in 0..data_descriptor.compressed_size {
          data.push(reader.read_u8()?);
        }
        data
      } else {
        let mut data = Vec::with_capacity(header.compressed_size.try_into()?);
        for _ in 0..header.compressed_size {
          data.push(reader.read_u8()?);
        }
        data
      };

      let data_descriptor = if header.indicates_data_descriptor_is_present() {
        Some(DataDescriptor::read(reader)?)
      } else {
        None
      };

      let value = Self { header, data, data_descriptor };

      Ok(value)
    })
  }

  pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
    writer.trace(self.expected_size(), |writer| {
      let Self { header, data, data_descriptor } = self;

      header.write(writer)?;
      writer.write_all(data)?;
      if let Some(data_descriptor) = data_descriptor {
        data_descriptor.write(writer)?;
      }

      Ok(())
    })
  }

  pub fn decompress(&self) -> Result<Vec<u8>> {
    let capacity = usize::try_from(self.header.uncompressed_size)?;
    let mut uncompressed = Vec::with_capacity(capacity);
    let mut compressed = DeflateDecoder::new(self.data.as_slice());
    compressed.read_to_end(&mut uncompressed)?;

    let crc32_of_uncompressed_data = crc32fast::hash(&uncompressed);
    if crc32_of_uncompressed_data != self.header.crc32_of_uncompressed_data {
      Ok(uncompressed)
    } else {
      Err(Error::ChecksumMismatch {
        expected: self.header.crc32_of_uncompressed_data,
        found: crc32_of_uncompressed_data,
      })
    }
  }
}
