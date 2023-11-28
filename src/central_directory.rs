use super::{CentralDirectoryFileHeader, EndOfCentralDirectory, Entry};
use crate::prelude::*;
use std::io::{Read, Seek, SeekFrom, Write};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CentralDirectory {
  pub files: Vec<CentralDirectoryFileHeader>,
  pub end: EndOfCentralDirectory,
}

impl ExpectedSize for CentralDirectory {
  fn expected_size(&self) -> u32 {
    self.files.iter().map(CentralDirectoryFileHeader::expected_size).sum::<u32>() + self.end.expected_size()
  }
}

impl CentralDirectory {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add(&mut self, entry: &Entry) -> Result<()> {
    let mut header = entry.header.clone();
    if header.indicates_data_descriptor_is_present() {
      #[allow(clippy::clone_on_copy)]
      if let Some(data_descriptor) = entry.data_descriptor.clone() {
        header.update(data_descriptor)?;
      }
    }

    let mut file = CentralDirectoryFileHeader::from(header);
    if let Some(last) = self.files.last() {
      file.relative_offset_of_local_file_header = last.relative_offset_of_local_file_header + last.expected_size();
    }

    self.files.push(file);
    self.end.number_of_central_directory_records_on_this_disk = self.files.len().try_into()?;
    self.end.total_number_of_central_directory_records = self.files.len().try_into()?;
    self.end.size_of_central_directory = self.files.iter().map(CentralDirectoryFileHeader::expected_size).sum();
    self.end.offset_of_start_of_central_directory_relative_to_start_of_archive += entry.expected_size();
    Ok(())
  }
}

impl CentralDirectory {
  pub fn read_from_end<R: Read + Seek>(reader: &mut R) -> Result<Self> {
    let initial_stream_position = reader.stream_position()?;
    reader.seek(SeekFrom::End(-22))?;
    let value = Self::read(reader)?;
    reader.seek(SeekFrom::Start(initial_stream_position))?;
    Ok(value)
  }

  pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
    reader.trace(|reader| {
      let end = EndOfCentralDirectory::read(reader)?;

      reader.seek(SeekFrom::Start(end.offset_of_start_of_central_directory_relative_to_start_of_archive.into()))?;

      let mut files = Vec::new();
      for _ in 0..end.total_number_of_central_directory_records {
        let file = CentralDirectoryFileHeader::read(reader)?;
        files.push(file);
      }

      let value = Self { files, end };

      Ok(value)
    })
  }

  pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
    writer.trace(self.expected_size(), |writer| {
      let Self { files, end } = self;

      for file in files {
        file.write(writer)?;
      }

      end.write(writer)?;

      Ok(())
    })
  }
}
