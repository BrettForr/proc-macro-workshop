use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DeriveInput, Error, Fields, GenericArgument, PathArguments, Type,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;
    let parsed_input = parse_macro_input!(input as DeriveInput);

    let name = parsed_input.ident;

    let builder_name = format_ident!("{}Builder", name);

    let builder_fields = generate_builder_fields(&parsed_input.data, &name);

    let builder_inits = initialize_builder_fields(&parsed_input.data, &name);

    let builder_setters = generate_builder_setters(&parsed_input.data, &name);

    let set_expressions = set_fields(&parsed_input.data, &name);

    let expanded = quote! {
        pub struct #builder_name {
            #builder_fields
        }

        impl #builder_name {
            pub fn build(&self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #set_expressions
                })
            }

            #builder_setters
        }

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #builder_inits
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_builder_fields(data: &Data, ident: &proc_macro2::Ident) -> proc_macro2::TokenStream {
    let expanded = match *data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => {
                let field_options = fields_named.named.iter().map(|field| {
                    let name = &field.ident;
                    let ty = &field.ty;

                    let name_unwrapped = name.as_ref();

                    if let Some(n) = name_unwrapped {
                        if is_option_type(ty) {
                            quote! { #n: #ty }
                        } else {
                            quote! { #n: Option<#ty>}
                        }
                    } else {
                        Error::new_spanned(ident, "Expected Struct with Named Fields")
                            .to_compile_error()
                    }
                });

                quote! {
                    #(#field_options,)*
                }
            }
            Fields::Unnamed(ref _fields_unnamed) => {
                Error::new_spanned(ident, "Expected Struct with Named Fields").to_compile_error()
            }
            Fields::Unit => {
                Error::new_spanned(ident, "Expected Struct with Named Fields").to_compile_error()
            }
        },
        _ => Error::new_spanned(ident, "Expected Struct with Named Fields").to_compile_error(),
    };

    expanded
}

fn initialize_builder_fields(data: &Data, ident: &proc_macro2::Ident) -> proc_macro2::TokenStream {
    let expanded = match *data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => {
                let fields_initial = fields_named.named.iter().map(|field| {
                    let name = &field.ident;

                    let name_unwrapped = name.as_ref();

                    if let Some(n) = name_unwrapped {
                        quote! { #n: None }
                    } else {
                        Error::new_spanned(ident, "Expected Struct with Named Fields")
                            .to_compile_error()
                    }
                });

                quote! {
                    #(#fields_initial,)*
                }
            }
            _ => Error::new_spanned(ident, "Expected Struct with Named Fields").to_compile_error(),
        },
        _ => Error::new_spanned(ident, "Expected Struct with Named Fields").to_compile_error(),
    };

    expanded
}

fn generate_builder_setters(data: &Data, ident: &proc_macro2::Ident) -> proc_macro2::TokenStream {
    let expanded = match *data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => {
                let fields_setters = fields_named.named.iter().map(|field| {
                    let name = &field.ident;
                    let field_type = &field.ty;

                    let name_unwrapped = name.as_ref();

                    if let Some(n) = name_unwrapped {
                        if is_option_type(field_type) {
                            let inner_ty = inner_option_type(field_type);
                            quote! { pub fn #n(&mut self, #n: #inner_ty) -> &mut Self {
                                self.#n = Some(#n);
                                self
                            }}
                        } else {
                            quote! { pub fn #n(&mut self, #n: #field_type) -> &mut Self {
                                self.#n = Some(#n);
                                self
                            }}
                        }
                    } else {
                        Error::new_spanned(ident, "Expected Struct with Named Fields")
                            .to_compile_error()
                    }
                });

                quote! {
                    #(#fields_setters)*
                }
            }
            _ => Error::new_spanned(ident, "Expected Struct with Named Fields").to_compile_error(),
        },
        _ => Error::new_spanned(ident, "Expected Struct with Named Fields").to_compile_error(),
    };

    expanded
}

fn set_fields(data: &Data, ident: &proc_macro2::Ident) -> proc_macro2::TokenStream {
    let expanded = match *data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => {
                let set_expressions = fields_named.named.iter().map(|field| {
                    let name = &field.ident;
                    let ty = &field.ty;

                    let name_unwrapped = name.as_ref();

                    if let Some(n) = name_unwrapped {
                        if is_option_type(ty) {
                            quote! {
                                #n: self.#n.clone()
                            }
                        } else {
                            let name_value = format_ident!("{}_value", n);

                            quote! {
                                #n: {if let Some(#name_value) = self.#n.clone() {
                                    #name_value
                                } else {
                                    return Err(format!("Field {} is not set", stringify!(#n)).into());
                                }}
                            }
                        }
                    } else {
                        Error::new_spanned(ident, "Expected Struct with Named Fields")
                            .to_compile_error()
                    }
                });

                quote! {
                    #(#set_expressions,)*
                }
            }
            _ => Error::new_spanned(ident, "Expected Struct with Named Fields").to_compile_error(),
        },
        _ => Error::new_spanned(ident, "Expected Struct with Named Fields").to_compile_error(),
    };

    expanded
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(ref type_path) = ty {
        let segments = &type_path.path.segments;

        if let Some(first_seg) = segments.first() {
            first_seg.ident == "Option"
        } else {
            false
        }
    } else {
        false
    }
}

fn inner_option_type(ty: &Type) -> &Type {
    if let Type::Path(type_path) = ty {
        if let Some(first_seg) = type_path.path.segments.first() {
            if let PathArguments::AngleBracketed(args) = &first_seg.arguments {
                if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                    return inner_ty;
                }
            }
        }
    }

    panic!("Should be an option"); // TODO: return error
}
