// Copyright © 2025 Stephan Kunz

//! Testing of woxml library

#[cfg(feature = "std")]
extern crate std;

use std::{println, str, vec, vec::Vec};
use woxml::XmlWriter;

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

	let actual = writer.into_inner();
	println!("{}", str::from_utf8(&actual).expect("should not happen"));
	assert_eq!(
		str::from_utf8(&actual).expect("should not happen"),
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

	let actual = writer.into_inner();
	println!("{}", str::from_utf8(&actual).expect("should not happen"));
	assert_eq!(
		str::from_utf8(&actual).expect("should not happen"),
		"<OTDS xmlns=\"http://localhost/\" xmlns:st=\"http://127.0.0.1/\">\n  <!-- nice to see you -->\n  <st:success/>\n  <st:node name=\"&quot;123&quot;\" id=\"abc\" \'unescaped\'=\"\"123\"\">&apos;text&apos;</st:node>\n  <stuff>\n    <![CDATA[blablab]]>\n  </stuff>\n  <no_children/>\n</OTDS>"
	);
	Ok(())
}

#[test]
fn comment() -> Result<(), woxml::Error> {
	let mut xml = XmlWriter::pretty_mode(Vec::new());
	xml.comment("comment")?;

	let actual = xml.into_inner();
	assert_eq!(str::from_utf8(&actual).expect("should not happen"), "<!-- comment -->");

	let mut xml = XmlWriter::compact_mode(Vec::new());
	xml.comment("comment")?;

	let actual = xml.into_inner();
	assert_eq!(str::from_utf8(&actual).expect("should not happen"), "<!-- comment -->");
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

	let actual = writer.into_inner();
	println!("{}", str::from_utf8(&actual).expect("should not happen"));
	assert_eq!(
		str::from_utf8(&actual).expect("should not happen"),
		"<OTDS xmlns=\"http://localhost/\" xmlns:st=\"http://127.0.0.1/\"><!-- nice to see you --><st:success/><st:node name=\"&quot;123&quot;\" id=\"abc\" \'unescaped\'=\"\"123\"\">&apos;text&apos;</st:node><stuff><![CDATA[blablab]]></stuff></OTDS>"
	);
	Ok(())
}
