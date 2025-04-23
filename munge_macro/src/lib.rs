//! The proc macro at the core of munge.

#![deny(
    missing_docs,
    unsafe_op_in_unsafe_fn,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse, parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Eq, FatArrow, Let, Semi},
    Error, Expr, FieldPat, Index, Pat, PatIdent, PatRest, PatSlice, PatStruct,
    PatTuple, PatTupleStruct, Path,
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
    crate_path: Path,
    _arrow: FatArrow,
    destructures: Punctuated<Destructure, Semi>,
}

impl parse::Parse for Input {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        Ok(Input {
            crate_path: input.parse::<Path>()?,
            _arrow: input.parse::<FatArrow>()?,
            destructures: input.parse_terminated(Destructure::parse, Semi)?,
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
            pat: Pat::parse_single(input)?,
            _eq_token: input.parse::<Eq>()?,
            expr: input.parse::<Expr>()?,
        })
    }
}

fn rest_check(crate_path: &Path, rest: &PatRest) -> (TokenStream, TokenStream) {
    let span = rest.dot2_token.span();
    let destructurer = quote! { destructurer };

    let expr = quote_spanned! { span => {
        let phantom = #crate_path::__macro::get_destructure(&#destructurer);
        #crate_path::__macro::only_borrow_destructuring_may_use_rest_patterns(
            phantom
        )
    } };
    (quote_spanned! { span => _ }, expr)
}

