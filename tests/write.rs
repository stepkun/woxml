// Copyright © 2026 Stephan Kunz

//! Tests for the [`Write`] trait and its default [`Write::write_all`] implementation.
#![allow(clippy::unwrap_used)]

#[cfg(feature = "std")]
extern crate std;

use std::vec::Vec;
use woxml::{Error, Write, XmlWriter};

// ---- custom Write impls exercising the default write_all -----------------------

/// A Write implementation that writes in fixed-size chunks and relies on the
/// default `write_all` provided by the trait.
struct ChunkWriter {
	buf: Vec<u8>,
	chunk_size: usize,
}

impl ChunkWriter {
	const fn new(chunk_size: usize) -> Self {
		Self {
			buf: Vec::new(),
			chunk_size,
		}
	}
}

impl Write for ChunkWriter {
	fn flush(&mut self) -> Result<(), Error> {
		Ok(())
	}

	fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
		let n = self.chunk_size.min(buf.len());
		self.buf.extend_from_slice(&buf[..n]);
		Ok(n)
	}
	// No write_all override → exercises the default trait implementation
}

/// A Write that always returns Ok(0) from write, triggering the `WriteAllEof`
/// branch in the default `write_all`.
struct StallWriter;

impl Write for StallWriter {
	fn flush(&mut self) -> Result<(), Error> {
		Ok(())
	}

	fn write(&mut self, _buf: &[u8]) -> Result<usize, Error> {
		Ok(0)
	}
}

/// A Write that always returns Err from write, triggering the Err branch
/// in the default `write_all`.
struct FailWriter;

impl Write for FailWriter {
	fn flush(&mut self) -> Result<(), Error> {
		Ok(())
	}

	fn write(&mut self, _buf: &[u8]) -> Result<usize, Error> {
		Err(Error::WriteAllEof)
	}
}

// ---- default Write::write_all --------------------------------------------------

#[test]
fn default_write_all_chunks() -> Result<(), Error> {
	// ChunkWriter writes 3 bytes at a time; the default write_all loops until done
	let buf = ChunkWriter::new(3);
	let mut xml = XmlWriter::compact_mode(buf);
	xml.begin_elem("root")?;
	xml.end_elem()?;

	let inner = xml.into_inner();
	assert_eq!(&inner.buf, b"<root/>");
	// same with chunk size > elements to write
	let buf = ChunkWriter::new(10);
	let mut xml = XmlWriter::compact_mode(buf);
	xml.begin_elem("root")?;
	xml.end_elem()?;

	let inner = xml.into_inner();
	assert_eq!(&inner.buf, b"<root/>");
	Ok(())
}

#[test]
fn default_write_all_stall_returns_eof() {
	// StallWriter.write() always returns Ok(0); write_all must return WriteAllEof
	let mut w = StallWriter;
	let err = w.write_all(b"hello").unwrap_err();
	assert!(matches!(err, Error::WriteAllEof));
}

#[test]
fn default_write_all_error_propagates() {
	// FailWriter.write() returns Err; write_all must propagate that error
	let mut w = FailWriter;
	let err = w.write_all(b"hello").unwrap_err();
	assert!(matches!(err, Error::WriteAllEof));
}

#[test]
fn default_write_all_empty_buf() -> Result<(), Error> {
	// write_all with an empty slice must succeed without calling write at all
	let mut w = ChunkWriter::new(5);
	w.write_all(b"")?;
	let mut w = StallWriter;
	w.write_all(b"")?;
	let mut w = FailWriter;
	w.write_all(b"")?;
	Ok(())
}
