// Copyright © 2025 Stephan Kunz
// Copyright © before 2025 Piotr Zolnierek

use std::fmt;
use std::io::{self, Write};

pub type Result = io::Result<()>;

/// The `XmlWriter` himself.
#[allow(clippy::struct_excessive_bools)]
pub struct XmlWriter<'a, W: Write> {
    /// element stack
    /// `bool` indicates element having children
    stack: Vec<(&'a str, bool)>,
    /// namespace stack
    ns_stack: Vec<Option<&'a str>>,
    writer: Box<W>,
    /// An XML namespace that all elements will be part of, unless `None`
    namespace: Option<&'a str>,
    /// If `true` it will
    /// - indent all opening elements
    /// - put closing elements into own line
    /// - elements without children are self-closing
    pretty: bool,
    /// if `true` current elem is open
    opened: bool,
    /// newline indicator
    newline: bool,
    /// if `true` current elem has only text content
    text_content: bool,
}

impl<W: Write> fmt::Debug for XmlWriter<'_, W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(write!(
            f,
            "XmlWriter {{ stack: {:?}, opened: {} }}",
            self.stack, self.opened
        )?)
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
            text_content: false,
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
            text_content: false,
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
    pub fn dtd(&mut self, encoding: &str) -> Result {
        self.write("<?xml version=\"1.0\" encoding=\"")?;
        self.write(encoding)?;
        self.write("\" ?>\n")
    }

    fn indent(&mut self) -> Result {
        let indent = self.stack.len();
        if self.pretty {
            if self.newline {
                self.write("\n")?;
            } else {
                self.newline = true;
            }
            for _ in 0..indent {
                self.write("  ")?;
            }
        }
        Ok(())
    }

    /// Write a namespace prefix for the current element,
    /// if there is one set
    fn ns_prefix(&mut self, namespace: Option<&'a str>) -> Result {
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
    pub fn ns_decl(&mut self, ns_map: &Vec<(Option<&'a str>, &'a str)>) -> Result {
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
    pub fn elem(&mut self, name: &str) -> Result {
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
    pub fn elem_text(&mut self, name: &str, text: &str) -> Result {
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
    pub fn begin_elem(&mut self, name: &'a str) -> Result {
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
    fn close_elem(&mut self, has_children: bool) -> Result {
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
    pub fn end_elem(&mut self) -> Result {
        self.close_elem(false)?;
        let ns = self.ns_stack.pop().unwrap_or_else(
            || panic!("Attempted to close namespaced element without corresponding open namespace, stack {:?}", self.ns_stack)
        );
        match self.stack.pop() {
            Some((name, children)) => {
                if self.pretty {
                    // elem without children have been self-closed
                    if !children {
                        return Ok(());
                    }
                    if !self.text_content {
                        self.indent()?;
                    }
                    self.text_content = false;
                }
                self.write("</")?;
                self.ns_prefix(ns)?;
                self.write(name)?;
                self.write(">")?;
                Ok(())
            }
            None => panic!(
                "Attempted to close an elem, when none was open, stack {:?}",
                self.stack
            ),
        }
    }

    /// Begin an empty elem
    /// # Errors
    /// - if writing to buffer fails
    pub fn empty_elem(&mut self, name: &'a str) -> Result {
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
    pub fn attr(&mut self, name: &str, value: &str) -> Result {
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
    pub fn attr_esc(&mut self, name: &str, value: &str) -> Result {
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
    fn escape(&mut self, text: &str, ident: bool) -> Result {
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
    pub fn text(&mut self, text: &str) -> Result {
        self.close_elem(true)?;
        // change previous elem to having children
        if let Some(mut previous) = self.stack.pop() {
            previous.1 = true;
            self.stack.push(previous);
        }
        self.text_content = true;
        self.escape(text, false)
    }

    /// Raw write, no escaping, no safety net, use at own risk
    /// # Errors
    /// - if writing to buffer fails
    pub fn write(&mut self, text: &str) -> Result {
        self.writer.write_all(text.as_bytes())?;
        Ok(())
    }

    /// Raw write, no escaping, no safety net, use at own risk
    /// # Errors
    /// - if writing to buffer fails
    fn write_slice(&mut self, slice: &[u8]) -> Result {
        self.writer.write_all(slice)?;
        Ok(())
    }

    /// Write a CDATA.
    /// # Errors
    /// - if writing to buffer fails
    pub fn cdata(&mut self, cdata: &str) -> Result {
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
    pub fn comment(&mut self, comment: &str) -> Result {
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
    pub fn close(&mut self) -> Result {
        for _ in 0..self.stack.len() {
            self.end_elem()?;
        }
        Ok(())
    }

    /// Flush the underlying Writer
    /// # Errors
    /// - if writing to buffer fails
    pub fn flush(&mut self) -> Result {
        self.writer.flush()
    }

    /// Consume the `XmlWriter` and return the inner Writer
    #[must_use]
    pub fn into_inner(self) -> W {
        *self.writer
    }
}

#[allow(unused_must_use)]
#[cfg(test)]
mod tests {
    use super::XmlWriter;
    use std::str;

    fn create_xml(
        writer: &mut XmlWriter<'_, Vec<u8>>,
        nsmap: &Vec<(Option<&'static str>, &'static str)>,
    ) {
        writer.begin_elem("OTDS");
        writer.ns_decl(nsmap);
        writer.comment("nice to see you");
        writer.namespace = Some("st");
        writer.empty_elem("success");
        writer.begin_elem("node");
        writer.attr_esc("name", "\"123\"");
        writer.attr("id", "abc");
        writer.attr("'unescaped'", "\"123\""); // this WILL generate invalid xml
        writer.text("'text'");
        writer.end_elem();
        writer.namespace = None;
        writer.begin_elem("stuff");
        writer.cdata("blablab");
        // xml.end_elem();
        // xml.end_elem();
        writer.close();
        writer.flush();
    }

    #[test]
    fn compact() {
        let nsmap = vec![
            (None, "http://localhost/"),
            (Some("st"), "http://127.0.0.1/"),
        ];
        let mut writer = XmlWriter::compact_mode(Vec::new());

        create_xml(&mut writer, &nsmap);

        let actual = writer.into_inner();
        println!("{}", str::from_utf8(&actual).expect("should not happen"));
        assert_eq!(
            str::from_utf8(&actual).expect("should not happen"),
            "<OTDS xmlns=\"http://localhost/\" xmlns:st=\"http://127.0.0.1/\"><!-- nice to see you --><st:success/><st:node name=\"&quot;123&quot;\" id=\"abc\" \'unescaped\'=\"\"123\"\">&apos;text&apos;</st:node><stuff><![CDATA[blablab]]></stuff></OTDS>"
        );
    }

    #[test]
    fn pretty() {
        let nsmap = vec![
            (None, "http://localhost/"),
            (Some("st"), "http://127.0.0.1/"),
        ];
        let mut writer = XmlWriter::pretty_mode(Vec::new());

        create_xml(&mut writer, &nsmap);

        let actual = writer.into_inner();
        println!("{}", str::from_utf8(&actual).expect("should not happen"));
        assert_eq!(
            str::from_utf8(&actual).expect("should not happen"),
            "<OTDS xmlns=\"http://localhost/\" xmlns:st=\"http://127.0.0.1/\">\n  <!-- nice to see you -->\n  <st:success/>\n  <st:node name=\"&quot;123&quot;\" id=\"abc\" \'unescaped\'=\"\"123\"\">&apos;text&apos;</st:node>\n  <stuff>\n    <![CDATA[blablab]]>\n  </stuff>\n</OTDS>"
        );
    }

    #[test]
    fn comment() {
        let mut xml = XmlWriter::pretty_mode(Vec::new());
        xml.comment("comment");

        let actual = xml.into_inner();
        assert_eq!(
            str::from_utf8(&actual).expect("should not happen"),
            "<!-- comment -->"
        );

        let mut xml = XmlWriter::compact_mode(Vec::new());
        xml.comment("comment");

        let actual = xml.into_inner();
        assert_eq!(
            str::from_utf8(&actual).expect("should not happen"),
            "<!-- comment -->"
        );
    }
}
