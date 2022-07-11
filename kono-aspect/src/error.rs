pub trait AspectError: Error {
	
}

impl<T> AspectError for T where T: Error {}
