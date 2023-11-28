#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("bad signature in central directory file header")]
  BadSignatureInCentralDirectoryFileHeader,
  #[error("bad signature in end of central directory header")]
  BadSignatureInEndOfCentralDirectoryHeader,
  #[error("bad signature in local file header")]
  BadSignatureInLocalFileHeader,
  #[error("checksum mismatch: expected={expected:#010X?}, found={found:#010X?}")]
  ChecksumMismatch { expected: u32, found: u32 },
  #[error("data descriptor conflicts with local file header")]
  DataDescriptorConflictsWithLocalFileHeader,
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  TryFromInt(#[from] std::num::TryFromIntError),
  #[error(transparent)]
  Utf8(#[from] std::string::FromUtf8Error),
}
