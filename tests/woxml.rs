// Copyright © 2025 Stephan Kunz

//! Testing of woxml library

#[cfg(feature = "std")]
extern crate std;

use std::{format, println, str, string::String, vec, vec::Vec};
use woxml::{Error, XmlWriter};

// ---- dtd -----------------------------------------------------------------------

#[test]
fn dtd_compact() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.dtd("UTF-8")?;
	xml.begin_elem("root")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<root/>");
	Ok(())
}

#[test]
fn dtd_pretty() -> Result<(), Error> {
	let mut xml = XmlWriter::pretty_mode(Vec::new());
	xml.dtd("UTF-8")?;
	xml.begin_elem("root")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<root/>");
	Ok(())
}

// ---- elem / elem_text ----------------------------------------------------------

#[test]
fn elem_self_closing() -> Result<(), Error> {
	// elem() calls close_elem(false), so it self-closes the open parent tag
	// and writes the new element at the same level (sibling, not child)
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.elem("br")?;
	xml.elem("hr")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root/><br/><hr/>");
	Ok(())
}

#[test]
fn elem_standalone() -> Result<(), Error> {
	// elem() at top level (no parent) just writes a self-closing element
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.elem("br")?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<br/>");
	Ok(())
}

#[test]
fn elem_with_namespace() -> Result<(), Error> {
	// elem() self-closes the open parent tag and writes the new element with namespace
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.set_namespace("ns");
	xml.begin_elem("root")?;
	xml.elem("child")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<ns:root/><ns:child/>");
	Ok(())
}

#[test]
fn elem_text_inline() -> Result<(), Error> {
	// elem_text() also calls close_elem(false), self-closing the open parent tag
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.elem_text("title", "Hello World")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root/><title>Hello World</title>");
	Ok(())
}

#[test]
fn elem_text_standalone() -> Result<(), Error> {
	// elem_text() at top level writes a complete element with text content
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.elem_text("title", "Hello World")?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<title>Hello World</title>");
	Ok(())
}

#[test]
fn elem_text_escapes_content() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.elem_text("msg", "<b>bold</b> & 'quoted'")?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(
		&res,
		"<msg>&lt;b&gt;bold&lt;/b&gt; &amp; &apos;quoted&apos;</msg>"
	);
	Ok(())
}

#[test]
fn elem_text_with_namespace() -> Result<(), Error> {
	// namespace prefix is applied to opening tag but not the closing tag in elem_text
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.set_namespace("ns");
	xml.elem_text("title", "text")?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<ns:title>text</title>");
	Ok(())
}

// ---- elem_text pretty ----------------------------------------------------------

#[test]
fn elem_text_pretty() -> Result<(), Error> {
	// elem_text() self-closes the open parent; the new element gets indentation
	let mut xml = XmlWriter::pretty_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.elem_text("title", "Hello")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root/>\n  <title>Hello</title>");
	Ok(())
}

// ---- escape characters ---------------------------------------------------------

#[test]
fn text_escapes_ampersand() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.text("a & b")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root>a &amp; b</root>");
	Ok(())
}

#[test]
fn text_escapes_lt_gt() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.text("<value>")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root>&lt;value&gt;</root>");
	Ok(())
}

#[test]
fn attr_esc_escapes_backslash_in_name() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.attr_esc("na\\me", "val")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root na\\\\me=\"val\"/>");
	Ok(())
}

#[test]
fn comment_escapes_special_chars() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.comment("a < b & c > d")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root><!-- a &lt; b &amp; c &gt; d --></root>");
	Ok(())
}

// ---- mode switching ------------------------------------------------------------

#[test]
fn mode_switch_compact_to_pretty() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.set_pretty_mode();
	xml.begin_elem("root")?;
	xml.begin_elem("child")?;
	xml.end_elem()?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root>\n  <child/>\n</root>");
	Ok(())
}

#[test]
fn mode_switch_pretty_to_compact() -> Result<(), Error> {
	let mut xml = XmlWriter::pretty_mode(Vec::new());
	xml.set_compact_mode();
	xml.begin_elem("root")?;
	xml.begin_elem("child")?;
	xml.end_elem()?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root><child/></root>");
	Ok(())
}

