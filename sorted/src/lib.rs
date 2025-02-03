use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, ItemEnum};

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_item_enum = parse_enum(input);

    let expanded = match parsed_item_enum {
        Ok(item_enum) => quote! {
            #item_enum
        },
        Err(err) => err.to_compile_error(),
    };

    TokenStream::from(expanded)
}

fn parse_enum(input: TokenStream) -> Result<ItemEnum, syn::Error> {
    let parsed_input = syn::parse::<Item>(input)?;

    match parsed_input {
        Item::Enum(item_enum) => Ok(item_enum),
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        )),
    }
}
