// Copyright © 2025 Stephan Kunz
// Copyright © of xml_writer::XmlWriter Piotr Zolnierek

// region:		--- modules
use alloc::{
	boxed::Box,
	string::{String, ToString},
	vec::Vec,
};

use crate::{
	error::{Error, Result},
	write::Write,
};
// endregion:	--- modules

// region:		--- constants
/// Multiple used literal definitions
const CLOSE: &str = ">";
const CLOSE_CLOSE: &str = "/>";
const OPEN: &str = "<";
const SELF_CLOSE_OPEN: &str = "</";
const SPACE: &str = " ";
const EQUAL_QUOTE: &str = "=\"";
const QUOTE: &str = "\"";
// endregion:	--- constants

// region:		--- XmlWriter
/// The `XmlWriter` himself.
/// Elements without children are automatically self-closing.
/// In 'pretty' mode the writer will
/// - indent all opening elements on a new line
/// - put closing elements into own line
pub struct XmlWriter<'a, Buffer: Write> {
	/// element stack
	/// `bool` indicates element having children
	stack: Vec<(&'a str, bool)>,
	/// namespace stack
	ns_stack: Vec<Option<&'a str>>,
	buffer: Box<Buffer>,
	/// An XML namespace that all elements will be part of, unless `None`
	namespace: Option<&'a str>,
	/// If `true` it will
	/// - indent all opening elements on a new line
	/// - put closing elements into own line
	/// - elements without children are automatically self-closing
	pretty: bool,
	/// if `true` an element is open
	opened: bool,
	/// newline/indentation indicator
	newline: bool,
}

impl<Buffer: Write> core::fmt::Debug for XmlWriter<'_, Buffer> {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		Ok(write!(
			f,
			"XmlWriter {{ stack: {:?}, namespaces: {:?}, opened: {} }}",
			self.stack, self.ns_stack, self.opened
		)?)
	}
}

impl<'a, W: Write> XmlWriter<'a, W> {
	/// Create a new writer with `compact` output which will
	/// - omit all indentations and newlines
	/// - elements without children are automatically self-closing
	pub fn compact_mode(buffer: W) -> Self {
		XmlWriter {
			stack: Vec::new(),
			ns_stack: Vec::new(),
			buffer: Box::new(buffer),
			namespace: None,
			pretty: false,
			opened: false,
			newline: false,
		}
	}

