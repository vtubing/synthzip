use super::DataDescriptor;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalFileHeader {
  pub signature: [u8; 4],
  pub version_needed_to_extract: u16,
  pub general_purpose_flags: u16,
  pub compression_method: u16,
  pub file_last_modification_time: u16,
  pub file_last_modification_date: u16,
  pub crc32_of_uncompressed_data: u32,
  pub compressed_size: u32,
  pub uncompressed_size: u32,
  pub file_name_length: u16,
  pub extra_field_length: u16,
  pub file_name: String,
  pub extra_field: Vec<u8>,
}

impl ExpectedSize for LocalFileHeader {
  fn expected_size(&self) -> u32 {
    30 + u32::from(self.file_name_length) + u32::from(self.extra_field_length)
  }
}

impl LocalFileHeader {
  const SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];

  pub fn has_valid_signature(&self) -> bool {
    self.signature == Self::SIGNATURE
  }

  pub fn validate_checksum(&self, uncompressed: &[u8]) -> bool {
    crc32fast::hash(uncompressed) == self.crc32_of_uncompressed_data
  }

  pub fn indicates_data_descriptor_is_present(&self) -> bool {
    self.general_purpose_flags & 0b00001000 != 0
  }

  pub fn update(
    &mut self,
    DataDescriptor {
      signature: _,
      crc32_of_uncompressed_data,
      compressed_size,
      uncompressed_size,
    }: DataDescriptor,
  ) -> Result<()> {
    if self.crc32_of_uncompressed_data == 0 {
      #[cfg(feature = "logging")]
      log::debug!("CRC-32 in Local File Header updated from Trailing Data Descriptor: {crc32_of_uncompressed_data:#X?}");
      self.crc32_of_uncompressed_data = crc32_of_uncompressed_data;
    } else if self.crc32_of_uncompressed_data == crc32_of_uncompressed_data {
      #[cfg(feature = "logging")]
      log::trace!("CRC-32 in Local File Header matches with Trailing Data Descriptor: {crc32_of_uncompressed_data:#X?}");
    } else {
      return Err(Error::DataDescriptorConflictsWithLocalFileHeader);
    }

    if self.compressed_size == 0 {
      #[cfg(feature = "logging")]
      log::debug!("Compressed Size in Local File Header updated from Trailing Data Descriptor: {compressed_size}");
      self.compressed_size = compressed_size;
    } else if self.compressed_size == compressed_size {
      #[cfg(feature = "logging")]
      log::trace!("Compressed Size in Local File Header matches with Trailing Data Descriptor")
    } else {
      return Err(Error::DataDescriptorConflictsWithLocalFileHeader);
    }

    if self.uncompressed_size == 0 {
      #[cfg(feature = "logging")]
      log::debug!("Uncompressed Size in Local File Header updated from Trailing Data Descriptor: {uncompressed_size}");
      self.uncompressed_size = uncompressed_size;
    } else if self.uncompressed_size == uncompressed_size {
      #[cfg(feature = "logging")]
      log::trace!("Uncompressed Size in Local File Header matches with Trailing Data Descriptor")
    } else {
      return Err(Error::DataDescriptorConflictsWithLocalFileHeader);
    }

    Ok(())
  }

  pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
    reader.trace(|reader| {
      let mut signature = [0u8; 4];
      reader.read_exact(&mut signature)?;

      if signature != Self::SIGNATURE {
        #[cfg(feature = "logging")]
        log::error!("read signature={signature:?} != {:?}", Self::SIGNATURE);
        return Err(Error::BadSignatureInLocalFileHeader);
      }

      let version_needed_to_extract = reader.read_u16::<LittleEndian>()?;
      let general_purpose_flags = reader.read_u16::<LittleEndian>()?;
      let compression_method = reader.read_u16::<LittleEndian>()?;
      let file_last_modification_time = reader.read_u16::<LittleEndian>()?;
      let file_last_modification_date = reader.read_u16::<LittleEndian>()?;
      let crc32_of_uncompressed_data = reader.read_u32::<LittleEndian>()?;
      let compressed_size = reader.read_u32::<LittleEndian>()?;
      let uncompressed_size = reader.read_u32::<LittleEndian>()?;
      let file_name_length = reader.read_u16::<LittleEndian>()?;
      let extra_field_length = reader.read_u16::<LittleEndian>()?;

      let file_name = {
        let mut file_name = Vec::with_capacity(file_name_length.into());
        for _ in 0..file_name_length {
          file_name.push(reader.read_u8()?);
        }
        String::from_utf8(file_name)?
      };

      let mut extra_field = Vec::with_capacity(extra_field_length.into());
      for _ in 0..extra_field_length {
        extra_field.push(reader.read_u8()?);
      }

      let value = Self {
        signature,
        version_needed_to_extract,
        general_purpose_flags,
        compression_method,
        file_last_modification_time,
        file_last_modification_date,
        crc32_of_uncompressed_data,
        compressed_size,
        uncompressed_size,
        file_name_length,
        extra_field_length,
        file_name,
        extra_field,
      };

      Ok(value)
    })
  }

  pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
    writer.trace(self.expected_size(), |writer| {
      if !self.has_valid_signature() {
        return Err(Error::BadSignatureInLocalFileHeader);
      }

      let Self {
        signature,
        version_needed_to_extract,
        general_purpose_flags,
        compression_method,
        file_last_modification_time,
        file_last_modification_date,
        crc32_of_uncompressed_data: crc32,
        compressed_size,
        uncompressed_size,
        file_name_length,
        extra_field_length,
        file_name,
        extra_field,
      } = self;

      writer.write_all(signature)?;
      writer.write_u16::<LittleEndian>(*version_needed_to_extract)?;
      writer.write_u16::<LittleEndian>(*general_purpose_flags)?;
      writer.write_u16::<LittleEndian>(*compression_method)?;
      writer.write_u16::<LittleEndian>(*file_last_modification_time)?;
      writer.write_u16::<LittleEndian>(*file_last_modification_date)?;
      writer.write_u32::<LittleEndian>(*crc32)?;
      writer.write_u32::<LittleEndian>(*compressed_size)?;
      writer.write_u32::<LittleEndian>(*uncompressed_size)?;
      writer.write_u16::<LittleEndian>(*file_name_length)?;
      writer.write_u16::<LittleEndian>(*extra_field_length)?;
      writer.write_all(file_name.as_bytes())?;
      writer.write_all(extra_field)?;

      Ok(())
    })
  }
}
