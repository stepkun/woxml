// Copyright © 2025 Stephan Kunz
//! Woxml's errors.

/// Shortcut for woxml's Result<T, E> type
pub type Result<T> = core::result::Result<T, Error>;

/// Things that may go wrong during creation of the XML.
#[non_exhaustive]
pub enum Error {
	/// Closing an elemtent without having one opened.
	CloseElement,
	/// Closing a namespace without having one opened.
	CloseNamespace,
	/// Opening a namespace without having an element.
	OpenNamespaceWithoutElement,
	/// Attempt to write without having an element opened.
	WriteWithoutElement,
	/// Writing the buffer failed.
	WriteAllEof,
	/// Conversion of buffer into String failed.
	ParsingUtf8,
}

/// Currently the default implementation is sufficient.
impl core::error::Error for Error {
	// fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
	// 	None
	// }

	// fn cause(&self) -> Option<&dyn core::error::Error> {
	// 	self.source()
	// }

	// fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {}
}

impl core::fmt::Debug for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::CloseElement => write!(f, "CloseElement"),
			Self::CloseNamespace => write!(f, "CloseNamespace"),
			Self::OpenNamespaceWithoutElement => write!(f, "OpenNamespaceWithoutElement"),
			Self::WriteWithoutElement => write!(f, "WriteWithoutElement"),
			Self::WriteAllEof => write!(f, "WriteAllEof"),
			Self::ParsingUtf8 => write!(f, "ConversionToString"),
		}
	}
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::CloseElement => write!(f, "attempted to close 'elem', when none was open"),
			Self::CloseNamespace => write!(f, "attempted to close namespaced 'elem' without corresponding open namespace"),
			Self::OpenNamespaceWithoutElement => write!(
				f,
				"attempted to write namespace declaration to 'elem', when no 'elem' was opened"
			),
			Self::WriteWithoutElement => write!(f, "attempted to write 'attr' to 'elem', when no 'elem' was opened"),
			Self::WriteAllEof => write!(f, "failed to write buffer"),
			Self::ParsingUtf8 => write!(f, "failed parsing buffer as UTF8"),
		}
	}
}
