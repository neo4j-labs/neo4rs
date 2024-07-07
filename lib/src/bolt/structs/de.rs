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
