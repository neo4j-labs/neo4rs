use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::DeriveInput;
use syn::{parse_macro_input, Attribute, LitInt, Token};

#[proc_macro_derive(BoltStruct, attributes(signature))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match derive_impl(ast) {
        Ok(data) => data,
        Err(err) => TokenStream::from(err.into_compile_error()),
    }
}

fn derive_impl(ast: DeriveInput) -> Result<TokenStream, syn::Error> {
    let struct_name = &ast.ident;

    let attr = ast
        .attrs
        .first()
        .ok_or_else(|| syn::Error::new_spanned(&ast, "Missing #[signature]"))?;

    let signature = Signature::try_from(attr)?;

    let fields = if let syn::Data::Struct(ref structure) = ast.data {
        match &structure.fields {
            syn::Fields::Named(syn::FieldsNamed { named, .. }) => Ok(Some(named)),
            syn::Fields::Unnamed(_) => Err(syn::Error::new_spanned(
                &ast,
                concat!(stringify!(#name), ": unnamed fields not supported"),
            )),
            syn::Fields::Unit => Ok(None),
        }
    } else {
        Err(syn::Error::new_spanned(
            &ast,
            concat!(stringify!(#name), ": not a struct"),
        ))
    }?;

    let can_parse = match signature.fields() {
        (marker, Some(signature)) => {
            quote! {
                input.len() >= 2 && input[0] == #marker && input[1] == #signature
            }
        }
        (marker, None) => {
            quote! {
                input.len() >= 1 && input[0] == #marker
            }
        }
    };

    let parse_signature = match signature.fields() {
        (_, Some(_)) => {
            quote! {
                input.get_u8();
                input.get_u8();
            }
        }
        (_, None) => {
            quote! {
                input.get_u8();
            }
        }
    };

    let deserialize_fields = fields.iter().flat_map(|o| o.iter()).map(|f| {
        let name = &f.ident;
        let typ = &f.ty;
        quote! {
            #name: #typ::parse(version, input)?
        }
    });

    let write_signature = match signature.fields() {
        (marker, Some(signature)) => {
            quote! {
                bytes.reserve(2);
                bytes.put_u8(#marker);
                bytes.put_u8(#signature);
            }
        }
        (marker, None) => {
            quote! {
                bytes.reserve(1);
                bytes.put_u8(#marker);
            }
        }
    };

    let serialize_fields = fields.iter().flat_map(|o| o.iter()).map(|f| {
        let name = &f.ident;
        quote! {
            self.#name.write_into(version, bytes)?
        }
    });

    let expanded = quote! {

        impl crate::types::BoltWireFormat for #struct_name {

            fn can_parse(_version: crate::Version, input: &[u8]) -> bool {
                use ::bytes::Buf;
                #can_parse
            }

            fn parse(version: crate::Version, input: &mut ::bytes::Bytes) -> crate::errors::Result<Self> {
                use ::bytes::Buf;
                #parse_signature
                Ok(#struct_name {
                    #(#deserialize_fields,)*
                })
            }

            fn write_into(&self, version: crate::Version, bytes: &mut ::bytes::BytesMut) -> crate::errors::Result<()> {
                use ::bytes::BufMut;
                #write_signature
                #(#serialize_fields;)*
                Ok(())
            }

        }
    };

    Ok(expanded.into())
}

struct Signature {
    marker: LitInt,
    signature: Option<LitInt>,
}

impl Signature {
    fn fields(&self) -> (&LitInt, Option<&LitInt>) {
        (&self.marker, self.signature.as_ref())
    }
}

impl TryFrom<&Attribute> for Signature {
    type Error = syn::Error;

    fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
        let nested = attr.parse_args_with(Punctuated::<LitInt, Token![,]>::parse_terminated)?;

        let mut iter = nested.iter();

        let marker = iter
            .next()
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    attr,
                    "Invalid signature, at least one marker byte is required",
                )
            })?
            .clone();

        let signature = iter.next().cloned();

        if let Some(item) = iter.next() {
            return Err(syn::Error::new_spanned(
                item,
                "Invalid signature, expected at most two elements.",
            ));
        }

        Ok(Self { marker, signature })
    }
}
