use hotbolt_ffi::ffi::{
	ENTRY_APP_COMPATIBLE,
	ENTRY_APP_DROP,
	ENTRY_APP_NEW,
	ENTRY_APP_RUN,
	ENTRY_APP_VERSION,
	ENTRY_STATE_DROP,
	ENTRY_STATE_NEW,
	ENTRY_SERVER_VERSION,
};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Item};

// https://stackoverflow.com/questions/38088067/equivalent-of-func-or-function-in-rust
macro_rules! function_name {
	() => {
		{
			fn f() {}
			fn type_name_of<T>(_: T) -> &'static str {
				std::any::type_name::<T>()
			}
			let name = type_name_of(f);
			&name[..name.len() - 3]
		}
	};
}

// TODO: Maybe compile time type assertions for cleaner errors?

fn wrap_method2<T>(name: &str, token_stream: TokenStream, function_fn: T) -> TokenStream
where
	T: Fn(&Ident) -> proc_macro2::TokenStream,
{
	let input: Item = syn::parse_macro_input!(token_stream);
	let input_function = match &input {
		Item::Fn(item) => item,
		_ => panic!("#[{}] is intended on a function", name),
	};

	let function = function_fn(&input_function.sig.ident);
	let expanded = quote! {
		#input

		#[cfg(not(feature = "hotbolt_erase"))]
		#[no_mangle]
		pub extern "C" #function
	};

	TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn hotbolt_trait_state(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let input: Item = syn::parse_macro_input!(token_stream);
	if let Item::Struct(input_struct) = &input {
		let name = &input_struct.ident;
		let expanded = quote! {
			type HotboltAutoState: hotbolt::ffi::FfiState = #name;

			#[hotbolt::hotbolt_entry_state_new]
			fn hotbolt_auto_state_new(serialized: hotbolt::ffi::FfiArray<'static, u8>) -> *mut c_void {
				HotboltAutoState::state_new(serialized)
			}

			#[hotbolt::hotbolt_entry_state_drop]
			fn hotbolt_auto_state_drop(state_ptr: *mut c_void) {
				HotboltAutoState::state_drop(state_ptr)
			}

			#[hotbolt::hotbolt_entry_state_serialize_new]
			fn hotbolt_auto_state_serialized(state_ptr: *const c_void) -> hotbolt::ffi::FfiArrayMut<'static, u8> {
				HotboltAutoState::state_serialize_new(state_ptr)
			}

			#[hotbolt::hotbolt_entry_state_serialize_drop]
			fn hotbolt_auto_state_serialized(serialized: hotbolt::ffi::FfiArrayMut<'static, u8>) {
				HotboltAutoState::state_serialize_drop(serialized)
			}
		};

		TokenStream::from(expanded)
	} else {
		panic!("#[hotbolt_trait_state] is intended on a struct");
	}
}

#[proc_macro_attribute]
pub fn hotbolt_trait_entry(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let input: Item = syn::parse_macro_input!(token_stream);
	if let Item::Struct(input_struct) = &input {
		let name = &input_struct.ident;
		let expanded = quote! {
			type HotboltAutoEntry: hotbolt::ffi::FfiEntry = #name;

			#[hotbolt::hotbolt_entry_run]
			fn hotbolt_auto_run(app_ptr: *mut c_void, server: FfiServer, state_ptr: *mut c_void) {
				HotboltAutoEntry::run(app_ptr, server, state_ptr)
			}
		};

		TokenStream::from(expanded)
	} else {
		panic!("#[{}] is intended on a struct", function_name!());
	}
}

#[proc_macro_attribute]
pub fn hotbolt_entry_run(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let run_method: proc_macro2::TokenStream =
		wrap_method2(function_name!(), token_stream, |ident| {
			let name = format_ident!("{}", ENTRY_APP_RUN);
			quote! {
				fn #name(app_ptr: *mut c_void, server: FfiServer, state_ptr: *mut c_void) {
					#ident(app_ptr, server, state_ptr)
				}
			}
		})
		.into();
	let version_method = hotbolt_version();
	let wrapper = quote! {
		#run_method

		#version_method
	};
	TokenStream::from(wrapper)
}

#[proc_macro_attribute]
pub fn hotbolt_entry_state_new(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method2(function_name!(), token_stream, |ident| {
		let name = format_ident!("{}", ENTRY_STATE_NEW);
		quote! {
			fn #name(serialized: hotbolt::ffi::FfiArray<'static, u8>) -> *mut std::ffi::c_void {
				#ident(serialized)
			}
		}
	})
}

#[proc_macro_attribute]
pub fn hotbolt_entry_state_drop(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method2(function_name!(), token_stream, |ident| {
		let name = format_ident!("{}", ENTRY_STATE_DROP);
		quote! {
			fn #name(state: *mut std::ffi::c_void) {
				#ident(state)
			}
		}
	})
}

#[proc_macro_attribute]
pub fn hotbolt_entry_state_serialize_new(
	_attr: TokenStream,
	token_stream: TokenStream,
) -> TokenStream {
	wrap_method2(function_name!(), token_stream, |ident| {
		let name = format_ident!("{}", ENTRY_STATE_NEW);
		quote! {
			fn #name(state_ptr: *const c_void) -> hotbolt::ffi::FfiArrayMut<'static, u8> {
				&#ident(state_ptr)
			}
		}
	})
}

#[proc_macro_attribute]
pub fn hotbolt_entry_state_serialize_drop(
	_attr: TokenStream,
	token_stream: TokenStream,
) -> TokenStream {
	wrap_method2(function_name!(), token_stream, |ident| {
		let name = format_ident!("{}", ENTRY_STATE_NEW);
		quote! {
			fn #name(serialized: FfiArrayMut<'static, u8>) {
				#ident(serialized)
			}
		}
	})
}