// ---- namespace getter ----------------------------------------------------------

#[test]
fn namespace_getter() {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	assert!(xml.namespace().is_none());
	xml.set_namespace("ns");
	assert_eq!(xml.namespace(), Some("ns"));
	xml.unset_namespace();
	assert!(xml.namespace().is_none());
}

// ---- into_inner ----------------------------------------------------------------

#[test]
fn into_inner_returns_buffer() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.end_elem()?;
	let buf = xml.into_inner();
	assert_eq!(&buf, b"<root/>");
	Ok(())
}

// ---- debug impl ----------------------------------------------------------------

#[test]
fn debug_format() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.begin_elem("root")?;
	let s = format!("{xml:?}");
	assert!(s.contains("XmlWriter"));
	assert!(s.contains("root"));
	Ok(())
}

// ---- close on empty stack ------------------------------------------------------

#[test]
fn close_on_empty_stack() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	// calling close with nothing open is a no-op
	xml.close()?;
	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "");
	Ok(())
}

// ---- error cases ---------------------------------------------------------------

#[test]
fn error_attr_without_element() {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	let err = xml.attr("key", "val").unwrap_err();
	assert!(matches!(err, Error::WriteWithoutElement));
}

#[test]
fn error_attr_esc_without_element() {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	let err = xml.attr_esc("key", "val").unwrap_err();
	assert!(matches!(err, Error::WriteWithoutElement));
}

#[test]
fn error_ns_decl_without_element() {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	let nsmap = vec![(None, "http://example.com/")];
	let err = xml.ns_decl(&nsmap).unwrap_err();
	assert!(matches!(err, Error::OpenNamespaceWithoutElement));
}

#[test]
fn error_end_elem_without_open() {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	let err = xml.end_elem().unwrap_err();
	assert!(matches!(err, Error::CloseElement));
}

// ---- nested namespaces ---------------------------------------------------------

#[test]
fn nested_namespace_change() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.set_namespace("a");
	xml.begin_elem("outer")?;
	xml.set_namespace("b");
	xml.begin_elem("inner")?;
	xml.end_elem()?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<a:outer><b:inner/></a:outer>");
	Ok(())
}

// ---- empty_elem without namespace ----------------------------------------------

#[test]
fn empty_elem_no_namespace() -> Result<(), Error> {
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.empty_elem("leaf")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root><leaf/></root>");
	Ok(())
}

// ---- pretty indentation details ------------------------------------------------

#[test]
fn pretty_deep_nesting() -> Result<(), Error> {
	let mut xml = XmlWriter::pretty_mode(Vec::new());
	xml.begin_elem("a")?;
	xml.begin_elem("b")?;
	xml.begin_elem("c")?;
	xml.text("leaf")?;
	xml.end_elem()?;
	xml.end_elem()?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<a>\n  <b>\n    <c>leaf</c>\n  </b>\n</a>");
	Ok(())
}

#[test]
fn pretty_empty_elem_no_namespace() -> Result<(), Error> {
	let mut xml = XmlWriter::pretty_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.empty_elem("br")?;
	xml.end_elem()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root>\n  <br/>\n</root>");
	Ok(())
}

// ---- close() with open elements ------------------------------------------------

#[test]
fn close_with_open_elements() -> Result<(), Error> {
	// close() must auto-close all open elements without explicit end_elem() calls
	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.begin_elem("child")?;
	xml.close()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root><child/></root>");
	Ok(())
}

#[test]
fn close_with_open_elements_pretty() -> Result<(), Error> {
	let mut xml = XmlWriter::pretty_mode(Vec::new());
	xml.begin_elem("root")?;
	xml.begin_elem("child")?;
	xml.begin_elem("leaf")?;
	xml.close()?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<root>\n  <child>\n    <leaf/>\n  </child>\n</root>");
	Ok(())
}

// ---- original integration tests ------------------------------------------------

