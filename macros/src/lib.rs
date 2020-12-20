use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

#[proc_macro_derive(BoltStruct)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let empty_fields = syn::punctuated::Punctuated::new();
    eprint!("{:#?}", ast);

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        &empty_fields
    };

    let fields_bytes = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            let #name: bytes::Bytes = self.#name.try_into()?
        }
    });

    let fields_lengths = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            total_bytes += #name.len()
        }
    });

    let fields_put_bytes = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            bytes.put(#name)
        }
    });

    let expanded = quote! {
        impl std::convert::TryInto<bytes::Bytes> for #name {
            type Error = crate::errors::Error;

            fn try_into(self) -> crate::errors::Result<bytes::Bytes> {
                #(#fields_bytes;)*
                let mut total_bytes = std::mem::size_of::<u8>() + std::mem::size_of::<u8>();
                #(#fields_lengths;)*
                let mut bytes = BytesMut::with_capacity(total_bytes);
                bytes.put_u8(MARKER);
                bytes.put_u8(SIGNATURE);
                #(#fields_put_bytes;)*
                Ok(bytes.freeze())
            }

        }
    };
    expanded.into()
}
