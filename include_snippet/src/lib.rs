use std::path::PathBuf;

use unsynn::{Literal, LiteralString, Parser as _, ToTokens as _, TokenStream};

#[proc_macro]
pub fn include_snippet(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    include_snippet_impl(input.into()).into()
}

fn include_snippet_impl(input: TokenStream) -> TokenStream {
    let mut it = input.into_token_iter();
    let literal = <LiteralString>::parser(&mut it).expect("Input must be a literal string");
    let file = literal.as_str();

    let root = std::env::var_os("CARGO_MANIFEST_DIR").expect("No `CARGO_MANIFEST_DIR`");
    let file = PathBuf::from(root).join(file);

    let snippet = std::fs::read_to_string(&file)
        .unwrap_or_else(|_| panic!("Could not read file: {}", file.display()));
    let mut snippet = snippet.as_str();

    let start_tag = "// snippet-start";
    let Some(start) = snippet.find(start_tag) else {
        panic!("Could not find '{start_tag}' in the file");
    };
    snippet = &snippet[start..];
    let start = snippet.find('\n').expect("No new-line after snippet-start");
    snippet = &snippet[start + 1..];

    let end_tag = "// snippet-end";
    let Some(end) = snippet.find(end_tag) else {
        panic!("Could not find '{end_tag}' in the file");
    };
    snippet = &snippet[..end];
    let end = snippet.rfind('\n').expect("No new-line before snippet-end");
    snippet = &snippet[..end];

    let snippet = Literal::string(snippet);

    unsynn::quote! { #snippet }
}
