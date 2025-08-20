// Copyright © 2025 Stephan Kunz
// Copyright © of crate 'xml_writer' Piotr Zolnierek
#![no_std]
#![doc = include_str!("../README.md")]

#[doc(hidden)]
extern crate alloc;

mod error;
mod woxml;
mod write;

pub use error::Error;
pub use woxml::XmlWriter;
pub use write::Write;
