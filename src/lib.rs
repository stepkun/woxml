// Copyright © 2025 Stephan Kunz
// Copyright © of crate 'xml_writer' Piotr Zolnierek
//#![no_std]

//! `XmlWriter` writes xml in an efficient way without any DOM or other intermediate structures.
//! 
//! The implementation is based on the crate xml_writer by Piotr Zolnierek.

#![doc = include_str!("../README.md")]

mod woxml;

pub use woxml::XmlWriter;
