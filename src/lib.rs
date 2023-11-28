mod central_directory;
mod central_directory_file_header;
mod data_descriptor;
mod end_of_central_directory;
mod entry;
mod error;
mod local_file_header;

pub(crate) mod prelude {
  pub(crate) use crate::error::Error;
  pub(crate) use crate::{ExpectedSize, ReadTracing, Result, WriteTracing};
  pub(crate) use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
  pub(crate) use std::io::prelude::*;
}

use prelude::*;

pub use central_directory::CentralDirectory;
pub use central_directory_file_header::CentralDirectoryFileHeader;
pub use data_descriptor::DataDescriptor;
pub use end_of_central_directory::EndOfCentralDirectory;
pub use entry::Entry;
pub use error::Error;
pub use local_file_header::LocalFileHeader;

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) trait ExpectedSize {
  fn expected_size(&self) -> u32;
}

pub(crate) trait ReadTracing: Read + Seek {
  fn trace<F: FnMut(&mut Self) -> Result<T>, T: ExpectedSize>(&mut self, mut function: F) -> Result<T> {
    #[cfg(not(feature = "logging"))]
    return function(self);

    #[cfg(feature = "logging")]
    {
      let initial_stream_position = self.stream_position()?;
      log::trace!("read -> address={:#010X?}", initial_stream_position);

      let value = function(self)?;

      let final_stream_position = self.stream_position()?;
      #[cfg(feature = "discovery")]
      {
        log::trace!("read size={}, expected={}", final_stream_position - initial_stream_position, value.expected_size());
        let expected_stream_position = initial_stream_position + u64::from(value.expected_size());
        if final_stream_position != expected_stream_position {
          log::warn!("read expected to end at {:#010X}, not {:#010X}", expected_stream_position, final_stream_position);
        }
      }
      #[cfg(not(feature = "discovery"))]
      {
        log::trace!("read size={}", final_stream_position - initial_stream_position);
      }
      log::trace!("read <- address={:#010X?}", final_stream_position);

      Ok(value)
    }
  }
}

impl<T> ReadTracing for T where T: Read + Seek {}

pub(crate) trait WriteTracing: Write + Seek {
  #[cfg_attr(not(feature = "discovery"), allow(unused_variables))]
  fn trace<F: FnMut(&mut Self) -> Result<T>, T>(&mut self, expected_size: u32, mut function: F) -> Result<T> {
    #[cfg(not(feature = "logging"))]
    return function(self);

    #[cfg(feature = "logging")]
    {
      let initial_stream_position = self.stream_position()?;
      log::trace!("write -> address={:#010X?}", initial_stream_position);

      let value = function(self)?;

      let final_stream_position = self.stream_position()?;
      #[cfg(feature = "discovery")]
      {
        log::trace!("write size={}, expected={}", final_stream_position - initial_stream_position, expected_size);
        let expected_stream_position = initial_stream_position + u64::from(expected_size);
        if final_stream_position != expected_stream_position {
          log::warn!("write expected to end at {:#010X}, not {:#010X}", expected_stream_position, final_stream_position);
        }
      }
      #[cfg(not(feature = "discovery"))]
      {
        log::trace!("write size={}", final_stream_position - initial_stream_position);
      }
      log::trace!("write <- address={:#010X?}", final_stream_position);

      Ok(value)
    }
  }
}

impl<T> WriteTracing for T where T: Write + Seek {}
