use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

#[proc_macro_derive(BoltStruct)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let fields = if let syn::Data::Struct(structure) = ast.data {
        match structure.fields {
            syn::Fields::Named(syn::FieldsNamed { named, .. }) => named,
            syn::Fields::Unnamed(_) => {
                unimplemented!("BoltStruct only applicable for named fields")
            }
            syn::Fields::Unit => syn::punctuated::Punctuated::new(),
        }
    } else {
        unimplemented!("BoltStruct only applicable for structs")
    };

    let serialize_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            let #name: bytes::Bytes = self.#name.try_into()?
        }
    });

    let allocate_bytes = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            total_bytes += #name.len()
        }
    });

    let put_bytes = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            bytes.put(#name)
        }
    });

    let deserialize_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: input.clone().try_into()?
        }
    });

    let expanded = quote! {
        use std::convert::*;
        use bytes::*;

        impl std::convert::TryInto<bytes::Bytes> for #struct_name {
            type Error = crate::errors::Error;

            fn try_into(self) -> crate::errors::Result<bytes::Bytes> {
                let (marker, signature) = Self::marker();
                #(#serialize_fields;)*
                let mut total_bytes = std::mem::size_of::<u8>() + std::mem::size_of::<u8>();
                #(#allocate_bytes;)*
                let mut bytes = BytesMut::with_capacity(total_bytes);
                bytes.put_u8(marker);
                if let Some(signature) = signature {
                    bytes.put_u8(signature);
                }
                #(#put_bytes;)*
                Ok(bytes.freeze())
            }

        }

        impl #struct_name {
            pub fn can_parse(input: std::rc::Rc<std::cell::RefCell<bytes::Bytes>>) -> bool {
                match Self::marker() {
                    (marker, Some(signature)) =>  {
                        input.borrow().len() >= 2 && input.borrow()[0] == marker && input.borrow()[1] == signature
                    },
                    (marker, None) => {
                        input.borrow().len() >= 1 && input.borrow()[0] == marker
                    }
                    _ => false
                }
            }
        }

        impl std::convert::TryFrom<std::rc::Rc<std::cell::RefCell<bytes::Bytes>>> for #struct_name {
            type Error = crate::errors::Error;

            fn try_from(input: std::rc::Rc<std::cell::RefCell<bytes::Bytes>>) -> crate::errors::Result<#struct_name> {

                match Self::marker() {
                    (marker, Some(signature)) =>  {
                        input.borrow_mut().get_u8();
                        input.borrow_mut().get_u8();
                    },
                    (marker, None) => {
                        input.borrow_mut().get_u8();
                    }
                }

                Ok(#struct_name {
                    #(#deserialize_fields,)*
                })
            }
        }


    };
    expanded.into()
}
