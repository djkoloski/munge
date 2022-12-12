//! The macro for `munge`.

#![deny(
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

use ::proc_macro2::TokenStream;
use ::quote::quote;
use ::syn::{
    parse,
    parse_macro_input,
    punctuated::Punctuated,
    token::{Eq, FatArrow, Let, Semi},
    Error,
    Expr,
    Index,
    Pat,
    PatTupleStruct,
    TypePath,
};

/// Destructures a value by projecting pointers.
#[proc_macro]
pub fn munge_with_path(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Input);
    destructure(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

struct Input {
    crate_path: TypePath,
    _arrow: FatArrow,
    destructures: Punctuated<Destructure, Semi>,
}

impl parse::Parse for Input {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        Ok(Input {
            crate_path: input.parse::<TypePath>()?,
            _arrow: input.parse::<FatArrow>()?,
            destructures: input.parse_terminated(Destructure::parse)?,
        })
    }
}

struct Destructure {
    _let_token: Let,
    pat: Pat,
    _eq_token: Eq,
    expr: Expr,
}

impl parse::Parse for Destructure {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        Ok(Destructure {
            _let_token: input.parse::<Let>()?,
            pat: input.parse::<Pat>()?,
            _eq_token: input.parse::<Eq>()?,
            expr: input.parse::<Expr>()?,
        })
    }
}

fn parse_pat(
    crate_path: &TypePath,
    pat: &Pat,
) -> Result<(TokenStream, TokenStream), Error> {
    Ok(match pat {
        Pat::Ident(pat_ident) => {
            let mutability = &pat_ident.mutability;
            let ident = &pat_ident.ident;
            (
                quote! { #mutability #ident },
                quote! {
                    unsafe {
                        #crate_path::Restructure::restructure(&value, ptr)
                    }
                },
            )
        }
        Pat::Tuple(pat) | Pat::TupleStruct(PatTupleStruct { pat, .. }) => {
            let parsed = pat
                .elems
                .iter()
                .map(|e| parse_pat(crate_path, e))
                .collect::<Result<Vec<_>, Error>>()?;
            let (idents, (exprs, indices)) = parsed
                .iter()
                .enumerate()
                .map(|(i, x)| (&x.0, (&x.1, Index::from(i))))
                .unzip::<_, _, Vec<_>, (Vec<_>, Vec<_>)>();
            (
                quote! { (#(#idents,)*) },
                quote! { (
                    #({
                        let ptr = unsafe {
                            ::core::ptr::addr_of_mut!((*ptr).#indices)
                        };
                        #exprs
                    },)*
                ) },
            )
        }
        Pat::Slice(pat_slice) => {
            let parsed = pat_slice
                .elems
                .iter()
                .map(|e| parse_pat(crate_path, e))
                .collect::<Result<Vec<_>, Error>>()?;
            let (idents, (exprs, indices)) = parsed
                .iter()
                .enumerate()
                .map(|(i, x)| (&x.0, (&x.1, Index::from(i))))
                .unzip::<_, _, Vec<_>, (Vec<_>, Vec<_>)>();
            (
                quote! { (#(#idents,)*) },
                quote! { (
                    #({
                        let ptr = unsafe {
                            ::core::ptr::addr_of_mut!((*ptr)[#indices])
                        };
                        #exprs
                    },)*
                ) },
            )
        }
        Pat::Struct(pat_struct) => {
            let parsed = pat_struct
                .fields
                .iter()
                .map(|fp| {
                    parse_pat(crate_path, &fp.pat).map(|ie| (&fp.member, ie))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            let (members, (idents, exprs)) =
                parsed.into_iter().unzip::<_, _, Vec<_>, (Vec<_>, Vec<_>)>();
            (
                quote! { (#(#idents,)*) },
                quote! { (
                    #({
                        let ptr = unsafe {
                            ::core::ptr::addr_of_mut!((*ptr).#members)
                        };
                        #exprs
                    },)*
                ) },
            )
        }
        Pat::Rest(pat_rest) => {
            let token = &pat_rest.dot2_token;
            (quote! { #token }, quote! {})
        }
        Pat::Wild(pat_wild) => {
            let token = &pat_wild.underscore_token;
            (quote! { #token }, quote! {})
        }
        _ => {
            return Err(Error::new_spanned(
                pat,
                "expected a destructuring pattern",
            ))
        }
    })
}

fn destructure(input: Input) -> Result<TokenStream, Error> {
    let crate_path = &input.crate_path;

    let mut result = TokenStream::new();
    for destructure in input.destructures.iter() {
        let pat = &destructure.pat;
        let expr = &destructure.expr;

        let (bindings, exprs) = parse_pat(crate_path, pat)?;

        result.extend(quote! {
            let mut value = #expr;
            let #bindings = {
                #[allow(
                    unused_mut,
                    unused_unsafe,
                    clippy::undocumented_unsafe_blocks,
                )]
                {
                    let ptr = #crate_path::Destructure::as_mut_ptr(&mut value);

                    #[allow(unreachable_code, unused_variables)]
                    if false {
                        unsafe {
                            ::core::hint::unreachable_unchecked();
                            let #pat = &*ptr;
                        }
                    }

                    #exprs
                }
            };
        });
    }
    Ok(result)
}
