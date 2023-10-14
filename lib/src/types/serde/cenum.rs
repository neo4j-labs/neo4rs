#[macro_export]
macro_rules! cenum {
    ($name:ident { $($variants:ident),+ $(,)? }) => {

        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum $name {
            $(
                $variants,
            )+
        }

        impl $name {

            pub const VARIANTS: &'static [Self] = &[
                $(
                    Self::$variants,
                )+
            ];

            paste::paste! {
                #[allow(unused)]
                pub const NAMES: &'static [&'static str] = &[
                    $(
                        stringify!([<$variants:snake:lower>])
                    ),+
                ];
            }

            fn into_discriminant(self) -> u8 {
                self as u8
            }

            fn from_discriminant(discriminant: u8) -> Option<Self> {
                Self::VARIANTS.get(usize::from(discriminant)).copied()
            }

            fn from_str(v: &str) -> Option<Self> {
                paste::paste! {
                    match v {
                        $(
                            stringify!([<$variants:snake:lower>]) => Some(<$name>::$variants),
                        )+
                        _ => None,
                    }
                }
            }

            #[allow(unused)]
            pub const fn name(self) -> &'static str {
                paste::paste! {
                    match self {
                        $(
                            Self::$variants => stringify!([<$variants:snake:lower>]),
                        )+
                    }
                }
            }
        }


        const _: () = {


            pub struct TheDeserializer<'de, E> {
                value: $name,
                _lifetime: ::std::marker::PhantomData<&'de ()>,
                _error: ::std::marker::PhantomData<E>,
            }

            impl<'de, E: ::serde::de::Error> TheDeserializer<'de, E> {
                fn new(value: $name) -> Self {
                    Self {
                        value,
                        _lifetime: ::std::marker::PhantomData,
                        _error: ::std::marker::PhantomData,
                    }
                }
            }

            impl<'de, E: ::serde::de::Error> ::serde::de::IntoDeserializer<'de, E> for $name {
                type Deserializer = TheDeserializer<'de, E>;

                fn into_deserializer(self) -> Self::Deserializer {
                    Self::Deserializer::new(self)
                }
            }

            impl<'de, E: ::serde::de::Error> ::serde::de::Deserializer<'de> for TheDeserializer<'de, E> {
                type Error = E;

                fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where
                    V: ::serde::de::Visitor<'de>,
                {
                    visitor.visit_u8(self.value.into_discriminant())
                }

                fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where
                    V: ::serde::de::Visitor<'de>,
                {
                    visitor.visit_str(self.value.name())
                }

                ::serde::forward_to_deserialize_any! {
                    char str string bytes byte_buf option unit unit_struct seq map
                    tuple tuple_struct struct enum newtype_struct ignored_any
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
                            formatter.write_str(concat!("a valid ", stringify!($name), " identifier"))
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

                        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                        where
                            E: ::serde::de::Error,
                        {
                            paste::paste! {
                                match <$name>::from_str(v) {
                                    Some(kind) => Ok(kind),
                                    _ => Err(E::unknown_variant(v, &[
                                        $(
                                            stringify!([<$variants:snake:lower>])
                                        ),+
                                    ])),
                                }
                            }
                        }
                    }

                    deserializer.deserialize_any(TheVisitor)
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


        paste::paste! {
            #[cfg(test)]
            mod [< $name:snake:lower _tests>] {
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

                #[allow(unused_assignments)]
                #[test]
                fn deserialize_str() {
                    $(

                        let result = serde_json::from_value::<$name>(serde_json::json!(<$name>::$variants.name())).unwrap();
                        assert_eq!(result, <$name>::$variants);

                    )+
                }
            }
        }
    }
}
