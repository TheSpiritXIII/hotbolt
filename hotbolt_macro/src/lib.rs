use hotbolt_ffi::ENTRY_MAIN;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Item};

#[proc_macro_attribute]
pub fn hotbolt_entry_main(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let input: Item = syn::parse_macro_input!(token_stream);
	if let Item::Fn(input_function) = &input {
		let input_function_name = &input_function.sig.ident;
		let entry_main_name = format_ident!("{}", ENTRY_MAIN);

		let input_count = input_function.sig.inputs.len();
		let args: Vec<(Ident, syn::Path)> = vec![
			(
				format_ident!("server"),
				syn::parse_str("hotbolt::internal::FfiServer").unwrap(),
			),
			(
				format_ident!("state"),
				syn::parse_str("hotbolt::internal::SizedCharArray").unwrap(),
			),
		];

		let export_args: Vec<_> = args
			.iter()
			.enumerate()
			.map(|(index, (name, field))| {
				let ident = if index < input_count {
					name.clone()
				} else {
					format_ident!("_{}", name)
				};
				quote! { #ident: #field }
			})
			.collect();

		let method_args: Vec<&Ident> = args
			.iter()
			.take(input_count)
			.map(|(name, _)| name)
			.collect();

		let expanded = quote! {
			#input

			#[cfg(not(feature = "hotbolt_erase"))]
			#[no_mangle]
			pub extern "C" fn #entry_main_name(#(#export_args),*) {
				#input_function_name(#(#method_args),*);
			}
		};

		TokenStream::from(expanded)
	} else {
		panic!("#[hotbolt_entry_main] is intended on a function");
	}
}