fn parse_pat(
    crate_path: &Path,
    pat: &Pat,
) -> Result<(TokenStream, TokenStream), Error> {
    Ok(match pat {
        Pat::Ident(pat_ident) => {
            let mutability = &pat_ident.mutability;
            let ident = &pat_ident.ident;

            if let Some(r#ref) = &pat_ident.by_ref {
                return Err(Error::new_spanned(
                    r#ref,
                    "`ref` is not allowed in munge destructures",
                ));
            }
            if let Some((at, _)) = &pat_ident.subpat {
                return Err(Error::new_spanned(
                    at,
                    "subpatterns are not allowed in munge destructures",
                ));
            }

            (
                quote! { #mutability #ident },
                quote! {
                    // SAFETY: `ptr` is a properly-aligned pointer to a subfield
                    // of the pointer underlying `destructurer`.
                    unsafe {
                        #crate_path::__macro::restructure_destructurer(
                            &destructurer,
                            ptr,
                        )
                    }
                },
            )
        }
        Pat::Tuple(PatTuple { elems, .. })
        | Pat::TupleStruct(PatTupleStruct { elems, .. }) => {
            let parsed = elems
                .iter()
                .map(|e| parse_pat(crate_path, e))
                .collect::<Result<Vec<_>, Error>>()?;
            let (bindings, (exprs, indices)) = parsed
                .iter()
                .enumerate()
                .map(|(i, x)| (&x.0, (&x.1, Index::from(i))))
                .unzip::<_, _, Vec<_>, (Vec<_>, Vec<_>)>();
            (
                quote! { (#(#bindings,)*) },
                quote! { (
                    #({
                        // SAFETY: `ptr` is guaranteed to always be non-null,
                        // properly-aligned, and valid for reads.
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
            let (bindings, (exprs, indices)) = parsed
                .iter()
                .enumerate()
                .map(|(i, x)| (&x.0, (&x.1, Index::from(i))))
                .unzip::<_, _, Vec<_>, (Vec<_>, Vec<_>)>();
            (
                quote! { (#(#bindings,)*) },
                quote! { (
                    #({
                        // SAFETY: `ptr` is guaranteed to always be non-null,
                        // properly-aligned, and valid for reads.
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
            let (members, (bindings, exprs)) =
                parsed.into_iter().unzip::<_, _, Vec<_>, (Vec<_>, Vec<_>)>();

            let (rest_binding, rest_expr) = if let Some(rest) = &pat_struct.rest
            {
                let (binding, expr) = rest_check(crate_path, rest);
                (Some(binding), Some(expr))
            } else {
                (None, None)
            };

            (
                quote! { (
                    #(#bindings,)*
                    #rest_binding
                ) },
                quote! { (
                    #({
                        // SAFETY: `ptr` is guaranteed to always be non-null,
                        // properly-aligned, and valid for reads.
                        let ptr = unsafe {
                            ::core::ptr::addr_of_mut!((*ptr).#members)
                        };
                        #exprs
                    },)*
                    #rest_expr
                ) },
            )
        }
        Pat::Rest(pat_rest) => rest_check(crate_path, pat_rest),
        Pat::Wild(pat_wild) => {
            let token = &pat_wild.underscore_token;
            (
                quote! { #token },
                quote! {
                    // SAFETY: `ptr` is a properly-aligned pointer to a subfield
                    // of the pointer underlying `destructurer`.
                    unsafe {
                        #crate_path::__macro::restructure_destructurer(
                            &destructurer,
                            ptr,
                        )
                    }
                },
            )
        }
        _ => {
            return Err(Error::new_spanned(
                pat,
                "expected a destructuring pattern",
            ));
        }
    })
}

fn strip_mut(pat: &Pat) -> Result<Pat, Error> {
    Ok(match pat {
        Pat::Ident(pat_ident) => Pat::Ident(PatIdent {
            attrs: pat_ident.attrs.clone(),
            by_ref: None,
            mutability: None,
            ident: pat_ident.ident.clone(),
            subpat: if let Some((at, pat)) = pat_ident.subpat.as_ref() {
                Some((*at, Box::new(strip_mut(pat)?)))
            } else {
                None
            },
        }),
        Pat::Tuple(pat_tuple) => {
            let mut elems = Punctuated::new();
            for elem in pat_tuple.elems.iter() {
                elems.push(strip_mut(elem)?);
            }
            Pat::Tuple(PatTuple {
                attrs: pat_tuple.attrs.clone(),
                paren_token: pat_tuple.paren_token,
                elems,
            })
        }
        Pat::TupleStruct(pat_tuple_struct) => {
            let mut elems = Punctuated::new();
            for elem in pat_tuple_struct.elems.iter() {
                elems.push(strip_mut(elem)?);
            }
            Pat::TupleStruct(PatTupleStruct {
                attrs: pat_tuple_struct.attrs.clone(),
                qself: pat_tuple_struct.qself.clone(),
                path: pat_tuple_struct.path.clone(),
                paren_token: pat_tuple_struct.paren_token,
                elems,
            })
        }
        Pat::Slice(pat_slice) => {
            let mut elems = Punctuated::new();
            for elem in pat_slice.elems.iter() {
                elems.push(strip_mut(elem)?);
            }
            Pat::Slice(PatSlice {
                attrs: pat_slice.attrs.clone(),
                bracket_token: pat_slice.bracket_token,
                elems,
            })
        }
        Pat::Struct(pat_struct) => {
            let mut fields = Punctuated::new();
            for field in pat_struct.fields.iter() {
                fields.push(FieldPat {
                    attrs: field.attrs.clone(),
                    member: field.member.clone(),
                    colon_token: field.colon_token,
                    pat: Box::new(strip_mut(&field.pat)?),
                });
            }
            Pat::Struct(PatStruct {
                attrs: pat_struct.attrs.clone(),
                qself: pat_struct.qself.clone(),
                path: pat_struct.path.clone(),
                brace_token: pat_struct.brace_token,
                fields,
                rest: pat_struct.rest.clone(),
            })
        }
        Pat::Rest(pat_rest) => Pat::Rest(pat_rest.clone()),
        Pat::Wild(pat_wild) => Pat::Wild(pat_wild.clone()),
        _ => todo!(),
    })
}

fn destructure(input: Input) -> Result<TokenStream, Error> {
    let crate_path = &input.crate_path;

    let mut result = TokenStream::new();
    for destructure in input.destructures.iter() {
        let pat = &destructure.pat;
        let expr = &destructure.expr;

        let test_pat = strip_mut(pat)?;

        let (bindings, exprs) = parse_pat(crate_path, pat)?;

        result.extend(quote! {
            let mut destructurer = #crate_path::__macro::make_destructurer(
                #expr
            );
            let #bindings = {
                #[allow(
                    unused_mut,
                    unused_unsafe,
                    clippy::undocumented_unsafe_blocks,
                )]
                {
                    let ptr = #crate_path::__macro::destructurer_ptr(
                        &mut destructurer
                    );

                    #[allow(unreachable_code, unused_variables)]
                    if false {
                        // SAFETY: This can never be called.
                        unsafe {
                            ::core::hint::unreachable_unchecked();
                            let #test_pat =
                                #crate_path::__macro::test_destructurer(
                                    &mut destructurer,
                                );
                        }
                    }

                    #exprs
                }
            };
        });
    }
    Ok(result)
}
