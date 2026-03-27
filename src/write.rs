// Copyright © 2025 Stephan Kunz
//! Definition of the [`Write`] trait.

use core::result::Result;

use crate::error::Error;

/// The trait for objects which are byte-oriented sinks.
pub trait Write {
	/// Flushes this output stream, ensuring that all intermediately buffered contents reach their destination.
	/// # Errors
	/// It is considered an error if not all bytes could be written due to I/O errors or EOF being reached.
	fn flush(&mut self) -> Result<(), Error>;

	/// Writes a buffer into this writer, returning how many bytes were written.
	/// # Errors
	/// Each call to write may generate an I/O error indicating that the operation could not be completed.
	/// If an error is returned then no bytes in the buffer were written to this writer.
	/// It is not considered an error if the entire buffer could not be written to this writer.
	fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;

	/// Attempts to write an entire buffer into this writer.
	///
	/// This method shall continuously call write until there is no more data to be written.
	/// This method shall not return until the entire buffer has been successfully written or an error occurs.
	/// # Errors
	/// This function shall return the first error that write returns.
	// #[cfg_attr(coverage_nightly, coverage(off))]
	fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Error> {
		while !buf.is_empty() {
			match self.write(buf) {
				Ok(n) => buf = &buf[n..],
				Err(e) => return Err(e),
			}
		}
		Ok(())
	}
}

//==== Implementations ====

/// [`Write`] implementation for [`Vec<u8>`].
impl Write for alloc::vec::Vec<u8> {
	#[inline]
	fn flush(&mut self) -> Result<(), Error> {
		Ok(())
	}

	#[inline]
	fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
		self.extend_from_slice(buf);
		Ok(buf.len())
	}
}

/// [`Write`] implementation for [`bytes::BytesMut`].
impl Write for bytes::BytesMut {
	#[inline]
	fn flush(&mut self) -> Result<(), Error> {
		Ok(())
	}

	#[inline]
	fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
		self.extend_from_slice(buf);
		Ok(buf.len())
	}
}
