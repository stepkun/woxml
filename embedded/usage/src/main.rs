#![no_main]
#![no_std]

//! Usage example for `woxml::XmlWriter`

extern crate alloc;

use alloc::vec::Vec;

use ariel_os::debug::{ExitCode, exit, log::*};
use woxml::XmlWriter;

#[ariel_os::task(autostart)]
async fn main() {
    let mut xml = XmlWriter::pretty_mode(Vec::new());
        let _ = xml.begin_elem("root");
        let _ = xml.comment("have a nice day");
        let _ = xml.begin_elem("first");
        let _ = xml.attr_esc("name", "\"123\"");
        let _ = xml.attr("id", "abc");
        let _ = xml.text("'text'");
        let _ = xml.end_elem();
        let _ = xml.begin_elem("stuff");
        let _ = xml.cdata("some cdata");
        let _ = xml.end_elem();
        let _ = xml.set_namespace("area51");
        let _ = xml.comment("in namespace 'area51'");
        let _ = xml.elem("first");
        let _ = xml.unset_namespace();
        let _ = xml.end_elem();
        let _ = xml.close(); // This will also close all open elements
        let _ = xml.flush();
        let actual = xml.into_inner();

        info!("{}\n", str::from_utf8(&actual).unwrap());

        exit(ExitCode::SUCCESS);
}