#[proc_macro_attribute]
pub fn hotbolt_trait_app(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let input: Item = syn::parse_macro_input!(token_stream);
	if let Item::Struct(input_struct) = &input {
		let name = &input_struct.ident;
		let expanded = quote! {
			type HotboltAutoApp: hotbolt::ffi::FfiApp = #name;

			#[hotbolt::hotbolt_entry_app_new]
			fn hotbolt_auto_app_new() -> *mut c_void {
				HotboltAutoApp::app_new()
			}

			#[hotbolt::hotbolt_entry_app_drop]
			fn hotbolt_auto_app_drop(app_ptr: *mut c_void) {
				HotboltAutoApp::app_drop(app_ptr)
			}
		};

		TokenStream::from(expanded)
	} else {
		panic!("#[{}] is intended on a struct", function_name!(),);
	}
}

#[proc_macro_attribute]
pub fn hotbolt_entry_app_new(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method2(function_name!(), token_stream, |ident| {
		let name = format_ident!("{}", ENTRY_APP_NEW);
		quote! {
			fn #name() -> *mut c_void {
				#ident()
			}
		}
	})
}

#[proc_macro_attribute]
pub fn hotbolt_entry_app_drop(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method2(function_name!(), token_stream, |ident| {
		let name = format_ident!("{}", ENTRY_APP_DROP);
		quote! {
			fn #name(app_ptr: *mut c_void) {
				#ident(app_ptr)
			}
		}
	})
}

#[proc_macro_attribute]
pub fn hotbolt_trait_app_version(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let input: Item = syn::parse_macro_input!(token_stream);
	if let Item::Struct(input_struct) = &input {
		let name = &input_struct.ident;
		let expanded = quote! {
			type HotboltAutoAppVersion: hotbolt::ffi::FfiAppVersion = #name;

			#[hotbolt::hotbolt_entry_app_version]
			fn hotbolt_auto_app_version() -> hotbolt::ffi::FfiArray<'static, u8> {
				HotboltAutoAppVersion::app_version()
			}

			#[hotbolt::hotbolt_entry_app_compatible]
			fn hotbolt_auto_app_compatible(other: hotbolt::ffi::FfiArray<'static, u8>) -> bool {
				HotboltAutoAppVersion::app_compatible(other)
			}
		};

		TokenStream::from(expanded)
	} else {
		panic!("#[{}] is intended on a struct", function_name!());
	}
}

#[proc_macro_attribute]
pub fn hotbolt_entry_app_version(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method2(function_name!(), token_stream, |ident| {
		let name = format_ident!("{}", ENTRY_APP_VERSION);
		quote! {
			fn #name() -> hotbolt::ffi::FfiArray<'static, u8> {
				#ident()
			}
		}
	})
}

#[proc_macro_attribute]
pub fn hotbolt_entry_app_compatible(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method2(function_name!(), token_stream, |ident| {
		let name = format_ident!("{}", ENTRY_APP_COMPATIBLE);
		quote! {
			fn #name(other: hotbolt::ffi::FfiArray<u8>) -> bool {
				#ident(other)
			}
		}
	})
}

// TODO: What to do if the main entry is named main (not allowed to our input)?
// TODO: Clippy warning: recursing into entrypoint `main`
/// Deprecated.
#[proc_macro_attribute]
pub fn hotbolt_entry_main(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let input: Item = syn::parse_macro_input!(token_stream);
	if let Item::Fn(input_function) = &input {
		let input_function_name = &input_function.sig.ident;
		let entry_name = format_ident!("{}", ENTRY_APP_RUN);

		let input_count = input_function.sig.inputs.len();
		let args: Vec<(Ident, syn::Path, _)> = vec![
			(
				format_ident!("server"),
				syn::parse_str("hotbolt::internal::FfiServer").unwrap(),
				quote! { server },
			),
			(
				format_ident!("state"),
				syn::parse_str("hotbolt::internal::SizedCharArray").unwrap(),
				quote! { state.as_u8_slice() },
			),
		];

		let export_args: Vec<_> = args
			.iter()
			.enumerate()
			.map(|(index, (name, field, _))| {
				let ident = if index < input_count {
					name.clone()
				} else {
					format_ident!("_{}", name)
				};
				quote! { #ident: #field }
			})
			.collect();

		let method_args: Vec<_> = args
			.iter()
			.take(input_count)
			.map(|(_, _, value)| value)
			.collect();

		let state_entry = if input_count < 2 {
			quote! {
				#[hotbolt::hotbolt_entry_state_get]
				fn hotbolt_expanded_entry() -> Vec<u8> {
					Vec::new()
				}
			}
		} else {
			quote! {}
		};

		let expanded = quote! {
			#input

			#[cfg(not(feature = "hotbolt_erase"))]
			#[no_mangle]
			pub extern "C" fn #entry_name(#(#export_args),*) {
				#input_function_name(#(#method_args),*);
			}
			#state_entry
		};

		TokenStream::from(expanded)
	} else {
		panic!("#[{}] is intended on a function", function_name!(),);
	}
}

/// Deprecated.
#[proc_macro_attribute]
pub fn hotbolt_entry_state_get(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method2(function_name!(), token_stream, |ident| {
		let name = format_ident!("{}", ENTRY_STATE_NEW);
		quote! {
			fn #name() -> hotbolt::internal::SizedCharArray {
				hotbolt::internal::SizedCharArray::from_slice(&#ident())
			}
		}
	})
}

fn hotbolt_version() -> proc_macro2::TokenStream {
	let ident = format_ident!("{}", ENTRY_SERVER_VERSION);
	quote! {
		fn #ident() -> u8 {
			hotbolt::SERVER_VERSION
		}
	}
}
