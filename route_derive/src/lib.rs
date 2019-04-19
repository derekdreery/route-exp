mod format;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use synstructure::decl_derive;

#[derive(Debug)]
struct Error(TokenStream);

impl Error {
    fn new(span: Span, message: &str) -> Error {
        Error(quote_spanned! { span =>
            compile_error!(#message);
        })
    }

    fn into_tokens(self) -> TokenStream {
        self.0
    }
}

decl_derive!([Routes, attributes(route)] => routes_derive);

fn routes_derive(s: synstructure::Structure) -> TokenStream {
    match routes_derive_impl(s) {
        Err(e) => e.into_tokens(),
        Ok(tokens) => tokens,
    }
}

fn routes_derive_impl(s: synstructure::Structure) -> Result<TokenStream, Error> {
    let ident = &s.ast().ident;

    let fmt_variants = s.each_variant(|v| {
        let variant_name = v.ast().ident.to_string();
        let mut v = v.clone();
        let route_tok = match route_attr(v.ast()) {
            Some(s) => s,
            None => return quote!(),
        };
        let route = route_tok.value();
        // todo we could process a stream of tokens, but there's probably no point for perf.
        let parts = {
            let mut lexer = format::Lexer::new(&route);
            let mut tokens = Vec::new();
            while let Some(tok) = lexer.next() {
                tokens.push(tok);
            }
            tokens
        };
        let placeholders = parts
            .iter()
            .filter_map(|p| match p {
                format::Token::Placeholder(name) => Some(*name),
                _ => None,
            })
            .collect::<Vec<_>>();
        // TODO: the unwrap below is because we only support named fields, the None case is for
        // positional fields. We should support these.
        //
        // Check all fields are present in the url
        let binding_names: Vec<_> = v
            .bindings()
            .iter()
            .map(|bi| bi.ast().ident.as_ref().map(ToString::to_string).unwrap())
            .collect();
        for binding_name in binding_names.iter() {
            if !placeholders.contains(&binding_name.as_str()) {
                panic!(
                    r#"In "{}::{}", the field "{}" is not in the url"#,
                    ident, variant_name, binding_name
                );
            }
        }
        // Check all placeholders are fields on the variant
        for ph in placeholders.iter() {
            if !binding_names.contains(&ph.to_string()) {
                panic!(
                    r#"In "{}::{}", the url placeholder "{}" is not in the enum variant"#,
                    ident, variant_name, ph
                );
            }
        }
        // now we know we've got matching fields
        let mut toks = TokenStream::new();
        for token in parts {
            toks.extend(match token {
                format::Token::Literal(s) => {
                    let literal = syn::LitStr::new(s, Span::call_site());
                    quote! {f.write_str(#literal)?;}
                }
                format::Token::Placeholder(p) => {
                    let ident = &v
                        .bindings()
                        .iter()
                        .find(|bi| bi.ast().ident.as_ref().map(ToString::to_string).unwrap() == p)
                        .unwrap()
                        .binding;
                    quote! {f.write_fmt(format_args!("{}", #ident))?;}
                }
            });
        }

        quote!({ #toks Ok(()) })
    });

    Ok(s.gen_impl(quote! {
        extern crate route;

        #[repr(transparent)]
        pub struct UrlDisplay(#ident);

        impl std::fmt::Display for UrlDisplay {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self.0 {
                    #fmt_variants
                }
            }
        }

        gen impl route::Routes for @Self {
            type UrlDisplay = UrlDisplay;
            fn url(&self) -> &UrlDisplay {
                unsafe { std::mem::transmute(self) }
            }
        }
    }))
}

fn route_attr(root: synstructure::VariantAst<'_>) -> Option<syn::LitStr> {
    let attr = root.attrs.iter().find(|a| a.path.is_ident("route"))?;
    let first_tree = only(attr.tts.clone()).expect("route takes one string literal parameter1");
    let lit_group = match first_tree {
        proc_macro2::TokenTree::Group(g) => g,
        _ => panic!("route takes one string literal parameter3"),
    };
    let first_in_group =
        only(lit_group.stream()).expect("route takes one string literal parameter1");
    let lit = match first_in_group {
        TokenTree::Literal(l) => syn::Lit::new(l),
        _ => panic!("route takes one string literal parameter4"),
    };
    Some(match lit {
        syn::Lit::Str(ls) => ls,
        _ => panic!("route takes one string literal parameter4"),
    })
}

// Gets a single value from an interator. Fails if there are 0 or 2+ items.
fn only<T, I>(input: I) -> Option<T>
where
    I: IntoIterator<Item = T>,
{
    let mut iter = input.into_iter();
    let first = iter.next()?;
    if iter.next().is_some() {
        return None;
    }
    Some(first)
}
