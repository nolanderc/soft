extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree};
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[proc_macro_derive(Interpolate)]
pub fn derive_interpolate(input: TokenStream) -> TokenStream {
    match parse_item(input) {
        Err(e) => e.to_compile_error().into(),
        Ok(item) => {
            let field_init = item.fields.into_iter().map(|Field { name, ty }| {
                let name = match name {
                    FieldName::Ident(ident) => quote!(#ident),
                    FieldName::Index(index, span) => {
                        let index = proc_macro::Literal::u32_unsuffixed(index);
                        let stream = TokenStream::from(TokenTree::Literal(index));
                        let stream2 = proc_macro2::TokenStream::from(stream);
                        quote_spanned!(span=> #stream2)
                    },
                };
                quote! {
                    #name: <#ty as soft::Interpolate>::tri_lerp(
                        &[values[0].#name, values[1].#name, values[2].#name],
                        factors,
                    )
                }
            });

            let name = item.name;
            let output = quote! {
                impl soft::Interpolate for #name {
                    fn tri_lerp(values: &[Self; 3], factors: [f32; 3]) -> Self {
                        Self {
                            #(#field_init),*
                        }
                    }
                }
            };
            output.into()
        }
    }
}

struct Item {
    name: syn::Ident,
    fields: Vec<Field>,
}

struct Field {
    name: FieldName,
    ty: syn::Type,
}

enum FieldName {
    Ident(syn::Ident),
    Index(u32, Span),
}

fn parse_item(input: TokenStream) -> syn::Result<Item> {
    let item = syn::parse::<syn::DeriveInput>(input)?;

    match item.data {
        syn::Data::Enum(data) => Err(syn::Error::new(
            data.enum_token.span,
            "expected `struct`, found `enum`",
        )),
        syn::Data::Union(data) => Err(syn::Error::new(
            data.union_token.span,
            "expected `struct`, found `union`",
        )),
        syn::Data::Struct(data) => {
            let fields = data
                .fields
                .into_iter()
                .enumerate()
                .map(|(i, field)| {
                    let ty = field.ty;
                    let name = field
                        .ident
                        .map(FieldName::Ident)
                        .unwrap_or_else(|| FieldName::Index(i as _, ty.span()));
                    Field { name, ty }
                })
                .collect();
            Ok(Item {
                name: item.ident,
                fields,
            })
        }
    }
}
