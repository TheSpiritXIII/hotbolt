use std::convert::Infallible;

// TODO: Really wish Rust provided this. RFC?

/// Unsafe variant of [`From`](std::convert::From).
pub trait UnsafeFrom<T>: Sized {
	/// Performs the conversion.
	unsafe fn unsafe_from(t: T) -> Self;
}

// Reflexivity.
impl<T> UnsafeFrom<T> for T {
	unsafe fn unsafe_from(t: T) -> T {
		t
	}
}

/// Unsafe variant of [`Into`](std::convert::Into).
pub trait UnsafeInto<T>: Sized {
	/// Performs the conversion.
	unsafe fn unsafe_into(self) -> T;
}

// Reflexivity.
impl<T, U> UnsafeInto<U> for T
where
	U: UnsafeFrom<T>,
{
	unsafe fn unsafe_into(self) -> U {
		U::unsafe_from(self)
	}
}

/// Unsafe variant of [`TryFrom`](std::convert::TryFrom).
pub trait UnsafeTryFrom<T>: Sized {
	/// The type returned in the event of a conversion error.
	type Error;

	/// Performs the conversion.
	unsafe fn unsafe_try_from(value: T) -> Result<Self, Self::Error>;
}

/// Unsafe variant of [`TryInto`](std::convert::TryInto).
pub trait UnsafeTryInto<T>: Sized {
	/// The type returned in the event of a conversion error.
	type Error;

	/// Performs the conversion.
	unsafe fn unsafe_try_into(self) -> Result<T, Self::Error>;
}

impl<T, U> UnsafeTryInto<U> for T
where
	U: UnsafeTryFrom<T>,
{
	type Error = U::Error;

	unsafe fn unsafe_try_into(self) -> Result<U, U::Error> {
		U::unsafe_try_from(self)
	}
}

impl<T, U> UnsafeTryFrom<U> for T
where
	U: UnsafeInto<T>,
{
	type Error = Infallible;

	unsafe fn unsafe_try_from(value: U) -> Result<Self, Self::Error> {
		Ok(U::unsafe_into(value))
	}
}
