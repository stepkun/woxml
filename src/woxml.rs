// Copyright © 2025 Stephan Kunz
// Copyright © of xml_writer::XmlWriter Piotr Zolnierek

extern crate alloc;

use alloc::{boxed::Box, string::ToString, vec::Vec};
use core::fmt::{Debug, Formatter};

use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

/// Error type
#[derive(Error, Debug)]
pub enum Error {
	/// Pass through `core::str::Utf8Error`
	#[error("{0}")]
	Utf8(#[from] core::str::Utf8Error),
	/// Pass through `core::fmt::Error`
	#[error("{0}")]
	Core(#[from] core::fmt::Error),
	/// failed to write whole buffer
	#[error("failed to write whole buffer")]
	WriteAllEof,
}

/// The trait for objects which are byte-oriented sinks.
pub trait Write {
	/// Flushes this output stream, ensuring that all intermediately buffered contents reach their destination.
	/// # Errors
	/// It is considered an error if not all bytes could be written due to I/O errors or EOF being reached.
	fn flush(&mut self) -> Result<()>;

	/// Writes a buffer into this writer, returning how many bytes were written.
	///
	/// # Errors
	/// Each call to write may generate an I/O error indicating that the operation could not be completed. If an error is returned then no bytes in the buffer were written to this writer.
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
				//Err(ref e) => e.is_interrupted() => {}
				Err(e) => return Err(e),
			}
		}
		Ok(())
	}
}

// #[cfg(not(feature = "std"))]
impl Write for Vec<u8> {
	#[inline]
	fn flush(&mut self) -> Result<()> {
		Ok(())
	}

	#[inline]
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.extend_from_slice(buf);
		Ok(buf.len())
	}

	#[inline]
	fn write_all(&mut self, data: &[u8]) -> Result<()> {
		if self.write(data)? < data.len() {
			Err(Error::WriteAllEof)
		} else {
			Ok(())
		}
	}
}

// #[cfg(not(feature = "std"))]
impl Write for bytes::BytesMut {
	#[inline]
	fn flush(&mut self) -> Result<()> {
		Ok(())
	}

	#[inline]
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		self.extend_from_slice(buf);
		Ok(buf.len())
	}

	#[inline]
	fn write_all(&mut self, data: &[u8]) -> Result<()> {
		if self.write(data)? < data.len() {
			Err(Error::WriteAllEof)
		} else {
			Ok(())
		}
	}
}

// /// Implement the trait for all types that implement [`core::fmt::Write`]
// #[cfg(not(feature = "std"))]
// impl<T> Write for T
// where
//     T: core::fmt::Write,
// {
//     #[inline]
//     fn flush(&mut self) -> Result<()> {
//         Ok(())
//     }

//     #[inline]
//     fn write(&mut self, buf: &[u8]) -> Result<usize> {
//         let str = str::from_utf8(buf)?;
//         let len = str.len();
//         T::write_str(self, str)?;
//         Ok(len)
//     }
// }

// /// Implement the trait for all types that implement [`std::io::Write`]
// #[cfg(feature = "std")]
// impl<T> Write for T
// where
//     T: std::io::Write,
// {
//     #[inline]
//     fn flush(&mut self) -> Result<()> {
//         T::flush(self)?;
//         Ok(())
//     }

//     #[inline]
//     fn write(&mut self, buf: &[u8]) -> Result<usize> {
//         let len = T::write(self, buf)?;
//         Ok(len)
//     }
// }

/// The `XmlWriter` himself.
pub struct XmlWriter<'a, Buf: Write> {
	/// element stack
	/// `bool` indicates element having children
	stack: Vec<(&'a str, bool)>,
	/// namespace stack
	ns_stack: Vec<Option<&'a str>>,
	writer: Box<Buf>,
	/// An XML namespace that all elements will be part of, unless `None`
	namespace: Option<&'a str>,
	/// If `true` it will
	/// - indent all opening elements on a new line
	/// - put closing elements into own line
	/// - elements without children are automatically self-closing
	pretty: bool,
	/// if `true` current element is open
	opened: bool,
	/// newline/indentation indicator
	newline: bool,
}

impl<Buf: Write> Debug for XmlWriter<'_, Buf> {
	fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
		Ok(write!(f, "XmlWriter {{ stack: {:?}, opened: {} }}", self.stack, self.opened)?)
	}
}

impl<'a, W: Write> XmlWriter<'a, W> {
	/// Create a new writer with `compact` output
	pub fn compact_mode(writer: W) -> Self {
		XmlWriter {
			stack: Vec::new(),
			ns_stack: Vec::new(),
			writer: Box::new(writer),
			namespace: None,
			pretty: false,
			opened: false,
			newline: false,
		}
	}

	/// Create a new writer with `pretty` output
	pub fn pretty_mode(writer: W) -> Self {
		XmlWriter {
			stack: Vec::new(),
			ns_stack: Vec::new(),
			writer: Box::new(writer),
			namespace: None,
			pretty: true,
			opened: false,
			newline: false,
		}
	}

