use std::{collections::HashMap, fmt, marker::PhantomData};

use serde::{
    de::{
        DeserializeSeed, EnumAccess, Error, IgnoredAny, MapAccess, SeqAccess, VariantAccess as _,
        Visitor,
    },
    Deserialize, Deserializer,
};

pub(super) struct Keys<'de>(pub(super) Vec<&'de str>);

impl<'de> Deserialize<'de> for Keys<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Vis<'de>(PhantomData<&'de ()>);

        impl<'de> Visitor<'de> for Vis<'de> {
            type Value = Keys<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("A map of properties")
            }

            fn visit_map<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut keys = Vec::with_capacity(seq.size_hint().unwrap_or(1));
                while let Some(key) = seq.next_key()? {
                    keys.push(key);
                    let _ignore = seq.next_value::<IgnoredAny>()?;
                }
                Ok(Keys(keys))
            }
        }

        deserializer.deserialize_map(Vis(PhantomData))
    }
}

pub(super) struct Single<'a, T>(&'a str, PhantomData<T>);

impl<'a, T> Single<'a, T> {
    pub(super) fn new(key: &'a str) -> Self {
        Self(key, PhantomData)
    }
}

impl<'a, 'de, T: Deserialize<'de> + 'de> DeserializeSeed<'de> for Single<'a, T> {
    type Value = Option<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Key {
            Found,
            NotFound,
        }

        struct Filter<'a>(&'a str);

        impl<'a, 'de> Visitor<'de> for Filter<'a> {
            type Value = Key;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("A string-like identifier of a property key")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(if v == self.0 {
                    Key::Found
                } else {
                    Key::NotFound
                })
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_str(&v)
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_str(v)
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(if self.0.as_bytes() == v {
                    Key::Found
                } else {
                    Key::NotFound
                })
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_bytes(&v)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Key::NotFound)
            }
        }

        impl<'a, 'de> DeserializeSeed<'de> for Filter<'a> {
            type Value = Key;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_identifier(Filter(self.0))
            }
        }

        struct Vis<'a, 'de, T>(&'a str, PhantomData<&'de T>);

        impl<'a, 'de, T: Deserialize<'de> + 'de> Visitor<'de> for Vis<'a, 'de, T> {
            type Value = Option<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("A map of properties")
            }

            fn visit_map<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut value = None::<T>;
                while let Some(key) = seq.next_key_seed(Filter(self.0))? {
                    if matches!(key, Key::Found) {
                        if value.is_some() {
                            return Err(A::Error::custom(format!("duplicate field `{}`", self.0)));
                        }
                        value = seq.next_value()?;
                    } else {
                        let _ignore = seq.next_value::<IgnoredAny>()?;
                    }
                }
                Ok(value)
            }
        }

        deserializer.deserialize_map(Vis(self.0, PhantomData))
    }
}

macro_rules! count_tts {
    () => { 0 };
    ($odd:tt $($a:tt $b:tt)*) => { ($crate::bolt::structs::de::count_tts!($($a)*) << 1) | 1 };
    ($($a:tt $even:tt)*) => { $crate::bolt::structs::de::count_tts!($($a)*) << 1 };
}

macro_rules! impl_visitor_ref {
    ($typ:ident <'de> ($($name:ident),+ $(,)? $([$($opt_name:ident),+ $(,)?])?) == $tag:literal) => {
        impl<'de> $typ<'de> {
            pub(super) fn visitor() -> impl ::serde::de::Visitor<'de, Value = Self> {
                struct Vis;

                impl<'de> ::serde::de::Visitor<'de> for Vis {
                    type Value = $typ<'de>;

                    $crate::bolt::structs::de::impl_visitor!(@__inner: $typ ($($name),+ $([$($opt_name),+])?) == $tag);
                }

                Vis
            }
        }
    };
}

