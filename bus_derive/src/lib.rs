use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{self};
use syn::{DeriveInput, Lit, Meta, NestedMeta};

#[proc_macro_derive(RedisStream, attributes(stream_name))]
pub fn derive_redis_stream(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let ident = &ast.ident;
    match stream_name(ast.clone()) {
        Ok(stream_name) => {
            let gen = quote! {
                impl #ident {
                    pub const name: &'static str = #stream_name;
                }
                impl redis_stream_bus::StreamParsable for #ident{
                    fn key(&self)-> String {
                        #stream_name.to_string()
                    }
                }
            };
            gen.into()
        }
        Err(msg) => {
            let tokens = quote_spanned! { proc_macro2::Span::call_site() =>
                compile_error!(#msg);
            };
            tokens.into()
        }
    }
}

pub(crate) fn stream_name(input: DeriveInput) -> Result<Lit, String> {
    let attrs = input.attrs;
    match attrs.get(0) {
        Some(attr) => match attr.parse_meta() {
            Ok(meta) => {
                let name = meta.path().get_ident();
                if name.expect("please add stream name") == "stream_name" {
                    match meta {
                        Meta::List(list) => match list.nested.first().unwrap() {
                            NestedMeta::Lit(l) => Ok(l.clone()),
                            _ => Err("Not a lit str".to_owned()),
                        },
                        _ => Err("Not a meta list".to_owned()),
                    }
                } else {
                    Err("invalid name, should be stream_name".to_owned())
                }
            }
            Err(_) => Err("Unable to parse meta".to_owned()),
        },
        None => Err("Need stream_name".to_owned()),
    }
}