	/// Switch to `compact` mode
	pub const fn set_compact_mode(&mut self) {
		self.pretty = false;
	}

	/// Switch to `pretty` mode
	pub const fn set_pretty_mode(&mut self) {
		self.pretty = true;
	}

	/// Get the namespace
	#[must_use]
	pub const fn namespace(&self) -> Option<&'a str> {
		self.namespace
	}

	/// Set the namespace
	pub const fn set_namespace(&mut self, namespace: &'a str) {
		self.namespace = Some(namespace);
	}

	/// Remove/Unset the namespace
	pub const fn unset_namespace(&mut self) {
		self.namespace = None;
	}

	/// Write the DTD.
	/// # Errors
	/// - if writing to buffer fails
	pub fn dtd(&mut self, encoding: &str) -> Result<()> {
		self.write("<?xml version=\"1.0\" encoding=\"")?;
		self.write(encoding)?;
		self.write("\" ?>\n")
	}

	fn indent(&mut self) -> Result<()> {
		if self.pretty {
			if self.newline {
				self.write("\n")?;
			} else {
				self.newline = true;
			}
			for _ in 0..self.stack.len() {
				self.write("  ")?;
			}
		}
		Ok(())
	}

	/// Write a namespace prefix for the current element,
	/// if there is one set
	fn ns_prefix(&mut self, namespace: Option<&'a str>) -> Result<()> {
		if let Some(ns) = namespace {
			self.write(ns)?;
			self.write(":")?;
		}
		Ok(())
	}

	/// Writes namespace declarations (xmlns:xx) into the currently open element.
	/// # Errors
	/// - if writing to buffer fails
	/// # Panics
	/// - when opening a namespace without having an element
	pub fn ns_decl(&mut self, ns_map: &Vec<(Option<&'a str>, &'a str)>) -> Result<()> {
		assert!(
			self.opened,
			"Attempted to write namespace decl to elem, when no elem was opened, stack {:?}",
			self.stack
		);

		for item in ns_map {
			let name = item
				.0
				.map_or_else(|| "xmlns".to_string(), |pre| "xmlns:".to_string() + pre);
			self.attr(&name, item.1)?;
		}
		Ok(())
	}

	/// Write a self-closing element like <br/>.
	/// # Errors
	/// - if writing to buffer fails
	pub fn elem(&mut self, name: &str) -> Result<()> {
		self.close_elem(false)?;
		self.indent()?;
		self.write("<")?;
		let ns = self.namespace;
		self.ns_prefix(ns)?;
		self.write(name)?;
		self.write("/>")
	}

	/// Write an element with inlined text content (escaped)
	/// # Errors
	/// - if writing to buffer fails
	pub fn elem_text(&mut self, name: &str, text: &str) -> Result<()> {
		self.close_elem(false)?;
		self.indent()?;
		self.write("<")?;
		let ns = self.namespace;
		self.ns_prefix(ns)?;
		self.write(name)?;
		self.write(">")?;

		self.escape(text, false)?;

		self.write("</")?;
		self.write(name)?;
		self.write(">")
	}

	/// Begin an elem, make sure name contains only allowed chars
	/// # Errors
	/// - if writing to buffer fails
	pub fn begin_elem(&mut self, name: &'a str) -> Result<()> {
		self.close_elem(true)?;
		// change previous elem to having children
		if let Some(mut previous) = self.stack.pop() {
			previous.1 = true;
			self.stack.push(previous);
		}
		self.indent()?;
		self.stack.push((name, false));
		self.ns_stack.push(self.namespace);
		self.write("<")?;
		self.opened = true;
		// stderr().write_fmt(format_args!("\nbegin {}", name));
		let ns = self.namespace;
		self.ns_prefix(ns)?;
		self.write(name)
	}

	/// Close an elem if open, do nothing otherwise.
	/// # Errors
	/// - if writing to buffer fails
	fn close_elem(&mut self, has_children: bool) -> Result<()> {
		if self.opened {
			if self.pretty && !has_children {
				self.write("/>")?;
			} else {
				self.write(">")?;
			}
			self.opened = false;
		}
		Ok(())
	}

	/// End and elem
	/// # Errors
	/// - if writing to buffer fails
	/// # Panics
	/// - when trying to close a namespace without having one opened
	pub fn end_elem(&mut self) -> Result<()> {
		self.close_elem(false)?;
		let ns = self.ns_stack.pop().unwrap_or_else(|| {
			panic!(
				"Attempted to close namespaced element without corresponding open namespace, stack {:?}",
				self.ns_stack
			)
		});
		match self.stack.pop() {
			Some((name, children)) => {
				if self.pretty {
					// elem without children have been self-closed
					if !children {
						return Ok(());
					}
					if self.newline {
						self.indent()?;
					}
					self.newline = true;
				}
				self.write("</")?;
				self.ns_prefix(ns)?;
				self.write(name)?;
				self.write(">")?;
				Ok(())
			}
			None => panic!("Attempted to close an elem, when none was open, stack {:?}", self.stack),
		}
	}

	/// Begin an empty elem
	/// # Errors
	/// - if writing to buffer fails
	pub fn empty_elem(&mut self, name: &'a str) -> Result<()> {
		self.close_elem(true)?;
		// change previous elem to having children
		if let Some(mut previous) = self.stack.pop() {
			previous.1 = true;
			self.stack.push(previous);
		}
		self.indent()?;
		self.write("<")?;
		let ns = self.namespace;
		self.ns_prefix(ns)?;
		self.write(name)?;
		self.write("/>")
	}

	/// Write an attr, make sure name and value contain only allowed chars.
	/// For an escaping version use `attr_esc`
	/// # Errors
	/// - if writing to buffer fails
	/// # Panics
	/// - when writing attributes without having an element
	pub fn attr(&mut self, name: &str, value: &str) -> Result<()> {
		assert!(
			self.opened,
			"Attempted to write attr to elem, when no elem was opened, stack {:?}",
			self.stack
		);
		self.write(" ")?;
		self.write(name)?;
		self.write("=\"")?;
		self.write(value)?;
		self.write("\"")
	}

	/// Write an attr, make sure name contains only allowed chars.
	/// # Errors
	/// - if writing to buffer fails
	/// # Panics
	/// - when writing attributes without having an element
	pub fn attr_esc(&mut self, name: &str, value: &str) -> Result<()> {
		assert!(
			self.opened,
			"Attempted to write attr to elem, when no elem was opened, stack {:?}",
			self.stack
		);
		self.write(" ")?;
		self.escape(name, true)?;
		self.write("=\"")?;
		self.escape(value, false)?;
		self.write("\"")
	}

	/// Escape identifiers or text.
	/// # Errors
	/// - if writing to buffer fails
	fn escape(&mut self, text: &str, ident: bool) -> Result<()> {
		for c in text.chars() {
			match c {
				'"' => self.write("&quot;")?,
				'\'' => self.write("&apos;")?,
				'&' => self.write("&amp;")?,
				'<' => self.write("&lt;")?,
				'>' => self.write("&gt;")?,
				'\\' if ident => self.write("\\\\")?,
				_ => self.write_slice(c.encode_utf8(&mut [0; 4]).as_bytes())?,
			}
		}
		Ok(())
	}

	/// Write a text content, escapes the text automatically
	/// # Errors
	/// - if writing to buffer fails
	pub fn text(&mut self, text: &str) -> Result<()> {
		self.close_elem(true)?;
		// change previous elem to having children
		if let Some(mut previous) = self.stack.pop() {
			previous.1 = true;
			self.stack.push(previous);
		}
		self.newline = false;
		self.escape(text, false)
	}

	/// Raw write, no escaping, no safety net, use at own risk
	/// # Errors
	/// - if writing to buffer fails
	pub fn write(&mut self, text: &str) -> Result<()> {
		self.writer.write_all(text.as_bytes())?;
		Ok(())
	}

	/// Raw write, no escaping, no safety net, use at own risk
	/// # Errors
	/// - if writing to buffer fails
	fn write_slice(&mut self, slice: &[u8]) -> Result<()> {
		self.writer.write_all(slice)?;
		Ok(())
	}

	/// Write a CDATA.
	/// # Errors
	/// - if writing to buffer fails
	pub fn cdata(&mut self, cdata: &str) -> Result<()> {
		self.close_elem(true)?;
		// change previous elem to having children
		if let Some(mut previous) = self.stack.pop() {
			previous.1 = true;
			self.stack.push(previous);
		}
		if self.pretty {
			self.indent()?;
		}
		self.write("<![CDATA[")?;
		self.write(cdata)?;
		self.write("]]>")
	}

	/// Write a comment
	/// # Errors
	/// - if writing to buffer fails
	pub fn comment(&mut self, comment: &str) -> Result<()> {
		self.close_elem(true)?;
		// change previous elem to having children
		if let Some(mut previous) = self.stack.pop() {
			previous.1 = true;
			self.stack.push(previous);
		}
		self.indent()?;
		self.write("<!-- ")?;
		self.escape(comment, false)?;
		self.write(" -->")
	}

	/// Close all open elems
	/// # Errors
	/// - if writing to buffer fails
	pub fn close(&mut self) -> Result<()> {
		for _ in 0..self.stack.len() {
			self.end_elem()?;
		}
		Ok(())
	}

	/// Flush the underlying Writer
	/// # Errors
	/// - if writing to buffer fails
	pub fn flush(&mut self) -> Result<()> {
		self.writer.flush()?;
		Ok(())
	}

	/// Consume the `XmlWriter` and return the inner Writer
	#[must_use]
	pub fn into_inner(self) -> W {
		*self.writer
	}
}
