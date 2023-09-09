use syn::{parse_macro_input, DeriveInput };
use quote::quote;
use proc_macro;

#[proc_macro_derive(Message)]
pub fn message_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = input.ident;
    let type_name_str = type_name.to_string();

    let output = quote!(
        impl bus_rs::MessageTypeName for #type_name {
            fn name() -> &'static str {
                #type_name_str
            }
        }

        impl Into<bus_rs::RawMessage> for #type_name {
            fn into(self) -> bus_rs::RawMessage {
                let payload = serde_json::to_string(&self).unwrap(); 

                bus_rs::RawMessage {
                    msg_type: #type_name_str.to_string(),
                    payload: payload
                }
            }
        }
    );

    proc_macro::TokenStream::from(output)
}
