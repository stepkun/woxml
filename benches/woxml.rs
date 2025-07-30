// Copyright © 2025 Stephan Kunz
#![allow(missing_docs)]

//! Benchmarks of woxml

#[doc(hidden)]
extern crate alloc;

use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use woxml::XmlWriter;

const SAMPLES: usize = 100;
const ITERATIONS: usize = 100;
const DURATION: Duration = Duration::from_secs(5);

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

fn woxml(c: &mut Criterion) {
    let mut group = c.benchmark_group("woxml");
    group.measurement_time(DURATION).sample_size(SAMPLES);

    let nsmap: Vec<(Option<&'static str>, &'static str)> = vec![
        (None, "http://localhost/"),
        (Some("st"), "http://127.0.0.1/"),
    ];

    let mut writer: XmlWriter<'_, Vec<u8>> = XmlWriter::compact_mode(Vec::new());
    group.bench_function("compact", |b| {
        b.iter(|| {
            for _ in 1..=ITERATIONS {
                create_xml(&mut writer, &nsmap);
            }
            std::hint::black_box(());
        });
    });

    let mut writer: XmlWriter<'_, Vec<u8>> = XmlWriter::pretty_mode(Vec::new());
    group.bench_function("pretty", |b| {
        b.iter(|| {
            for _ in 1..=ITERATIONS {
                create_xml(&mut writer, &nsmap);
            }
            std::hint::black_box(());
        });
    });
}

criterion_group!(benches, woxml);

criterion_main!(benches);
