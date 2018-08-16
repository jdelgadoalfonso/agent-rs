#![crate_type = "proc-macro"]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{
    DeriveInput, Field, Fields,
    NestedMeta::Meta,
    Meta::{
        List,
        NameValue,
        Word
    },
    punctuated::Punctuated
};


macro_rules! my_quote {
    ($($t:tt)*) => (quote_spanned!(Span::call_site() => $($t)*))
}

fn make_push_fn(
    ast: &DeriveInput,
    fields: Option<&Punctuated<Field, Token![,]>>
) -> TokenStream2
{
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let empty = Default::default();
    let fields: Vec<_> = fields.unwrap_or(&empty).iter().enumerate()    
    .map(|(_i, f)| {
        let ident = &f.ident;
        let ty = &f.ty;
        for attr in &f.attrs {
            if attr.path.segments[0].ident == "influx" {
                if let Some(List(ref m)) = attr.interpret_meta() {
                    for nm in m.nested.iter() {
                        if let Meta(Word(ref word)) = nm {
                            if word == "skip" {
                                return my_quote!();
                            }
                        }

                        if let Meta(NameValue(ref nv)) = nm {
                            if nv.ident == "datatype" {
                                if let syn::Lit::Str(ref lit) = &nv.lit {
                                    let method = syn::Ident::new(&lit.value(), Span::call_site());
                                    let to = syn::Ident::new(
                                        &format!("to_{}", lit.value().to_lowercase()), Span::call_site());
                                    return my_quote! {
                                        point.add_field(stringify!(#ident), Value::#method(self.#ident.#to()));
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }
        let (method, tas) = match format!("{}", quote!(#ty)).as_ref() {
            "u64" | "u32" | "u16" | "u8" | "i32" | "i16" | "i8" => ( quote!(Integer), my_quote!(as i64) ),
            "f32" => ( quote!(Float), my_quote!(as f64) ),
            "i64" => ( quote!(Integer), my_quote!() ),
            "f64" => ( quote!(Float), my_quote!() ),
            "bool" => ( quote!(Boolean), my_quote!() ),
            "String" => ( quote!(String), my_quote!() ),
            _ => ( quote!(String), my_quote!(.to_string()) ),
        };
        quote! {
            point.add_field(stringify!(#ident), Value::#method(self.#ident #tas));
        }
    }).collect();
    my_quote! {
        impl #impl_generics Influx for #name #ty_generics #where_clause {
            fn push(self) -> Point {
                let mut point = point!(stringify!(#name));
                #(#fields)*
                point
            }
        }
    }
}

fn for_struct(ast: &DeriveInput, fields: &Fields) -> TokenStream2
{
    match *fields {
        syn::Fields::Named(ref fields) => {
            make_push_fn(&ast, Some(&fields.named))
        },
        syn::Fields::Unit => {
            make_push_fn(&ast, None)
        },
        syn::Fields::Unnamed(ref fields) => {
            make_push_fn(&ast, Some(&fields.unnamed))
        },
    }
}

#[proc_macro_derive(InfluxDB, attributes(influx))]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).expect("Couldn't parse item");

    // Build the impl
    let expanded = match ast.data {
        syn::Data::Struct(ref s) => for_struct(&ast, &s.fields),
        syn::Data::Enum(_) => panic!("doesn't work with enums yet"),
        syn::Data::Union(_) => panic!("doesn't work with unions yet"),
    };

    expanded.into()
}