	/// Create a new writer with `pretty` output which will
	/// - indent all opening elements on a new line
	/// - put closing elements into own line
	/// - elements without children are automatically self-closing
	pub fn pretty_mode(buffer: W) -> Self {
		XmlWriter {
			stack: Vec::new(),
			ns_stack: Vec::new(),
			buffer: Box::new(buffer),
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
	/// - when opening a namespace without having an element
	pub fn ns_decl(&mut self, ns_map: &Vec<(Option<&'a str>, &'a str)>) -> Result<()> {
		if !self.opened {
			return Err(Error::OpenNamespaceWithoutElement);
		}

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
		self.write(OPEN)?;
		let ns = self.namespace;
		self.ns_prefix(ns)?;
		self.write(name)?;
		self.write(CLOSE_CLOSE)
	}

	/// Write an element with inlined text content (escaped)
	/// # Errors
	/// - if writing to buffer fails
	pub fn elem_text(&mut self, name: &str, text: &str) -> Result<()> {
		self.close_elem(false)?;
		self.indent()?;
		self.write(OPEN)?;
		let ns = self.namespace;
		self.ns_prefix(ns)?;
		self.write(name)?;
		self.write(CLOSE)?;

		self.escape(text, false)?;

		self.write(SELF_CLOSE_OPEN)?;
		self.write(name)?;
		self.write(CLOSE)
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
		self.write(OPEN)?;
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
			if has_children {
				self.write(CLOSE)?;
			} else {
				self.write(CLOSE_CLOSE)?;
			}
			self.opened = false;
		}
		Ok(())
	}

	/// End and elem
	/// # Errors
	/// - if writing to buffer fails
	/// - when trying to close a namespace without having one opened
	/// - when trying to close an element without having one opened
	pub fn end_elem(&mut self) -> Result<()> {
		self.close_elem(false)?;
		let Some(ns) = self.ns_stack.pop() else {
			return Err(Error::CloseNamespace);
		};
		match self.stack.pop() {
			Some((name, children)) => {
				// elem without children have been self-closed
				if !children {
					return Ok(());
				}
				if self.newline {
					self.indent()?;
				}
				self.newline = true;
				self.write(SELF_CLOSE_OPEN)?;
				self.ns_prefix(ns)?;
				self.write(name)?;
				self.write(CLOSE)?;
				Ok(())
			}
			None => Err(Error::CloseElement),
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
		self.write(OPEN)?;
		let ns = self.namespace;
		self.ns_prefix(ns)?;
		self.write(name)?;
		self.write(CLOSE_CLOSE)
	}

	/// Write an attr, make sure name and value contain only allowed chars.
	/// For an escaping version use `attr_esc`
	/// # Errors
	/// - if writing to buffer fails
	/// - when writing attributes without having an element
	pub fn attr(&mut self, name: &str, value: &str) -> Result<()> {
		if !self.opened {
			return Err(Error::WriteWithoutElement);
		}
		self.write(SPACE)?;
		self.write(name)?;
		self.write(EQUAL_QUOTE)?;
		self.write(value)?;
		self.write(QUOTE)
	}

	/// Write an attr, make sure name contains only allowed chars.
	/// # Errors
	/// - if writing to buffer fails
	/// - when writing attributes without having an element
	pub fn attr_esc(&mut self, name: &str, value: &str) -> Result<()> {
		if !self.opened {
			return Err(Error::WriteWithoutElement);
		}
		self.write(SPACE)?;
		self.escape(name, true)?;
		self.write(EQUAL_QUOTE)?;
		self.escape(value, false)?;
		self.write(QUOTE)
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
		self.buffer.write_all(text.as_bytes())?;
		Ok(())
	}

	/// Raw write, no escaping, no safety net, use at own risk
	/// # Errors
	/// - if writing to buffer fails
	fn write_slice(&mut self, slice: &[u8]) -> Result<()> {
		self.buffer.write_all(slice)?;
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
		self.buffer.flush()?;
		Ok(())
	}

	/// Consume the `XmlWriter` and return the inner Writer
	#[must_use]
	pub fn into_inner(self) -> W {
		*self.buffer
	}
}
// endregion:	--- XmlWriter

// region:		--- vec<u8>
/// Fallible conversion to [`String`] for [`Vec<u8>`].
impl<'a> TryFrom<XmlWriter<'a, Vec<u8>>> for String {
	type Error = Error;

	fn try_from(writer: XmlWriter<'a, Vec<u8>>) -> core::result::Result<Self, Self::Error> {
		Self::from_utf8(writer.into_inner()).map_or(Err(Error::ParsingUtf8), Ok)
	}
}

/// [`Write`] implementation for [`Vec<u8>`].
impl Write for alloc::vec::Vec<u8> {
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
// endregion:	--- Vec<u8>

// region:		--- BytesMut
/// Fallible conversion to [`String`] for [`bytes::BytesMut`].
impl<'a> TryFrom<XmlWriter<'a, bytes::BytesMut>> for String {
	type Error = Error;

	fn try_from(writer: XmlWriter<'a, bytes::BytesMut>) -> core::result::Result<Self, Self::Error> {
		Self::from_utf8(writer.into_inner().to_vec()).map_or(Err(Error::ParsingUtf8), Ok)
	}
}

/// [`Write`] implementation for [`bytes::BytesMut`].
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
// endregion:	--- BytesMut
