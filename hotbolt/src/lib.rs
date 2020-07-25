use hotbolt_ffi::ENTRY_MAIN;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::Item;

#[proc_macro_attribute]
pub fn hotbolt_entry_main(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let input: Item = syn::parse_macro_input!(token_stream);
	if let Item::Fn(input_function) = &input {
		let input_function_name = &input_function.sig.ident;
		let entry_main_name = format_ident!("{}", ENTRY_MAIN);
		let expanded = quote! {
			#input

			#[cfg(not(feature = "hotbolt_erase"))]
			#[no_mangle]
			pub extern "C" fn #entry_main_name() {
				#input_function_name();
			}
		};

		TokenStream::from(expanded)
	} else {
		panic!("#[hotbolt_entry_main] is intended on a function");
	}
}
