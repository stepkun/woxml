// Copyright © 2025 Stephan Kunz
// Copyright © of crate 'xml_writer' Piotr Zolnierek
#![no_std]
#![doc = include_str!("../README.md")]

mod woxml;

pub use woxml::{Error, XmlWriter};
