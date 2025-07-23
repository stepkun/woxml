// Copyright © 2025 Stephan Kunz
// Copyright © before 2025 Piotr Zolnierek

use std::fmt;
use std::io::{self, Write};

pub type Result = io::Result<()>;

/// The XmlWriter himself
pub struct XmlWriter<'a, W: Write> {
    /// `bool` indicates self closing
    stack: Vec<(&'a str, bool)>,
    /// `bool` indicates self closing
    ns_stack: Vec<Option<&'a str>>,
    writer: Box<W>,
    opened: bool,
    /// If `true` it will indent all opening elements
    pretty: bool,
    /// An XML namespace that all elements will be part of, unless `None`
    namespace: Option<&'a str>,
    /// includes `pretty`, additional:
    /// - puts closing elements into own line
    /// - elements without children are self-closing
    very_pretty: bool,
    /// if `true` current elem has children
    children: bool,
    /// newline indicator
    newline: bool
}

impl<'a, W: Write> fmt::Debug for XmlWriter<'a, W> {
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
    pub fn compact_mode(writer: W) -> XmlWriter<'a, W> {
        XmlWriter {
            stack: Vec::new(),
            ns_stack: Vec::new(),
            writer: Box::new(writer),
            opened: false,
            pretty: false,
            namespace: None,
            very_pretty: false,
            children: false,
            newline: false,
        }
    }

    /// Create a new writer with `pretty` output
    pub fn pretty_mode(writer: W) -> XmlWriter<'a, W> {
        XmlWriter {
            stack: Vec::new(),
            ns_stack: Vec::new(),
            writer: Box::new(writer),
            opened: false,
            pretty: true,
            namespace: None,
            very_pretty: true,
            children: false,
            newline: false,
        }
    }

    /// Switch to `compact` mode
    pub fn set_compact_mode(&mut self) {
        self.pretty = false;
        self.very_pretty = false;
    }

    /// Switch to `pretty` mode
    pub fn set_pretty_mode(&mut self) {
        self.pretty = true;
        self.very_pretty = true;
    }

    /// Set a namespace
    pub fn set_namespace(&mut self, namespace: &'a str) {
        self.namespace = Some(namespace);
    }

    /// Write the DTD
    pub fn dtd(&mut self, encoding: &str) -> Result {
        self.write("<?xml version=\"1.0\" encoding=\"")?;
        self.write(encoding)?;
        self.write("\" ?>\n")
    }

    fn indent(&mut self) -> Result {
        let indent = self.stack.len();
        if self.very_pretty {
            if self.newline {
                self.write("\n")?;
            } else {
                self.newline = true;
            }
            for _ in 0..indent {
                self.write("  ")?;
            }
        } else if self.pretty && !self.stack.is_empty() {
            self.write("\n")?;
            for _ in 0..(indent) {
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

    /// Writes namespace declarations (xmlns:xx) into the currently open element
    pub fn ns_decl(&mut self, ns_map: &Vec<(Option<&'a str>, &'a str)>) -> Result {
        if !self.opened {
            panic!(
                "Attempted to write namespace decl to elem, when no elem was opened, stack {:?}",
                self.stack
            );
        }

        for item in ns_map {
            let name = match item.0 {
                Some(pre) => "xmlns:".to_string() + pre,
                None => "xmlns".to_string(),
            };
            self.attr(&name, item.1)?;
        }
        Ok(())
    }

    /// Write a self-closing element like <br/>
    pub fn elem(&mut self, name: &str) -> Result {
        self.close_elem()?;
        self.indent()?;
        self.write("<")?;
        let ns = self.namespace;
        self.ns_prefix(ns)?;
        self.write(name)?;
        self.write("/>")
    }

    /// Write an element with inlined text (escaped)
    pub fn elem_text(&mut self, name: &str, text: &str) -> Result {
        self.close_elem()?;
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
    pub fn begin_elem(&mut self, name: &'a str) -> Result {
        self.children = true;
        self.close_elem()?;
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
        self.children = false;
        // stderr().write_fmt(format_args!("\nbegin {}", name));
        let ns = self.namespace;
        self.ns_prefix(ns)?;
        self.write(name)
    }

    /// Close an elem if open, do nothing otherwise
    fn close_elem(&mut self) -> Result {
        if self.opened {
            if self.very_pretty && !self.children {
                self.write("/>")?;
            } else {
                self.write(">")?;
            }
            self.opened = false;
        }
        Ok(())
    }

    /// End and elem
    pub fn end_elem(&mut self) -> Result {
        self.close_elem()?;
        let ns = self.ns_stack.pop().unwrap_or_else(
            || panic!("Attempted to close namespaced element without corresponding open namespace, stack {:?}", self.ns_stack)
        );
        match self.stack.pop() {
            Some((name, children)) => {
                if self.very_pretty {
                    // elem without children have been self-closed
                    if !children {
                        return Ok(())
                    }
                    self.indent()?;
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
    pub fn empty_elem(&mut self, name: &'a str) -> Result {
        self.children = true;
        self.close_elem()?;
        // change previous elem to having children
        if let Some(mut previous) = self.stack.pop() {
            previous.1 = true;
            self.stack.push(previous);
        }
        self.children = false;
        self.indent()?;
        self.write("<")?;
        let ns = self.namespace;
        self.ns_prefix(ns)?;
        self.write(name)?;
        self.write("/>")
    }

    /// Write an attr, make sure name and value contain only allowed chars.
    /// For an escaping version use `attr_esc`
    pub fn attr(&mut self, name: &str, value: &str) -> Result {
        if !self.opened {
            panic!(
                "Attempted to write attr to elem, when no elem was opened, stack {:?}",
                self.stack
            );
        }
        self.write(" ")?;
        self.write(name)?;
        self.write("=\"")?;
        self.write(value)?;
        self.write("\"")
    }

    /// Write an attr, make sure name contains only allowed chars
    pub fn attr_esc(&mut self, name: &str, value: &str) -> Result {
        if !self.opened {
            panic!(
                "Attempted to write attr to elem, when no elem was opened, stack {:?}",
                self.stack
            );
        }
        self.write(" ")?;
        self.escape(name, true)?;
        self.write("=\"")?;
        self.escape(value, false)?;
        self.write("\"")
    }

    /// Escape identifiers or text
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
            };
        }
        Ok(())
    }

    /// Write a text, escapes the text automatically
    pub fn text(&mut self, text: &str) -> Result {
        self.children = true;
        self.close_elem()?;
        // change previous elem to having children
        if let Some(mut previous) = self.stack.pop() {
            previous.1 = true;
            self.stack.push(previous);
        }
        self.children = false;
        if self.very_pretty {
            self.indent()?;
        }
        self.escape(text, false)
    }

    /// Raw write, no escaping, no safety net, use at own risk
    pub fn write(&mut self, text: &str) -> Result {
        self.writer.write_all(text.as_bytes())?;
        Ok(())
    }

    /// Raw write, no escaping, no safety net, use at own risk
    fn write_slice(&mut self, slice: &[u8]) -> Result {
        self.writer.write_all(slice)?;
        Ok(())
    }

    /// Write a CDATA
    pub fn cdata(&mut self, cdata: &str) -> Result {
        self.children = true;
        self.close_elem()?;
        // change previous elem to having children
        if let Some(mut previous) = self.stack.pop() {
            previous.1 = true;
            self.stack.push(previous);
        }
        if self.very_pretty {
            self.indent()?;
        }
        self.children = false;
        self.write("<![CDATA[")?;
        self.write(cdata)?;
        self.write("]]>")
    }

    /// Write a comment
    pub fn comment(&mut self, comment: &str) -> Result {
        self.children = true;
        self.close_elem()?;
        // change previous elem to having children
        if let Some(mut previous) = self.stack.pop() {
            previous.1 = true;
            self.stack.push(previous);
        }
        self.indent()?;
        self.children = false;
        self.write("<!-- ")?;
        self.escape(comment, false)?;
        self.write(" -->")
    }

    /// Close all open elems
    pub fn close(&mut self) -> Result {
        for _ in 0..self.stack.len() {
            self.end_elem()?;
        }
        Ok(())
    }

    /// Flush the underlying Writer
    pub fn flush(&mut self) -> Result {
        self.writer.flush()
    }

    /// Consume the XmlWriter and return the inner Writer
    pub fn into_inner(self) -> W {
        *self.writer
    }
}

#[allow(unused_must_use)]
#[cfg(test)]
mod tests {
    use super::XmlWriter;
    use std::str;

    #[test]
    fn compact() {
        let nsmap = vec![
            (None, "http://localhost/"),
            (Some("st"), "http://127.0.0.1/"),
        ];
        let mut xml = XmlWriter::compact_mode(Vec::new());
        xml.begin_elem("OTDS");
        xml.ns_decl(&nsmap);
        xml.comment("nice to see you");
        xml.namespace = Some("st");
        xml.empty_elem("success");
        xml.begin_elem("node");
        xml.attr_esc("name", "\"123\"");
        xml.attr("id", "abc");
        xml.attr("'unescaped'", "\"123\""); // this WILL generate invalid xml
        xml.text("'text'");
        xml.end_elem();
        xml.namespace = None;
        xml.begin_elem("stuff");
        xml.cdata("blablab");
        // xml.end_elem();
        // xml.end_elem();
        xml.close();
        xml.flush();

        let actual = xml.into_inner();
        println!("{}", str::from_utf8(&actual).unwrap());
        assert_eq!(str::from_utf8(&actual).unwrap(), "<OTDS xmlns=\"http://localhost/\" xmlns:st=\"http://127.0.0.1/\"><!-- nice to see you --><st:success/><st:node name=\"&quot;123&quot;\" id=\"abc\" \'unescaped\'=\"\"123\"\">&apos;text&apos;</st:node><stuff><![CDATA[blablab]]></stuff></OTDS>");
    }

    #[test]
    fn pretty() {
        let nsmap = vec![
            (None, "http://localhost/"),
            (Some("st"), "http://127.0.0.1/"),
        ];
        let mut xml = XmlWriter::pretty_mode(Vec::new());
        xml.begin_elem("OTDS");
        xml.ns_decl(&nsmap);
        xml.comment("have a nice day");
        xml.namespace = Some("st");
        xml.empty_elem("success");
        xml.begin_elem("node");
        xml.attr_esc("name", "\"123\"");
        xml.attr("id", "abc");
        xml.attr("'unescaped'", "\"123\""); // this WILL generate invalid xml
        xml.text("'text'");
        xml.end_elem();
        xml.namespace = None;
        xml.begin_elem("stuff");
        xml.cdata("blablab");
        xml.end_elem();
        xml.end_elem();
        xml.close();
        xml.flush();

        let actual = xml.into_inner();
        println!("{}", str::from_utf8(&actual).unwrap());
        assert_eq!(str::from_utf8(&actual).unwrap(), "<OTDS xmlns=\"http://localhost/\" xmlns:st=\"http://127.0.0.1/\">\n  <!-- have a nice day -->\n  <st:success/>\n  <st:node name=\"&quot;123&quot;\" id=\"abc\" \'unescaped\'=\"\"123\"\">\n    &apos;text&apos;\n  </st:node>\n  <stuff>\n    <![CDATA[blablab]]>\n  </stuff>\n</OTDS>");
    }

    #[test]
    fn comment() {
        let mut xml = XmlWriter::pretty_mode(Vec::new());
        xml.comment("comment");

        let actual = xml.into_inner();
        assert_eq!(str::from_utf8(&actual).unwrap(), "<!-- comment -->");

        let mut xml = XmlWriter::compact_mode(Vec::new());
        xml.comment("comment");

        let actual = xml.into_inner();
        assert_eq!(str::from_utf8(&actual).unwrap(), "<!-- comment -->");
    }
}