fn create_xml(
	writer: &mut XmlWriter<'_, Vec<u8>>,
	nsmap: &Vec<(Option<&'static str>, &'static str)>,
) -> Result<(), woxml::Error> {
	writer.begin_elem("OTDS")?;
	writer.ns_decl(nsmap)?;
	writer.comment("nice to see you")?;
	writer.set_namespace("st");
	writer.empty_elem("success")?;
	writer.begin_elem("node")?;
	writer.attr_esc("name", "\"123\"")?;
	writer.attr("id", "abc")?;
	writer.attr("'unescaped'", "\"123\"")?;
	writer.text("'text'")?;
	writer.end_elem()?;
	writer.unset_namespace();
	writer.begin_elem("stuff")?;
	writer.cdata("blablab")?;
	writer.end_elem()?;
	writer.begin_elem("no_children")?;
	writer.end_elem()?;
	writer.end_elem()?;
	writer.close()?;
	writer.flush()
}

#[test]
fn compact() -> Result<(), woxml::Error> {
	let nsmap = vec![
		(None, "http://localhost/"),
		(Some("st"), "http://127.0.0.1/"),
	];
	let mut writer = XmlWriter::compact_mode(Vec::new());

	create_xml(&mut writer, &nsmap)?;

	let xml = String::try_from(writer).unwrap();
	println!("{}", &xml);
	assert_eq!(
		&xml,
		"<OTDS xmlns=\"http://localhost/\" xmlns:st=\"http://127.0.0.1/\"><!-- nice to see you --><st:success/><st:node name=\"&quot;123&quot;\" id=\"abc\" \'unescaped\'=\"\"123\"\">&apos;text&apos;</st:node><stuff><![CDATA[blablab]]></stuff><no_children/></OTDS>"
	);
	Ok(())
}

#[test]
fn pretty() -> Result<(), woxml::Error> {
	let nsmap = vec![
		(None, "http://localhost/"),
		(Some("st"), "http://127.0.0.1/"),
	];
	let mut writer = XmlWriter::pretty_mode(Vec::new());

	create_xml(&mut writer, &nsmap)?;

	let xml = String::try_from(writer).unwrap();
	println!("{}", &xml);
	assert_eq!(
		&xml,
		"<OTDS xmlns=\"http://localhost/\" xmlns:st=\"http://127.0.0.1/\">\n  <!-- nice to see you -->\n  <st:success/>\n  <st:node name=\"&quot;123&quot;\" id=\"abc\" \'unescaped\'=\"\"123\"\">&apos;text&apos;</st:node>\n  <stuff>\n    <![CDATA[blablab]]>\n  </stuff>\n  <no_children/>\n</OTDS>"
	);
	Ok(())
}

#[test]
fn comment() -> Result<(), woxml::Error> {
	let mut xml = XmlWriter::pretty_mode(Vec::new());
	xml.comment("comment")?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<!-- comment -->");

	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.comment("comment")?;

	let res = String::try_from(xml).unwrap();
	assert_eq!(&res, "<!-- comment -->");
	Ok(())
}

#[test]
fn buffer() -> Result<(), woxml::Error> {
	let nsmap = vec![
		(None, "http://localhost/"),
		(Some("st"), "http://127.0.0.1/"),
	];
	let mut writer = XmlWriter::compact_mode(bytes::BytesMut::new());

	writer.begin_elem("OTDS")?;
	writer.ns_decl(&nsmap)?;
	writer.comment("nice to see you")?;
	writer.set_namespace("st");
	writer.empty_elem("success")?;
	writer.begin_elem("node")?;
	writer.attr_esc("name", "\"123\"")?;
	writer.attr("id", "abc")?;
	writer.attr("'unescaped'", "\"123\"")?;
	writer.text("'text'")?;
	writer.end_elem()?;
	writer.unset_namespace();
	writer.begin_elem("stuff")?;
	writer.cdata("blablab")?;
	writer.end_elem()?;
	writer.end_elem()?;
	writer.close()?;
	writer.flush()?;

	let xml = String::try_from(writer).unwrap();
	println!("{}", &xml);
	assert_eq!(
		&xml,
		"<OTDS xmlns=\"http://localhost/\" xmlns:st=\"http://127.0.0.1/\"><!-- nice to see you --><st:success/><st:node name=\"&quot;123&quot;\" id=\"abc\" \'unescaped\'=\"\"123\"\">&apos;text&apos;</st:node><stuff><![CDATA[blablab]]></stuff></OTDS>"
	);
	Ok(())
}
