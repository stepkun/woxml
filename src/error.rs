// Copyright © 2025 Stephan Kunz

pub use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

/// Error type
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
	/// error close an element
	#[error("attempted to close 'elem', when none was open")]
	CloseElement,
	/// error close a namespace
	#[error("attempted to close namespaced 'elem' without corresponding open namespace")]
	CloseNamespace,
	/// error open a namespace
	#[error("attempted to write namespace declaration to 'elem', when no 'elem' was opened")]
	OpenNamespaceWithoutElement,
	/// error write an element
	#[error("attempted to write 'attr' to 'elem', when no 'elem' was opened")]
	WriteWithoutElement,
	/// error write buffer
	#[error("failed to write whole buffer")]
	WriteAllEof,
}
