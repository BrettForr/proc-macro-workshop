use proc_macro::TokenStream;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{spanned::Spanned, ExprMatch, Item, ItemEnum, ItemFn};
use syn::{Arm, Ident};

#[proc_macro_attribute]
pub fn check(_args: TokenStream, input: TokenStream) -> TokenStream {
    match find_sort_sites(input.clone()) {
        Ok(item_fn) => item_fn,
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn find_sort_sites(input: TokenStream) -> Result<TokenStream, syn::Error> {
    let parsed_input = syn::parse::<Item>(input)?;

    match parsed_input {
        Item::Fn(item_fn) => {
            let modified_fn = traverse_fn(item_fn)?;
            Ok(modified_fn)
        }
        _ => Err(syn::Error::new(
            parsed_input.span(),
            "sorted::check must be applied to a fn.",
        )),
    }
}

fn traverse_fn(item_fn: ItemFn) -> Result<TokenStream, syn::Error> {
    let mut item_fn = item_fn;

    let mut match_visitor = MatchVisitor { error: None };

    match_visitor.visit_item_fn_mut(&mut item_fn);

    let compile_error = match match_visitor.error {
        Some(err) => err.to_compile_error(),
        None => quote! {},
    };

    let expanded = quote! {
        #compile_error
        #item_fn
    };

    Ok(TokenStream::from(expanded))
}

struct MatchVisitor {
    error: Option<syn::Error>,
}

impl VisitMut for MatchVisitor {
    fn visit_expr_match_mut(&mut self, node: &mut ExprMatch) {
        let attrs = &node.attrs;

        let sorted_attr_position = attrs.iter().position(|attr| attr.path().is_ident("sorted"));

        if let Some(position) = sorted_attr_position {
            node.attrs.remove(position);

            let mut previous_arm = String::new();

            for arm in node.arms.iter() {
                // eprintln!("In {:?}", arm);

                match get_arm_name(arm) {
                    Ok(current_ident) => {
                        let current_name = current_ident.to_string();
                        if current_name < previous_arm {
                            self.error = Some(syn::Error::new_spanned(
                                current_ident,
                                format!("{} should sort before {}", current_name, previous_arm)
                                    .as_str(),
                            ));
                            break;
                        }

                        previous_arm = current_name;
                    }
                    Err(err) => {
                        self.error = Some(err);
                        break;
                    }
                }
            }
        }

        syn::visit_mut::visit_expr_match_mut(self, node);
    }
}

fn get_arm_name(arm: &Arm) -> Result<Ident, syn::Error> {
    let arm_span = arm.span();

    match &arm.pat {
        syn::Pat::Path(expr_path) => {
            let current_item = expr_path.path.segments.last();

            if let Some(segment) = current_item {
                Ok(segment.ident.clone())
            } else {
                Err(syn::Error::new(arm_span, "No path fields"))
            }
        }
        syn::Pat::TupleStruct(tuple_struct) => {
            let ident = tuple_struct.path.segments.last();
            ident
                .map(|segment| segment.ident.to_owned())
                .ok_or_else(|| {
                    syn::Error::new(arm_span, "Tuple struct doesn't have a single ident")
                })
        }
        syn::Pat::Struct(pat_struct) => {
            let ident = pat_struct.path.get_ident();
            ident.map(|id| id.to_owned()).ok_or_else(|| {
                syn::Error::new(arm_span, "Tuple struct doesn't have a single ident")
            })
        }
        syn::Pat::Slice(_) => Err(syn::Error::new(arm.pat.span(), "unsupported by #[sorted]")),
        _ => Err(syn::Error::new(
            arm_span,
            "Expect match arm to be a path, tuple struct, or struct",
        )),
    }
}

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
                let inner_name = inner_variant.ident.to_string();
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
