use std::{marker::PhantomData, str::Utf8Error};

use crate::prelude::UnsafeFrom;

/// Represents a native Rust pointer type.
pub trait Pointer<T: Sized> {
	/// Returns this pointer as a const pointer.
	fn as_const_ptr(&self) -> *const T;

	/// Returns a null pointer.
	fn null() -> Self;
}

impl<T> Pointer<T> for *const T {
	#[inline]
	fn as_const_ptr(&self) -> *const T {
		*self
	}

	#[inline]
	fn null() -> Self {
		std::ptr::null()
	}
}

impl<T> Pointer<T> for *mut T {
	#[inline]
	fn as_const_ptr(&self) -> *const T {
		*self
	}

	#[inline]
	fn null() -> Self {
		std::ptr::null_mut()
	}
}

/// Array passed around using a C FFI.
///
/// This class is generally unsafe to use because it does not deallocate the contained array. As a
/// general rule, you should always use the variant of this class with a lifetime.
///
/// Any instance of this class with a `static` lifetime has forfeited its right to any safety. Any
/// conversion to an instance with this lifetime means you are giving ownership away.
#[repr(C)]
pub struct RawFfiArray<'a, T: Sized, U: Pointer<T>> {
	data: U,
	len: usize,
	capacity: usize,
	phantom: PhantomData<&'a T>,
}

/// Array passed around using a C FFI, used when for borrowing.
///
/// For more information, see [`RawFfiArray`](RawFfiArray).
pub type FfiArray<'a, T> = RawFfiArray<'a, T, *const T>;

/// Array passed around using a C FFI, used when ownership is being transferred.
///
/// For more information, see [`RawFfiArray`](RawFfiArray).
pub type FfiArrayMut<'a, T> = RawFfiArray<'a, T, *mut T>;

impl<'a, T, U: Pointer<T>> RawFfiArray<'a, T, U> {
	/// Returns an empty instance of this object.
	pub fn empty() -> Self {
		Self {
			data: U::null(),
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
		std::slice::from_raw_parts(self.data.as_const_ptr(), self.len)
	}
}

impl<'a, T: Pointer<u8>> RawFfiArray<'a, u8, T> {
	/// Casts the array into a native Rust string without any validations.
	pub unsafe fn as_str_unchecked(&self) -> &str {
		std::str::from_utf8_unchecked(self.as_slice())
	}

	/// Casts the array into a native Rust string, checking whether the string is valid UTF-8.
	pub unsafe fn as_str(&self) -> Result<&str, Utf8Error> {
		std::str::from_utf8(self.as_slice())
	}
}

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
			data: vec.as_ptr(),
			len: vec.len(),
			capacity: vec.capacity(),
			phantom: PhantomData,
		}
	}
}

impl<'a, T> From<&'a [T]> for FfiArray<'a, T> {
	fn from(slice: &[T]) -> Self {
		Self {
			data: slice.as_ptr(),
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

impl<T> UnsafeFrom<Vec<T>> for FfiArrayMut<'static, T> {
	unsafe fn unsafe_from(vec: Vec<T>) -> Self {
		// TODO: https://github.com/rust-lang/rust/issues/65816
		let array = Self {
			data: vec.as_ptr() as *mut T,
			len: vec.len(),
			capacity: vec.capacity(),
			phantom: PhantomData,
		};
		vec.leak();
		array
	}
}

impl<T> UnsafeFrom<Box<T>> for FfiArrayMut<'static, T> {
	unsafe fn unsafe_from(value: Box<T>) -> Self {
		Self {
			data: Box::leak(value),
			len: 1,
			capacity: 1,
			phantom: PhantomData,
		}
	}
}

impl<T> UnsafeFrom<FfiArrayMut<'static, T>> for Vec<T> {
	unsafe fn unsafe_from(array: FfiArrayMut<'static, T>) -> Self {
		Vec::from_raw_parts(array.data, array.len, array.capacity)
	}
}

impl<T> UnsafeFrom<FfiArrayMut<'static, T>> for Box<T> {
	unsafe fn unsafe_from(array: FfiArrayMut<'static, T>) -> Self {
		Box::from_raw(array.data)
	}
}

/// Serializes the given value as an array of bytes.
pub trait Serializer<T> {
	/// Perform the conversion.
	fn serialize(value: &T) -> Result<FfiArrayMut<'static, u8>, ()>;
}

/// Deserializes an array of bytes to the type `T`.
pub trait Deserializer<T> {
	/// Perform the conversion.
	fn deserialize(bytes: &[u8]) -> Result<T, ()>;
}
