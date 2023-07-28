extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn instrument(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut function = parse_macro_input!(input as ItemFn);

    // Get the function's name
    let fname = &function.sig.ident;

    // The original function block
    let original_block = &function.block;

    // Construct a new block with our instrument calls and the original function body
    function.block = syn::parse2(quote!({
        span_enter(stringify!(#fname));
        let result = #original_block;
        span_exit();
        result
    }))
    .unwrap();

    // Return the new function
    TokenStream::from(quote!(#function))
}
