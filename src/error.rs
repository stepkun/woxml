// Copyright © 2025 Stephan Kunz

pub use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

/// Error type
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
	/// Pass through `core::str::Utf8Error`
	#[error("{0}")]
	Utf8(#[from] core::str::Utf8Error),
	/// Pass through `core::fmt::Error`
	#[error("{0}")]
	Core(#[from] core::fmt::Error),
	/// failed to close an element
	#[error("attempted to close an elem, when none was open")]
	CloseElement,
	/// failed to close a namespace
	#[error("attempted to close namespaced element without corresponding open namespace")]
	CloseNamespace,
	/// tried to open a namespace without an element
	#[error("attempted to write namespace decl to elem, when no elem was opened")]
	OpenNamespaceWithoutElement,
	/// tried to write without an element
	#[error("attempted to write attr to elem, when no elem was opened")]
	WriteWithoutElement,
	/// failed to write whole buffer
	#[error("failed to write whole buffer")]
	WriteAllEof,
}
