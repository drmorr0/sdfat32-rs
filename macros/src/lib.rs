#![feature(extend_one)]
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
};
use syn::{
    parse::{
        Parse,
        ParseStream,
        Result,
    },
    parse_macro_input,
    punctuated::Punctuated,
    Expr,
    Ident,
    LitStr,
    Token,
    Type,
};
use uuid::Uuid;

mod kw {
    syn::custom_keyword!(progmem);
}


struct ProgmemStr {
    name: Ident,
    _ty: Type,
    the_string: LitStr,
}

impl Parse for ProgmemStr {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![static]>()?;
        input.parse::<kw::progmem>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let _ty: Type = input.parse()?;
        input.parse::<Token![=]>()?;
        let the_string: LitStr = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(ProgmemStr { name, _ty, the_string })
    }
}

#[proc_macro]
pub fn progmem_str(input: TokenStream) -> TokenStream {
    let ProgmemStr { name, _ty, the_string } = parse_macro_input!(input as ProgmemStr);
    let the_string_bytes: Vec<u8> = the_string.value().into_bytes();
    let the_string_array_string = quote! {
        [ #(#the_string_bytes),* ]
    };
    let the_string_len: usize = the_string_bytes.len();

    let output = quote! {
        #[link_section = ".progmem.data"]
        static #name: [u8; #the_string_len] = #the_string_array_string;
    };

    TokenStream::from(output)
}

struct ProgmemStrWrite {
    io: Ident,
    fmt_str: LitStr,
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for ProgmemStrWrite {
    fn parse(input: ParseStream) -> Result<Self> {
        let io = input.parse()?;
        input.parse::<Token![,]>()?;
        let fmt_str = input.parse()?;
        input.parse::<Token![,]>().ok();
        let args = input.parse_terminated(Expr::parse)?;

        Ok(ProgmemStrWrite { io, fmt_str, args })
    }
}

#[proc_macro]
pub fn pm_write(input: TokenStream) -> TokenStream {
    let ProgmemStrWrite { io, fmt_str, args } = parse_macro_input!(input as ProgmemStrWrite);

    let mut args_iter = args.iter();
    let mut var_defs: Vec<TokenStream2> = vec![];
    let mut wexprs: Vec<TokenStream2> = vec![];
    for chunk in fmt_str.value().split("{}") {
        let chunk_len = chunk.len();
        if chunk_len <= 3 || chunk.trim().is_empty() {
            wexprs.push(quote! { f.write_str(#chunk)?; });
        } else {
            let ident = format_ident!("STR_{}", Uuid::new_v4().to_simple().to_string());
            var_defs.push(quote! {
                progmem_str! {
                    static progmem #ident: &'static str = #chunk;
                }
            });
            wexprs.push(quote! {
                for i in 0..#chunk_len {
                    let p_addr: *const u8 = core::ptr::addr_of!(#ident[i]);
                    let res: u8;
                    unsafe {
                        llvm_asm!("lpm" : "={r0}"(res) : "z"(p_addr));
                    }
                    f.write_char(res as char)?;
                }
            });
        }
        if let Some(a) = args_iter.next() {
            wexprs.push(quote! {
                ufmt::uDisplay::fmt(&(#a), f)?;
            });
        }
    }

    let output = quote! {
        {use ufmt::UnstableDoAsFormatter as _;

        #io.do_as_formatter(|f| {
            #(#var_defs)*
            #(#wexprs)*
            Ok(())
        })}
    };
    TokenStream::from(output)
}
