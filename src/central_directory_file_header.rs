use super::LocalFileHeader;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CentralDirectoryFileHeader {
  pub signature: [u8; 4],
  pub version_made_by: u16,
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
  pub file_comment_length: u16,
  pub disk_number_where_file_starts: u16,
  pub internal_file_attributes: u16,
  pub external_file_attributes: u32,
  pub relative_offset_of_local_file_header: u32,
  pub file_name: String,
  pub extra_field: Vec<u8>,
  pub file_comment: Vec<u8>,
}

impl ExpectedSize for CentralDirectoryFileHeader {
  fn expected_size(&self) -> u32 {
    46 + u32::from(self.file_name_length) + u32::from(self.extra_field_length) + u32::from(self.file_comment_length)
  }
}

impl CentralDirectoryFileHeader {
  const SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x01, 0x02];

  pub fn has_valid_signature(&self) -> bool {
    self.signature == Self::SIGNATURE
  }

  pub fn validate_checksum(&self, uncompressed: &[u8]) -> bool {
    crc32fast::hash(uncompressed) == self.crc32_of_uncompressed_data
  }

  pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
    reader.trace(|reader| {
      let mut signature = [0u8; 4];
      reader.read_exact(&mut signature)?;

      if signature != Self::SIGNATURE {
        #[cfg(feature = "logging")]
        log::error!("read signature={signature:?} != {:?}", Self::SIGNATURE);
        return Err(Error::BadSignatureInCentralDirectoryFileHeader);
      }

      let version_made_by = reader.read_u16::<LittleEndian>()?;
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
      let file_comment_length = reader.read_u16::<LittleEndian>()?;
      let disk_number_where_file_starts = reader.read_u16::<LittleEndian>()?;
      let internal_file_attributes = reader.read_u16::<LittleEndian>()?;
      let external_file_attributes = reader.read_u32::<LittleEndian>()?;
      let relative_offset_of_local_file_header = reader.read_u32::<LittleEndian>()?;

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

      let mut file_comment = Vec::with_capacity(file_comment_length.into());
      for _ in 0..file_comment_length {
        file_comment.push(reader.read_u8()?);
      }

      let value = Self {
        signature,
        version_made_by,
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
        file_comment_length,
        disk_number_where_file_starts,
        internal_file_attributes,
        external_file_attributes,
        relative_offset_of_local_file_header,
        file_name,
        extra_field,
        file_comment,
      };

      Ok(value)
    })
  }

  pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
    writer.trace(self.expected_size(), |writer| {
      if !self.has_valid_signature() {
        return Err(Error::BadSignatureInCentralDirectoryFileHeader);
      }

      let Self {
        signature,
        version_made_by,
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
        file_comment_length,
        disk_number_where_file_starts,
        internal_file_attributes,
        external_file_attributes,
        relative_offset_of_local_file_header,
        file_name,
        extra_field,
        file_comment,
      } = self;

      writer.write_all(signature)?;
      writer.write_u16::<LittleEndian>(*version_made_by)?;
      writer.write_u16::<LittleEndian>(*version_needed_to_extract)?;
      writer.write_u16::<LittleEndian>(*general_purpose_flags)?;
      writer.write_u16::<LittleEndian>(*compression_method)?;
      writer.write_u16::<LittleEndian>(*file_last_modification_time)?;
      writer.write_u16::<LittleEndian>(*file_last_modification_date)?;
      writer.write_u32::<LittleEndian>(*crc32_of_uncompressed_data)?;
      writer.write_u32::<LittleEndian>(*compressed_size)?;
      writer.write_u32::<LittleEndian>(*uncompressed_size)?;
      writer.write_u16::<LittleEndian>(*file_name_length)?;
      writer.write_u16::<LittleEndian>(*extra_field_length)?;
      writer.write_u16::<LittleEndian>(*file_comment_length)?;
      writer.write_u16::<LittleEndian>(*disk_number_where_file_starts)?;
      writer.write_u16::<LittleEndian>(*internal_file_attributes)?;
      writer.write_u32::<LittleEndian>(*external_file_attributes)?;
      writer.write_u32::<LittleEndian>(*relative_offset_of_local_file_header)?;
      writer.write_all(file_name.as_bytes())?;
      writer.write_all(extra_field)?;
      writer.write_all(file_comment)?;

      Ok(())
    })
  }
}

impl From<LocalFileHeader> for CentralDirectoryFileHeader {
  fn from(local_file_header: LocalFileHeader) -> Self {
    let LocalFileHeader {
      signature: _,
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
    } = local_file_header;

    Self {
      signature: Self::SIGNATURE,
      version_made_by: version_needed_to_extract,
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
      file_comment_length: 0,
      disk_number_where_file_starts: 0,
      internal_file_attributes: 0,
      external_file_attributes: 0,
      relative_offset_of_local_file_header: 0,
      file_name,
      extra_field,
      file_comment: Vec::new(),
    }
  }
}
