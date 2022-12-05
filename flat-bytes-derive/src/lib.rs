#![deny(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
use syn::parse_macro_input;
use syn::Field;
use syn::Fields;
use syn::ItemEnum;
use syn::ItemStruct;

#[proc_macro_derive(Flat)]
pub fn derive_flat(input: TokenStream) -> TokenStream {
    #![allow(clippy::similar_names)]

    let input = parse_macro_input!(input as ItemStruct);

    let ident = &input.ident;

    let fields: Vec<Field> = match input.fields {
        Fields::Named(ref n) => n.named.iter().cloned().collect(),
        Fields::Unnamed(ref un) => un.unnamed.iter().cloned().collect(),
        Fields::Unit => vec![],
    };

    let fields_ser = fields.iter().enumerate().map(|(idx, f)| {
        let ty = &f.ty;
        if let Some(i) = &f.ident {
            quote! {
                res.append(&mut <#ty as Flat>::serialize(&self.#i));
            }
        } else {
            let idx = syn::Index::from(idx);
            quote! {
                res.append(&mut <#ty as Flat>::serialize(&self.#idx));
            }
        }
    });

    let fields_der = fields.iter().enumerate().map(|(idx, f)| {
        let ty = &f.ty;
        if let Some(i) = &f.ident {
            quote! {
                let #i = <#ty as flat_bytes::Flat>::deserialize_with_size(data)?;
                total += #i.1;
                let data = &data[#i.1..];
                let #i = #i.0;
            }
        } else {
            let i = format_ident!("field{}", idx);
            quote! {
                let #i = <#ty as flat_bytes::Flat>::deserialize_with_size(data)?;
                total += #i.1;
                let data = &data[#i.1..];
                let #i = #i.0;
            }
        }
    });

    let alloc = match input.fields {
        Fields::Named(ref n) => {
            let names = n.named.iter().map(|f| f.ident.as_ref().unwrap());
            quote! {
                #ident{#(#names),*}
            }
        }
        Fields::Unnamed(ref un) => {
            let names = (0..un.unnamed.len()).map(|i| format_ident!("field{}", i));
            quote! {
                #ident(#(#names),*)
            }
        }
        Fields::Unit => ident.to_token_stream(),
    };

    let output = quote! {
      impl flat_bytes::Flat for #ident {
        fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
            let mut total = 0;
            #(#fields_der)*
            Some((#alloc, total))
        }

        fn serialize(&self) -> Vec<u8> {
            use flat_bytes::Flat;
            let mut res = vec![];
            #(#fields_ser;)*
            res
        }
      }
    };
    output.into()
}

fn derive_serialize(input: &ItemEnum, dtype: &syn::Path) -> proc_macro2::TokenStream {
    let mut last_idx = 0;
    let match_arms = input.variants.iter().map(|v| {
        let i = v.ident.clone();
        let d = v
            .discriminant
            .as_ref()
            .and_then(|(_, e)| match e {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(i),
                    ..
                }) => i.base10_parse::<u64>().ok(),
                _ => None,
            })
            .unwrap_or(last_idx + 1);
        last_idx = d;
        match &v.fields {
            syn::Fields::Unit => quote! {
              Self::#i => {
                let i = #d as #dtype;
                res.extend_from_slice(&i.to_le_bytes());
              }
            },
            syn::Fields::Unnamed(fu) => {
                let fields = fu
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let ty = &f.ty;
                        let i = format_ident!("field{}", i);
                        let t = quote! {
                            &mut <#ty as Flat>::serialize(#i)
                        };
                        (i, t)
                    })
                    .collect::<Vec<_>>();
                let (names, fields): (Vec<_>, Vec<_>) = fields.iter().cloned().unzip();
                quote! {
                  Self::#i(#(#names),*) => {
                    let i = #d as #dtype;
                    res.extend_from_slice(&i.to_le_bytes());
                    #(
                      res.append(#fields);
                    )*
                  }
                }
            }
            syn::Fields::Named(fs) => {
                let fields = fs
                    .named
                    .iter()
                    .map(|f| {
                        let ty = &f.ty;
                        let i = f.ident.as_ref().unwrap();
                        (
                            i,
                            quote! {
                                &mut <#ty as Flat>::serialize(#i)
                            },
                        )
                    })
                    .collect::<Vec<_>>();
                let (names, fields): (Vec<_>, Vec<_>) = fields.iter().cloned().unzip();
                quote! {
                  Self::#i{#(#names),*} => {
                    let i = #d as #dtype;
                    res.extend_from_slice(&i.to_le_bytes());
                    #(
                      res.append(#fields);
                    )*
                  }
                }
            }
        }
    });

    quote! {
      let mut res: Vec<u8> = vec![];
      match self {
        #(#match_arms),*
      }
      res
    }
}

