// Copyright © 2025 Stephan Kunz

//! The XmlWriter writes xml in an efficient way, by writing directly to the provided stream,
//! without any DOM or other intermediate structures. It strives to be zero allocation.

#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(unsafe_code)]

mod woxml;

pub use woxml::XmlWriter;
