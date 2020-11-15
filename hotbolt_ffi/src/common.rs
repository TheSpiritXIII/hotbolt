use std::{marker::PhantomData, str::Utf8Error};

use crate::prelude::UnsafeFrom;

/// Array passed around using a C FFI.
///
/// This class is generally unsafe to use because it does not deallocate the contained array. As a
/// general rule, you should always use the variant of this class with a lifetime.
///
/// Any instance of this class with a `static` lifetime has forfeited its right to any safety. Any
/// conversion to an instance with this lifetime means you are giving ownership away.
#[repr(C)]
pub struct FfiArray<'a, T> {
	data: *const T,
	len: usize,
	capacity: usize,
	phantom: PhantomData<&'a T>,
}

impl<'a, T> FfiArray<'a, T> {
	/// Returns an empty instance of this object.
	pub fn empty() -> Self {
		Self {
			data: std::ptr::null(),
			len: 0,
			capacity: 0,
			phantom: PhantomData,
		}
	}

	/// Returns true if the length is 0.
	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	/// Casts the array into a native Rust [slice](std::slice).
	pub unsafe fn as_slice(&self) -> &[T] {
		std::slice::from_raw_parts(self.data, self.len)
	}
}

impl<'a> FfiArray<'a, u8> {
	/// Casts the array into a native Rust string without any validations.
	pub unsafe fn as_str_unchecked(&self) -> &str {
		std::str::from_utf8_unchecked(self.as_slice())
	}

	/// Casts the array into a native Rust string, checking whether the string is valid UTF-8.
	pub unsafe fn as_str(&self) -> Result<&str, Utf8Error> {
		std::str::from_utf8(self.as_slice())
	}
}

// # From X to FfiArray.

impl<'a> From<&str> for FfiArray<'a, u8> {
	fn from(slice: &str) -> Self {
		Self {
			data: slice.as_ptr(),
			len: slice.len(),
			capacity: slice.len(),
			phantom: PhantomData,
		}
	}
}

impl<'a, T> From<&'a Vec<T>> for FfiArray<'a, T> {
	fn from(vec: &Vec<T>) -> Self {
		Self {
			data: vec.as_ptr() as *const T,
			len: vec.len(),
			capacity: vec.capacity(),
			phantom: PhantomData,
		}
	}
}

impl<'a, T> From<&'a [T]> for FfiArray<'a, T> {
	fn from(slice: &[T]) -> Self {
		Self {
			data: slice.as_ptr() as *const T,
			len: slice.len(),
			capacity: slice.len(),
			phantom: PhantomData,
		}
	}
}

impl<'a, T> From<&'a T> for FfiArray<'a, T> {
	fn from(value: &T) -> Self {
		Self {
			data: value as *const T,
			len: 1,
			capacity: 1,
			phantom: PhantomData,
		}
	}
}

impl<T> UnsafeFrom<Vec<T>> for FfiArray<'static, T> {
	unsafe fn unsafe_from(vec: Vec<T>) -> Self {
		// TODO: https://github.com/rust-lang/rust/issues/65816
		let array = Self {
			data: vec.as_ptr() as *const T,
			len: vec.len(),
			capacity: vec.capacity(),
			phantom: PhantomData,
		};
		vec.leak();
		array
	}
}

impl<T> UnsafeFrom<Box<T>> for FfiArray<'static, T> {
	unsafe fn unsafe_from(value: Box<T>) -> Self {
		Self {
			data: Box::leak(value) as *const T,
			len: 1,
			capacity: 1,
			phantom: PhantomData,
		}
	}
}

/// Serializes the given value as an array of bytes.
pub trait Serializer<T> {
	/// Perform the conversion.
	fn serialize(value: &T) -> Result<FfiArray<'static, u8>, ()>;
}

/// Deserializes an array of bytes to the type `T`.
pub trait Deserializer<T> {
	/// Perform the conversion.
	fn deserialize(bytes: &[u8]) -> Result<T, ()>;
}
