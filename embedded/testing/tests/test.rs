#![no_main]
#![no_std]

//! Testing for `woxml::XmlWriter`

extern crate alloc;

use alloc::vec::Vec;
use woxml::XmlWriter;
 
fn create_xml(
    writer: &mut XmlWriter<'_, Vec<u8>>,
    nsmap: &Vec<(Option<&'static str>, &'static str)>,
) {
    let _ = writer.begin_elem("OTDS");
    let _ = writer.ns_decl(nsmap);
    let _ = writer.comment("nice to see you");
    let _ = writer.set_namespace("st");
    let _ = writer.empty_elem("success");
    let _ = writer.begin_elem("node");
    let _ = writer.attr_esc("name", "\"123\"");
    let _ = writer.attr("id", "abc");
    let _ = writer.attr("'unescaped'", "\"123\""); // this WILL generate invalid xml
    let _ = writer.text("'text'");
    let _ = writer.end_elem();
    let _ = writer.unset_namespace();
    let _ = writer.begin_elem("stuff");
    let _ = writer.cdata("blablab");
    let _ = writer.end_elem();
    let _ = writer.end_elem();
    let _ = writer.flush();
}

#[cfg(test)]
#[embedded_test::tests]
mod tests {
    use super::*;
    use alloc::vec;

    use ariel_os::debug::log::*;

    #[test]
    async fn compact() {
        let nsmap = vec![
            (None, "http://localhost/"),
            (Some("st"), "http://127.0.0.1/"),
        ];
        let mut writer = XmlWriter::compact_mode(Vec::new());

        create_xml(&mut writer, &nsmap);

        let actual = writer.into_inner();
        info!("{}", str::from_utf8(&actual).unwrap());
        assert_eq!(
            str::from_utf8(&actual).unwrap(),
            "<OTDS xmlns=\"http://localhost/\" xmlns:st=\"http://127.0.0.1/\"><!-- nice to see you --><st:success/><st:node name=\"&quot;123&quot;\" id=\"abc\" \'unescaped\'=\"\"123\"\">&apos;text&apos;</st:node><stuff><![CDATA[blablab]]></stuff></OTDS>"
        );
    }


    // A test which takes the state returned by the init function (optional)
    #[test]
    async fn pretty() {
        let nsmap = vec![
            (None, "http://localhost/"),
            (Some("st"), "http://127.0.0.1/"),
        ];
        let mut writer = XmlWriter::pretty_mode(Vec::new());

        create_xml(&mut writer, &nsmap);

        let actual = writer.into_inner();
        info!("{}", str::from_utf8(&actual).unwrap());
        assert_eq!(
            str::from_utf8(&actual).unwrap(),
            "<OTDS xmlns=\"http://localhost/\" xmlns:st=\"http://127.0.0.1/\">\n  <!-- nice to see you -->\n  <st:success/>\n  <st:node name=\"&quot;123&quot;\" id=\"abc\" \'unescaped\'=\"\"123\"\">&apos;text&apos;</st:node>\n  <stuff>\n    <![CDATA[blablab]]>\n  </stuff>\n</OTDS>"
        );
    }

    #[test]
    async fn comment() {
        let mut xml = XmlWriter::pretty_mode(Vec::new());
        let _ = xml.comment("comment");

        let actual = xml.into_inner();
        assert_eq!(
            str::from_utf8(&actual).unwrap(),
            "<!-- comment -->"
        );

        let mut xml = XmlWriter::compact_mode(Vec::new());
        let _ = xml.comment("comment");

        let actual = xml.into_inner();
        assert_eq!(
            str::from_utf8(&actual).unwrap(),
            "<!-- comment -->"
        );
    }
}
