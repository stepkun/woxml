# woxml::XmlWriter
The `XmlWriter` is designed to write xml in an efficient way without any DOM or other intermediate structures.<br/>

The implementation is based on the crate [xml_writer](https://github.com/pzol/xml_writer) by Piotr Zolnierek,
but can also be used in 'no_std' environments (use 'default-features = false').<br/>

It is not an exact drop-in-replacement for xml_writer's XmlWriter as the access to interiors is prohibitet, 
you have to use different constructors and accessors respectively.
It also is not yet possible to use it for all 'std::io::Write' implementors, missing ones can be added in future versions.

It works for:
- Vec<u8>
- bytes::BytesMut

## Usage

```rust
extern crate woxml;
use woxml::XmlWriter;

let mut xml = XmlWriter::pretty_mode(Vec::new()); // supply a woxml::Write implementor
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
    xml.set_namespace("area51");
        xml.comment("in namespace 'area51'");
        xml.elem("first");
    xml.unset_namespace();
xml.end_elem();
xml.close(); // This will also close all open elements
xml.flush();

let actual = xml.into_inner();
println!("{}", str::from_utf8(&actual).unwrap())
```

## License

Licensed under either of
 * Apache License, Version 2.0 [LICENSE-APACHE](LICENSE-APACHE) or [source](http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license [LICENSE-MIT](LICENSE-MIT) or [source](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above,
without any additional terms or conditions.
