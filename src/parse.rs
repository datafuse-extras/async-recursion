use std::fmt::Formatter;
use proc_macro2::Span;
use syn::{parse::{Error, Parse, ParseStream, Result}, token::Question, ItemFn, Token, Attribute, Type};
use syn::spanned::Spanned;

pub struct AsyncItem(pub ItemFn);

impl Parse for AsyncItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let item: ItemFn = input.parse()?;

        // Check that this is an async function
        if item.sig.asyncness.is_none() {
            return Err(Error::new(Span::call_site(), "expected an async function"));
        }

        Ok(AsyncItem(item))
    }
}

pub struct RecursionArgs {
    pub send_bound: bool,
    pub sync_bound: bool,
    pub attrs: Vec<Attribute>,
}

/// Custom keywords for parser
mod kw {
    syn::custom_keyword!(Send);
    syn::custom_keyword!(Sync);
}

enum Arg {
    NotSend,
    Sync,
    Attrs(Vec<Attribute>),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![?]) {
            input.parse::<Question>()?;
            input.parse::<kw::Send>()?;
            Ok(Arg::NotSend)
        } else if input.peek(Token![#]) {
            let attrs = input.call(Attribute::parse_outer)?;
            Ok(Arg::Attrs(attrs))
        } else {
            input.parse::<kw::Sync>()?;
            Ok(Arg::Sync)
        }
    }
}

impl Parse for RecursionArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut send_bound: bool = true;
        let mut sync_bound: bool = false;

        let args_parsed: Vec<Arg> =
            syn::punctuated::Punctuated::<Arg, syn::Token![,]>::parse_terminated(input)
                .map_err(|e| input.error(format!("failed to parse macro arguments: {e}")))?
                .into_iter()
                .collect();

        let mut attrs = vec![];
        for arg in args_parsed {
            match arg {
                Arg::NotSend => {
                    if !send_bound {
                        return Err(Error::new(
                            Span::call_site(),
                            "received duplicate argument: `?Send`",
                        ));
                    }

                    send_bound = false;
                }
                Arg::Sync => {
                    if sync_bound {
                        return Err(Error::new(
                            Span::call_site(),
                            "received duplicate argument: `Sync`",
                        ));
                    }
                    sync_bound = true
                }
                Arg::Attrs(v) => {
                    attrs.extend(v)
                }
            }
        }

        Ok(Self {
            attrs,
            send_bound,
            sync_bound,
        })
    }
}