macro_rules! impl_visitor {
    ($typ:ident ($($name:ident),+ $(,)? $([$($opt_name:ident),+ $(,)?])?) == $tag:literal) => {
        impl $typ {
            pub(super) fn visitor<'de>() -> impl ::serde::de::Visitor<'de, Value = Self> {
                struct Vis<'de>(::std::marker::PhantomData<&'de ()>);

                impl<'de> ::serde::de::Visitor<'de> for Vis<'de> {
                    type Value = $typ;

                    $crate::bolt::structs::de::impl_visitor!(@__inner: $typ ($($name),+ $([$($opt_name),+])?) == $tag);
                }

                Vis(::std::marker::PhantomData)
            }
        }
    };

    (@__inner: $typ:ident ($($name:ident),+ $(,)? $([$($opt_name:ident),+ $(,)?])? $({$($def_name:ident),+ $(,)?})?) == $tag:literal) => {
        fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(formatter, concat!("a valid Bolt ", stringify!($typ), " struct (tag '{}' or 0x{:02X})"), $tag, $tag)
        }

        fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
        where
            A: ::serde::de::EnumAccess<'de>,
        {
            let (tag, data) = ::serde::de::EnumAccess::variant::<u8>(data)?;
            if tag != $tag {
                return Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Other(&format!("struct with tag {:02X}", tag)),
                    &self,
                ));
            }
            ::serde::de::VariantAccess::struct_variant(data, &[], self)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: ::serde::de::SeqAccess<'de>,
        {

            let req_len = $crate::bolt::structs::de::count_tts!($($name)+);
            let max_len = req_len + ($crate::bolt::structs::de::count_tts!($($($opt_name)+)?));

            let len = ::serde::de::SeqAccess::size_hint(&seq).unwrap_or_default();
            if len < req_len || len > max_len {
                return Err(::serde::de::Error::invalid_length(
                    len,
                    &format!("a sequence of length {} to {}", req_len, max_len).as_str(),
                ));
            }

            $(
                let $name = ::serde::de::SeqAccess::next_element(&mut seq)?
                    .ok_or_else(|| ::serde::de::Error::missing_field(stringify!($name)))?;
            )+

            $(
                $(
                    let $opt_name = seq.next_element()?;
                )+
            )?

            Ok($typ {
                $($name,)+
                $($($opt_name,)+)?
                $($($def_name: ::std::default::Default::default(),)+)?
            })
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: ::serde::de::MapAccess<'de>,
        {
            let tag = ::serde::de::MapAccess::next_key::<u8>(&mut map)?;
            match tag {
                Some($tag) => {},
                Some(tag) => return Err(serde::de::Error::invalid_type(
                    serde::de::Unexpected::Other(&format!("struct with tag {:02X}", tag)),
                    &self,
                )),
                None => return Err(serde::de::Error::missing_field("tag")),
            };

            let this = ::serde::de::MapAccess::next_value::<Self::Value>(&mut map)?;
            Ok(this)
        }
    };
}

pub(crate) use count_tts;
pub(crate) use impl_visitor;
pub(crate) use impl_visitor_ref;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys() {
        let json = r#"{
            "name": "Alice",
            "age": 42,
            "email": "foo@bar.com"
        }"#;

        let keys = serde_json::from_str::<Keys>(json).unwrap();

        assert_eq!(keys.0, vec!["name", "age", "email"]);
    }

    #[test]
    fn single() {
        let json = r#"{
            "name": "Alice",
            "age": 42,
            "email": "foo@bar.com"
        }"#;

        let name = Single::<&str>::new("name")
            .deserialize(&mut serde_json::Deserializer::from_str(json))
            .unwrap();
        let age = Single::<u64>::new("age")
            .deserialize(&mut serde_json::Deserializer::from_str(json))
            .unwrap();
        let email = Single::<String>::new("email")
            .deserialize(&mut serde_json::Deserializer::from_str(json))
            .unwrap();
        let missing = Single::<bool>::new("missing")
            .deserialize(&mut serde_json::Deserializer::from_str(json))
            .unwrap();

        assert_eq!(name, Some("Alice"));
        assert_eq!(age, Some(42));
        assert_eq!(email, Some("foo@bar.com".to_owned()));
        assert_eq!(missing, None);
    }
}
