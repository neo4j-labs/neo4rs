#[macro_export]
macro_rules! cenum {
    ($name:ident { $($variants:ident),+ $(,)? }) => {
        $crate::cenum!($name { $($variants),+ } tests);
    };

    ($name:ident { $($variants:ident),+ $(,)? } $tests:ident) => {

        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum $name {
            $(
                $variants,
            )+
        }

        impl $name {

            const VARIANTS: &[Self] = &[
                $(
                    Self::$variants,
                )+
            ];


            fn into_discriminant(self) -> u8 {
                self as u8
            }

            fn from_discriminant(discriminant: u8) -> Option<Self> {
                Self::VARIANTS.get(usize::from(discriminant)).copied()
            }
        }


        const _: () = {


            pub struct TheDeserializer<'de> {
                value: $name,
                _lifetime: ::std::marker::PhantomData<&'de ()>,
            }

            impl<'de> TheDeserializer<'de> {
                fn new(value: $name) -> Self {
                    Self {
                        value,
                        _lifetime: ::std::marker::PhantomData,
                    }
                }
            }

            impl<'de> ::serde::de::IntoDeserializer<'de, $crate::DeError> for $name {
                type Deserializer = TheDeserializer<'de>;

                fn into_deserializer(self) -> Self::Deserializer {
                    Self::Deserializer::new(self)
                }
            }

            impl<'de> ::serde::de::Deserializer<'de> for TheDeserializer<'de> {
                type Error = $crate::DeError;

                fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where
                    V: ::serde::de::Visitor<'de>,
                {
                    visitor.visit_u8(self.value.into_discriminant())
                }

                ::serde::forward_to_deserialize_any! {
                    char str string bytes byte_buf option unit unit_struct newtype_struct
                    seq tuple tuple_struct map struct enum identifier ignored_any
                    bool i8 i16 i32 i64 i128 u8 u16 u32 u64 f32 f64
                }
            }

            impl<'de> ::serde::Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> Result<$name, D::Error>
                where
                    D: ::serde::de::Deserializer<'de>,
                {
                    struct TheVisitor;

                    impl<'de> ::serde::de::Visitor<'de> for TheVisitor {
                        type Value = $name;

                        fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                            // panic!("this should never be called");
                            formatter.write_str(concat!("a valid ", stringify!($name), " discriminant"))
                        }

                        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                        where
                            E: ::serde::de::Error,
                        {
                            match u8::try_from(v).ok().and_then(<$name>::from_discriminant) {
                                Some(kind) => Ok(kind),
                                None => Err(E::invalid_value(::serde::de::Unexpected::Signed(v), &self)),
                            }
                        }

                        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                        where
                            E: ::serde::de::Error,
                        {
                            match u8::try_from(v).ok().and_then(<$name>::from_discriminant) {
                                Some(kind) => Ok(kind),
                                None => Err(E::invalid_value(::serde::de::Unexpected::Unsigned(v), &self)),
                            }
                        }
                    }

                    deserializer.deserialize_u8(TheVisitor)
                }
            }

            impl ::serde::Serialize for $name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::ser::Serializer,
                {
                    serializer.serialize_u8(self.into_discriminant())
                }
            }
        };


        #[cfg(test)]
        mod $tests {
            use super::*;

            #[allow(unused_assignments)]
            #[test]
            fn serialize() {
                let mut expected = 0;
                $(

                    let result = serde_json::to_value(<$name>::$variants).unwrap();
                    assert_eq!(result, serde_json::json!(expected));
                    expected += 1;

                )+
            }

            #[allow(unused_assignments)]
            #[test]
            fn deserialize() {
                let mut value = 0;
                $(

                    let result = serde_json::from_value::<$name>(serde_json::json!(value)).unwrap();
                    assert_eq!(result, <$name>::$variants);
                    value += 1;

                )+
            }
        }
    }
}
