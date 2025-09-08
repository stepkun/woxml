// Copyright © 2025 Stephan Kunz
//! Definition of the [`Write`] trait.

use crate::error::{Error, Result};

/// The trait for objects which are byte-oriented sinks.
pub trait Write {
	/// Flushes this output stream, ensuring that all intermediately buffered contents reach their destination.
	/// # Errors
	/// It is considered an error if not all bytes could be written due to I/O errors or EOF being reached.
	fn flush(&mut self) -> Result<()>;

	/// Writes a buffer into this writer, returning how many bytes were written.
	/// # Errors
	/// Each call to write may generate an I/O error indicating that the operation could not be completed.
	/// If an error is returned then no bytes in the buffer were written to this writer.
	/// It is not considered an error if the entire buffer could not be written to this writer.
	fn write(&mut self, buf: &[u8]) -> Result<usize>;

	/// Attempts to write an entire buffer into this writer.
	///
	/// This method will continuously call write until there is no more data to be written.
	/// This method will not return until the entire buffer has been successfully written or an error occurs.
	/// # Errors
	/// This function will return the first error that write returns.
	fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
		while !buf.is_empty() {
			match self.write(buf) {
				Ok(0) => {
					return Err(Error::WriteAllEof);
				}
				Ok(n) => buf = &buf[n..],
				// Err(ref e) => e.is_interrupted() => {},
				Err(e) => return Err(e),
			}
		}
		Ok(())
	}
}
