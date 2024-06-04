extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn timer_start(stream: TokenStream) -> TokenStream {
    if cfg!(debug_assertions) {
        let name = parse_macro_input!(stream as LitStr);
        quote! {
            let mut _scope_timer = crate::timer::Timer::new(#name);
        }
        .into()
    } else {
        TokenStream::new()
    }
}

#[proc_macro]
pub fn timer_next(stream: TokenStream) -> TokenStream {
    if cfg!(debug_assertions) {
        let name = parse_macro_input!(stream as LitStr);
        quote! {
            _scope_timer.step(#name);
        }
        .into()
    } else {
        TokenStream::new()
    }
}

#[proc_macro]
pub fn timer_done(_: TokenStream) -> TokenStream {
    if cfg!(debug_assertions) {
        quote! {
            std::mem::drop(_scope_timer);
        }
        .into()
    } else {
        TokenStream::new()
    }
}
