#![no_main]
#![no_std]

//! Benchmarks for `woxml::XmlWriter`

extern crate alloc;

use alloc::vec::Vec;
use alloc::vec;

use ariel_os::debug::{ExitCode, exit, log::*};
use woxml::XmlWriter;

fn create_xml(
    writer: &mut XmlWriter<'_, Vec<u8>>,
    nsmap: &Vec<(Option<&'static str>, &'static str)>,
) {
    _ = writer.begin_elem("OTDS");
    _ = writer.ns_decl(nsmap);
    _ = writer.comment("have a nice day");
    writer.set_namespace("st");
    _ = writer.empty_elem("success");
    _ = writer.begin_elem("node");
    _ = writer.attr_esc("name", "\"123\"");
    _ = writer.attr("id", "abc");
    _ = writer.attr("'unescaped'", "\"123\""); // this WILL generate invalid xml
    _ = writer.text("'text'");
    _ = writer.end_elem();
    writer.unset_namespace();
    _ = writer.comment("comment");
    _ = writer.begin_elem("stuff");
    _ = writer.cdata("blablab");
    _ = writer.end_elem();
    _ = writer.end_elem();
    _ = writer.close();
    _ = writer.flush();
}

#[ariel_os::thread(autostart)]
fn main() {
    info!("starting benchmark");
    let nsmap: Vec<(Option<&'static str>, &'static str)> = vec![
        (None, "http://localhost/"),
        (Some("st"), "http://127.0.0.1/"),
    ];

    match ariel_os::bench::benchmark(100, || {
        let mut xml = XmlWriter::compact_mode(Vec::new());
        create_xml(&mut xml, &nsmap);
    }) {
        Ok(ticks) => {
            info!("compact mode took {} per iteration", ticks);
        }
        Err(err) => {
            error!("benchmark returned: {}", err);
            exit(ExitCode::FAILURE);
        }
    }

    match ariel_os::bench::benchmark(100, || {
        let mut xml = XmlWriter::pretty_mode(Vec::new());
        create_xml(&mut xml, &nsmap);
    }) {
        Ok(ticks) => {
            info!("pretty mode took {} per iteration", ticks);
        }
        Err(err) => {
            error!("benchmark returned: {}", err);
            exit(ExitCode::FAILURE);
        }
    }
    info!("benchmark finished");

    exit(ExitCode::SUCCESS);
}
