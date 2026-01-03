/// An error that can be ignored with just a warning.
pub trait IgnorableError {
	fn ignore(self);
}

impl<T, E> IgnorableError for std::result::Result<T, E>
where
	E: std::error::Error,
{
	fn ignore(self) {
		if let Err(e) = self {
			log::warn!("ignored error: {e} - {e:?}");
		}
	}
}

