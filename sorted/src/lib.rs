use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, ItemEnum};

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_item = handle_parsing(input);

    let expanded = match parsed_item {
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

fn check_order(item_enum: &ItemEnum) -> Result<(), syn::Error> {
    let variants = &item_enum.variants;

    let mut previous_varint = String::new();
    for variant in variants.iter() {
        let variant_ident = &variant.ident;

        let name = variant_ident.to_string();

        if name < previous_varint {
            // TODO: this seems messy
            for inner_variant in variants.iter() {
                let inner_name = (&inner_variant.ident).to_string();
                if name < inner_name {
                    return Err(syn::Error::new(
                        variant_ident.span(),
                        format!("{} should sort before {}", name, inner_name).as_str(),
                    ));
                }
            }
        }

        previous_varint = name;
    }

    Ok(())
}

fn handle_parsing(input: TokenStream) -> Result<ItemEnum, syn::Error> {
    let parsed_item_enum = parse_enum(input)?;

    check_order(&parsed_item_enum)?;

    Ok(parsed_item_enum)
}
