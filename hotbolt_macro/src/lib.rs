use hotbolt_ffi::{
	ENTRY_CLIENT_COMPATIBLE,
	ENTRY_CLIENT_DROP,
	ENTRY_CLIENT_NEW,
	ENTRY_CLIENT_VERSION,
	ENTRY_RUN,
	ENTRY_STATE_DROP,
	ENTRY_STATE_GET,
	ENTRY_STATE_NEW,
};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Item, Signature};

#[proc_macro_attribute]
pub fn hotbolt_main(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let input: Item = syn::parse_macro_input!(token_stream);
	let client_type = match &input {
		Item::Struct(item) => item,
		_ => panic!("#[{}] is intended on a struct", "hotbolt_main"),
	};

	let client_name = &client_type.ident;

	let expanded = quote! {
		type HotboltEntryClient: hotbolt::Client = #client_name;

		#[hotbolt::hotbolt_entry_compatible]
		fn hotbolt_auto_compatible() {
			todo!();
		}

		#[hotbolt::hotbolt_entry_init]
		fn hotbolt_auto_init() {
			todo!();
		}

		#[hotbolt::hotbolt_entry_main]
		fn hotbolt_auto_main(server: impl Server, state: &[u8]) {
			todo!();
		}

		#[hotbolt::hotbolt_entry_destroy]
		fn hotbolt_auto_destroy() {
			todo!();
		}

		#[hotbolt::hotbolt_entry_state_get]
		fn hotbolt_auto_state() -> Vec<u8> {
			todo!();
		}

		#[hotbolt::hotbolt_entry_version]
		fn hotbolt_auto_version() -> Vec<u8> {
			todo!();
		}
	};

	TokenStream::from(expanded)
}

fn wrap_method<TSignature, TBody>(
	name: &str,
	token_stream: TokenStream,
	signature_fn: TSignature,
	body_fn: TBody,
) -> TokenStream
where
	TSignature: Fn() -> proc_macro2::TokenStream,
	TBody: Fn(&Signature) -> proc_macro2::TokenStream,
{
	let input: Item = syn::parse_macro_input!(token_stream);
	let input_function = match &input {
		Item::Fn(item) => item,
		_ => panic!("#[{}] is intended on a function", name),
	};

	let sig = signature_fn();
	let body = body_fn(&input_function.sig);
	let expanded = quote! {
		#input

		#[cfg(not(feature = "hotbolt_erase"))]
		#[no_mangle]
		pub extern "C" #sig {
			#body
		}
	};

	TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn hotbolt_entry_version(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method(
		"hotbolt_entry_version",
		token_stream,
		|| {
			let ident = format_ident!("{}", ENTRY_CLIENT_VERSION);
			quote! {
				fn #ident()
			}
		},
		|function_ident| {
			let ident = &function_ident.ident;
			quote! {
				#ident()
			}
		},
	)
}

#[proc_macro_attribute]
pub fn hotbolt_entry_compatible(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method(
		"hotbolt_entry_compatible",
		token_stream,
		|| {
			let ident = format_ident!("{}", ENTRY_CLIENT_COMPATIBLE);
			quote! {
				fn #ident()
			}
		},
		|function_ident| {
			let ident = &function_ident.ident;
			quote! {
				#ident()
			}
		},
	)
}

#[proc_macro_attribute]
pub fn hotbolt_entry_init(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method(
		"hotbolt_entry_init",
		token_stream,
		|| {
			let ident = format_ident!("{}", ENTRY_CLIENT_NEW);
			quote! {
				fn #ident()
			}
		},
		|function_ident| {
			let ident = &function_ident.ident;
			quote! {
				#ident()
			}
		},
	)
}

#[proc_macro_attribute]
pub fn hotbolt_entry_destroy(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method(
		"hotbolt_entry_destroy",
		token_stream,
		|| {
			let ident = format_ident!("{}", ENTRY_CLIENT_DROP);
			quote! {
				fn #ident()
			}
		},
		|function_ident| {
			let ident = &function_ident.ident;
			quote! {
				#ident()
			}
		},
	)
}

// TODO: What to do if the main entry is named main (not allowed to our input)?
// TODO: Clippy warning: recursing into entrypoint `main`
#[proc_macro_attribute]
pub fn hotbolt_entry_main(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	let input: Item = syn::parse_macro_input!(token_stream);
	if let Item::Fn(input_function) = &input {
		let input_function_name = &input_function.sig.ident;
		let entry_name = format_ident!("{}", ENTRY_RUN);

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
		// let value = format!("{}", expanded);
		// TokenStream::from(quote! {
		// 	#[doc = #value]
		// 	///
		// 	#input
		// })
	} else {
		panic!("#[hotbolt_entry_main] is intended on a function");
	}
}

#[proc_macro_attribute]
pub fn hotbolt_entry_state_new(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method(
		"hotbolt_entry_state_new",
		token_stream,
		|| {
			let ident = format_ident!("{}", ENTRY_STATE_NEW);
			quote! {
				fn #ident() -> hotbolt::internal::SizedCharArray
			}
		},
		|function_ident| {
			let ident = &function_ident.ident;
			quote! {
				hotbolt::internal::SizedCharArray::from_slice(&#ident())
			}
		},
	)
}

#[proc_macro_attribute]
pub fn hotbolt_entry_state_drop(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method(
		"hotbolt_entry_state_drop",
		token_stream,
		|| {
			let ident = format_ident!("{}", ENTRY_STATE_DROP);
			quote! {
				fn #ident() -> hotbolt::internal::SizedCharArray
			}
		},
		|function_ident| {
			let ident = &function_ident.ident;
			quote! {
				hotbolt::internal::SizedCharArray::from_slice(&#ident())
			}
		},
	)
}

#[proc_macro_attribute]
pub fn hotbolt_entry_state_get(_attr: TokenStream, token_stream: TokenStream) -> TokenStream {
	wrap_method(
		"hotbolt_entry_state_get",
		token_stream,
		|| {
			let ident = format_ident!("{}", ENTRY_STATE_GET);
			quote! {
				fn #ident() -> hotbolt::internal::SizedCharArray
			}
		},
		|function_ident| {
			let ident = &function_ident.ident;
			quote! {
				hotbolt::internal::SizedCharArray::from_slice(&#ident())
			}
		},
	)
}
