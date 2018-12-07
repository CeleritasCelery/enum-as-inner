extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro2::{Ident, Span, TokenStream};
use syn::DeriveInput;

/// returns first the types to return, the match names, and then tokens to the field accesses
fn unnamed_fields_return(fields: &syn::FieldsUnnamed) -> (TokenStream, TokenStream, TokenStream) {
    match fields.unnamed.len() {
        1 => {
            let field = fields.unnamed.first().expect("no fields on type");
            let field = field.value();

            let returns = &field.ty;
            let returns = quote!(&#returns);
            let matches = quote!(inner);
            let accesses = quote!(&inner);

            (returns, matches, accesses)
        }
        0 => (quote!(()), quote!(), quote!(())),
        _ => {
            let mut returns = TokenStream::new();
            let mut matches = TokenStream::new();
            let mut accesses = TokenStream::new();

            for (i, ty) in fields.unnamed.iter().enumerate() {
                let rt = &ty.ty;
                let match_name = Ident::new(&format!("match_{}", i), Span::call_site());
                returns.extend(quote!(&#rt,));
                matches.extend(quote!(#match_name,));
                accesses.extend(quote!(&#match_name,));
            }

            // panic!(
            //     "returns: {} mathes: {} accesses: {}",
            //     returns, matches, accesses
            // );

            (quote!((#returns)), quote!(#matches), quote!((#accesses)))
        }
    }
}

fn impl_all_as_fns(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let enum_data = if let syn::Data::Enum(data) = &ast.data {
        data
    } else {
        panic!("{} is not an enum", name);
    };

    let mut stream = TokenStream::new();

    for variant_data in &enum_data.variants {
        let variant_name = &variant_data.ident;
        let function_name = Ident::new(
            &format!("as_{}", variant_name).to_lowercase(),
            Span::call_site(),
        );

        let (returns, matches, accesses) = match &variant_data.fields {
            syn::Fields::Unnamed(unnamed) => unnamed_fields_return(&unnamed),
            _ => panic!("not supported"),
        };

        stream.extend(quote!(
            impl #name {
                pub fn #function_name(&self) -> Option<#returns> {
                   match self {
                       #name::#variant_name(#matches) => {
                           Some(#accesses)
                       }
                       _ => None
                   }
                }
            }
        ));
    }

    stream
}

#[proc_macro_derive(EnumAsInner)]
pub fn enum_as_inner(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // get a usable token stream
    let ast: DeriveInput = parse_macro_input!(input as DeriveInput);

    // Build the impl
    let expanded: TokenStream = impl_all_as_fns(&ast);

    // Return the generated impl
    proc_macro::TokenStream::from(expanded)
}
