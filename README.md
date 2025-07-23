# woxml::XmlWriter
Small and fast XML-Writer

## Usage

```rust
extern crate woxml;
use woxml::XmlWriter;

let mut xml = XmlWriter::pretty_mode(Vec::new()); // supply a Writer, preferably a BufferedWriter
xml.begin_elem("root");
    xml.comment("have a nice day");
    xml.begin_elem("first");
        xml.attr_esc("name", "\"123\"");
        xml.attr("id", "abc");
        xml.text("'text'");
    xml.end_elem();
    xml.begin_elem("stuff");
        xml.cdata("some cdata");
    xml.end_elem();
xml.end_elem();
xml.close(); // This will also close all open elements
xml.flush();

let actual = xml.into_inner();
println!("{}", str::from_utf8(&actual).unwrap())
```

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
