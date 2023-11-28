use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EndOfCentralDirectory {
  pub signature: [u8; 4],
  pub number_of_this_disk: u16,
  pub disk_where_central_directory_starts: u16,
  pub number_of_central_directory_records_on_this_disk: u16,
  pub total_number_of_central_directory_records: u16,
  pub size_of_central_directory: u32,
  pub offset_of_start_of_central_directory_relative_to_start_of_archive: u32,
  pub comment_length: u16,
  pub comment: Vec<u8>,
}

impl Default for EndOfCentralDirectory {
  fn default() -> Self {
    Self {
      signature: Self::SIGNATURE,
      number_of_this_disk: 0,
      disk_where_central_directory_starts: 0,
      number_of_central_directory_records_on_this_disk: 0,
      total_number_of_central_directory_records: 0,
      size_of_central_directory: 0,
      offset_of_start_of_central_directory_relative_to_start_of_archive: 0,
      comment_length: 0,
      comment: Vec::new(),
    }
  }
}

impl ExpectedSize for EndOfCentralDirectory {
  fn expected_size(&self) -> u32 {
    22 + u32::from(self.comment_length)
  }
}

impl EndOfCentralDirectory {
  const SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];

  pub fn has_valid_signature(&self) -> bool {
    self.signature == Self::SIGNATURE
  }

  pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
    reader.trace(|reader| {
      let mut signature = [0u8; 4];
      reader.read_exact(&mut signature)?;

      if signature != Self::SIGNATURE {
        #[cfg(feature = "logging")]
        log::error!("read signature={signature:?} != {:?}", Self::SIGNATURE);
        return Err(Error::BadSignatureInEndOfCentralDirectoryHeader);
      }

      let number_of_this_disk = reader.read_u16::<LittleEndian>()?;
      let disk_where_central_directory_starts = reader.read_u16::<LittleEndian>()?;
      let number_of_central_directory_records_on_this_disk = reader.read_u16::<LittleEndian>()?;
      let total_number_of_central_directory_records = reader.read_u16::<LittleEndian>()?;
      let size_of_central_directory = reader.read_u32::<LittleEndian>()?;
      let offset_of_start_of_central_directory_relative_to_start_of_archive = reader.read_u32::<LittleEndian>()?;
      let comment_length = reader.read_u16::<LittleEndian>()?;

      let mut comment = Vec::with_capacity(comment_length.into());
      for _ in 0..comment_length {
        comment.push(reader.read_u8()?);
      }

      let value = Self {
        signature,
        number_of_this_disk,
        disk_where_central_directory_starts,
        number_of_central_directory_records_on_this_disk,
        total_number_of_central_directory_records,
        size_of_central_directory,
        offset_of_start_of_central_directory_relative_to_start_of_archive,
        comment_length,
        comment,
      };

      Ok(value)
    })
  }

  pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
    writer.trace(self.expected_size(), |writer| {
      if !self.has_valid_signature() {
        return Err(Error::BadSignatureInEndOfCentralDirectoryHeader);
      }

      let Self {
        signature,
        number_of_this_disk,
        disk_where_central_directory_starts,
        number_of_central_directory_records_on_this_disk,
        total_number_of_central_directory_records,
        size_of_central_directory,
        offset_of_start_of_central_directory_relative_to_start_of_archive,
        comment_length,
        comment,
      } = self;

      writer.write_all(signature)?;
      writer.write_u16::<LittleEndian>(*number_of_this_disk)?;
      writer.write_u16::<LittleEndian>(*disk_where_central_directory_starts)?;
      writer.write_u16::<LittleEndian>(*number_of_central_directory_records_on_this_disk)?;
      writer.write_u16::<LittleEndian>(*total_number_of_central_directory_records)?;
      writer.write_u32::<LittleEndian>(*size_of_central_directory)?;
      writer.write_u32::<LittleEndian>(*offset_of_start_of_central_directory_relative_to_start_of_archive)?;
      writer.write_u16::<LittleEndian>(*comment_length)?;
      writer.write_all(comment)?;

      Ok(())
    })
  }
}
