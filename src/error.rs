// Copyright © 2025 Stephan Kunz
//! Woxml's errors.

use thiserror::Error;

/// Things that may go wrong during creation of the XML.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
	/// Closing an elemtent without having one opened.
	#[error("attempted to close 'elem', when none was open")]
	CloseElement,
	/// Closing a namespace without having one opened.
	#[error("attempted to close namespaced 'elem' without corresponding open namespace")]
	CloseNamespace,
	/// Opening a namespace without having an element.
	#[error("attempted to write namespace declaration to 'elem', when no 'elem' was opened")]
	OpenNamespaceWithoutElement,
	/// Attempt to write without having an element opened.
	#[error("attempted to write 'attr' to 'elem', when no 'elem' was opened")]
	WriteWithoutElement,
	/// Writing the buffer failed.
	#[error("failed to write buffer")]
	WriteAllEof,
	/// Conversion of buffer into String failed.
	#[error("failed parsing buffer as UTF8")]
	ParsingUtf8,
}