fn derive_deserialize(input: &ItemEnum, dtype: &syn::Path) -> proc_macro2::TokenStream {
    let ident = &input.ident;
    let mut last_idx = 0;
    let match_arms = input.variants.iter().map(|v| {
        let i = v.ident.clone();
        let d = v
            .discriminant
            .as_ref()
            .and_then(|(_, e)| match e {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(i),
                    ..
                }) => i.base10_parse::<u64>().ok(),
                _ => None,
            })
            .unwrap_or(last_idx + 1);
        last_idx = d;
        match &v.fields {
            syn::Fields::Unit => quote! {
              #d => {
                Some((#ident::#i, total))
              }
            },
            syn::Fields::Unnamed(fu) => {
                let fields = fu
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let name = quote::format_ident!("field{}", i);
                        let ty = &f.ty;
                        quote! {
                          let #name = #ty::deserialize_with_size(data)?;
                          let data = &data[#name.1..];
                          total += #name.1;
                          let #name = #name.0;
                        }
                    })
                    .collect::<Vec<_>>();
                let field_names = fu
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _f)| quote::format_ident!("field{}", i))
                    .collect::<Vec<_>>();
                quote! {
                  #d => {
                    #(
                      #fields
                    )*
                    Some((#ident::#i(#(#field_names),*), total))
                  }
                }
            }
            syn::Fields::Named(fs) => {
                let fields = fs
                    .named
                    .iter()
                    .map(|f| {
                        let name = f.ident.clone().unwrap();
                        let ty = &f.ty;
                        quote! {
                          let #name = #ty::deserialize_with_size(data)?;
                          let data = &data[#name.1..];
                          total += #name.1;
                          let #name = #name.0;
                        }
                    })
                    .collect::<Vec<_>>();
                let field_names = fs
                    .named
                    .iter()
                    .map(|f| f.ident.clone().unwrap())
                    .collect::<Vec<_>>();
                quote! {
                  #d => {
                    #(
                      #fields
                    )*
                    Some((#ident::#i{#(#field_names),*}, total))
                  }
                }
            }
        }
    });

    quote! {
      if data.len() < ::std::mem::size_of::<#dtype>() {
        return None
      }
      let idx = {
        let mut tmp = [0u8; ::std::mem::size_of::<#dtype>()];
        tmp.copy_from_slice(&data[..::std::mem::size_of::<#dtype>()]);
        #dtype::from_le_bytes(tmp) as u64
      };
      let data = &data[::std::mem::size_of::<#dtype>()..];
      let mut total = ::std::mem::size_of::<#dtype>();

      match idx {
        #(#match_arms,)*
        _ => None,
      }
    }
}

#[proc_macro]
pub fn flat_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    let mut enum_output = input.clone();
    for v in enum_output.variants.iter_mut() {
        v.discriminant = None;
    }

    let ident = &input.ident;
    let dtype = input
        .attrs
        .iter()
        .flat_map(syn::Attribute::parse_meta)
        .find_map(|m| {
            if !m.path().is_ident("repr") {
                return None;
            }
            match m {
                syn::Meta::List(l) => match l.nested.first() {
                    Some(syn::NestedMeta::Meta(m)) => Some(m.path().clone()),
                    _ => None,
                },
                _ => None,
            }
        })
        .unwrap();

    let serialize = derive_serialize(&input, &dtype);
    let deserialize = derive_deserialize(&input, &dtype);

    (quote! {
      #enum_output

      impl flat_bytes::Flat for #ident {
        fn deserialize_with_size(data: &[u8]) -> Option<(Self, usize)> {
          #deserialize
        }

        fn serialize(&self) -> Vec<u8> {
          use flat_bytes::Flat;
          #serialize
        }
      }
    })
    .into()
}
