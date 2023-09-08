use syn::{parse_macro_input, DeriveInput };
use quote::quote;
use proc_macro;

#[proc_macro_derive(MessageResolver)]
pub fn message_resolver_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = input.ident;
    let type_name_str = type_name.to_string();

    let output = quote!(
        impl bus_rs::MessageResolver for #type_name {
            fn name() -> &'static str {
                #type_name_str
            }
        }
    );

    proc_macro::TokenStream::from(output)
}
