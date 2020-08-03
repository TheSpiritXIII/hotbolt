use hotbolt_ffi::{ENTRY_STATE, ENTRY_MAIN};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Item};


#[proc_macro_attribute]
pub fn hotbolt_entry_state(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let input: Item = syn::parse_macro_input!(token_stream);
	if let Item::Fn(input_function) = &input {
		let input_function_name = &input_function.sig.ident;
		let entry_name = format_ident!("{}", ENTRY_STATE);

		let expanded = quote! {
			#input

			#[cfg(not(feature = "hotbolt_erase"))]
			#[no_mangle]
			pub extern "C" fn #entry_name() -> hotbolt::internal::SizedCharArray {
				hotbolt::internal::SizedCharArray::from_slice(&#input_function_name())
			}
		};

		TokenStream::from(expanded)
	} else {
		panic!("#[hotbolt_entry_main] is intended on a function");
	}
}

#[proc_macro_attribute]
pub fn hotbolt_entry_main(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let input: Item = syn::parse_macro_input!(token_stream);
	if let Item::Fn(input_function) = &input {
		let input_function_name = &input_function.sig.ident;
		let entry_name = format_ident!("{}", ENTRY_MAIN);

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
				#[hotbolt::hotbolt_entry_state]
				fn hotbolt_expanded_entry_state() -> Vec<u8> {
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
		panic!("#[hotbolt_entry_main] is intended on a function");
	}
}
