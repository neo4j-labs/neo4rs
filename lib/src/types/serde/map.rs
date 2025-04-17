use serde::{
    de::{
        value::{MapDeserializer, SeqDeserializer},
        DeserializeSeed, EnumAccess, Error as _, IntoDeserializer, Unexpected, VariantAccess,
        Visitor,
    },
    forward_to_deserialize_any, Deserialize, Deserializer,
};

use crate::{BoltMap, BoltString, BoltType};

use super::DeError;

pub struct BoltMapDeserializer<'de> {
    map: &'de BoltMap,
}

impl<'de> BoltMapDeserializer<'de> {
    pub fn new(map: &'de BoltMap) -> Self {
        BoltMapDeserializer { map }
    }
}

impl<'de> serde::Deserializer<'de> for BoltMapDeserializer<'de> {
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut deserializer = MapDeserializer::new(self.map.value.iter());
        let map = visitor.visit_map(&mut deserializer)?;
        deserializer.end()?;
        Ok(map)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut iter = self.map.value.iter();
        let (variant, value) = match iter.next() {
            Some(v) => v,
            None => {
                return Err(DeError::invalid_value(
                    Unexpected::Map,
                    &"map with a single key",
                ));
            }
        };
        if iter.next().is_some() {
            return Err(serde::de::Error::invalid_value(
                Unexpected::Map,
                &"map with a single key",
            ));
        }

        visitor.visit_enum(EnumDeserializer {
            variant,
            value: &value,
        })
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier
    }
}

struct EnumDeserializer<'de> {
    variant: &'de BoltString,
    value: &'de BoltType,
}

impl<'de> EnumAccess<'de> for EnumDeserializer<'de> {
    type Error = DeError;
    type Variant = VariantDeserializer<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer<'de>), DeError>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer<'de> {
    value: &'de BoltType,
}

impl<'de> VariantAccess<'de> for VariantDeserializer<'de> {
    type Error = DeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            BoltType::Null(_) => Ok(()),
            _ => Deserialize::deserialize(self.value.into_deserializer()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            BoltType::Null(_) => Err(DeError::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
            _ => seed.deserialize(self.value.into_deserializer()),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            BoltType::List(v) => {
                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                    let mut deserializer = SeqDeserializer::new(v.value.iter());
                    let seq = visitor.visit_seq(&mut deserializer)?;
                    deserializer.end()?;
                    Ok(seq)
                }
            }
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            BoltType::Map(v) => MapDeserializer::new(v.value.iter()).deserialize_any(visitor),
            _ => Err(DeError::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::{BoltMap, BoltNull, BoltString, BoltType};

    #[test]
    fn deserialize_externally_tagged_enum() {
        #[derive(Deserialize, Debug, PartialEq)]
        pub enum TestEnum {
            Variant1 { value: String },
            Variant2 { value: String },
        }

        let content = [(BoltString::from("value"), BoltType::from("test"))]
            .into_iter()
            .collect::<BoltMap>();
        let value = [(BoltString::from("Variant1"), BoltType::Map(content))]
            .into_iter()
            .collect::<BoltMap>();
        let actual = value.to::<TestEnum>().unwrap();
        assert_eq!(
            actual,
            TestEnum::Variant1 {
                value: "test".to_string()
            }
        );
    }

    #[test]
    fn deserialize_adjacently_tagged_enum() {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(tag = "type", content = "content")]
        pub enum TestEnum {
            Variant1 { value: String },
            Variant2 { value: String },
        }
        let content = [(BoltString::from("value"), BoltType::from("test"))]
            .into_iter()
            .collect::<BoltMap>();
        let value = [
            (BoltString::from("type"), BoltType::from("Variant1")),
            (BoltString::from("content"), BoltType::Map(content)),
        ]
        .into_iter()
        .collect::<BoltMap>();
        let actual = value.to::<TestEnum>().unwrap();
        assert_eq!(
            actual,
            TestEnum::Variant1 {
                value: "test".to_string()
            }
        );
    }

    #[test]
    fn deserialize_untagged_enum() {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(untagged)]
        pub enum TestEnum {
            Variant1 { val1: String },
            Variant2 { val2: String },
        }
        let value = [(BoltString::from("val1"), BoltType::from("test"))]
            .into_iter()
            .collect::<BoltMap>();
        let actual = value.to::<TestEnum>().unwrap();
        assert_eq!(
            actual,
            TestEnum::Variant1 {
                val1: "test".to_string()
            }
        );
    }

    #[test]
    fn deserialize_internally_tagged_enum() {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(tag = "type")]
        pub enum TestEnum {
            Variant1 { value: String },
            Variant2 { value: String },
        }
        let value = [
            (BoltString::from("type"), BoltType::from("Variant1")),
            (BoltString::from("value"), BoltType::from("test")),
        ]
        .into_iter()
        .collect::<BoltMap>();
        let actual = value.to::<TestEnum>().unwrap();
        assert_eq!(
            actual,
            TestEnum::Variant1 {
                value: "test".to_string()
            }
        );
    }

    #[test]
    fn deserialize_newtype_enum() {
        #[derive(Deserialize, Debug, PartialEq)]
        pub enum TestEnum {
            Variant1(String),
            Variant2(String),
        }

        let value = [(BoltString::from("Variant1"), "test".into())]
            .into_iter()
            .collect::<BoltMap>();
        let actual = value.to::<TestEnum>().unwrap();
        assert_eq!(actual, TestEnum::Variant1("test".to_string()));
    }

    #[test]
    fn deserialize_tuple_enum() {
        #[derive(Deserialize, Debug, PartialEq)]
        pub enum TestEnum {
            Variant1(String, u64),
            Variant2(String),
        }

        let content: Vec<BoltType> = vec!["test".into(), 10.into()];
        let value = [(BoltString::from("Variant1"), content.into())]
            .into_iter()
            .collect::<BoltMap>();
        let actual = value.to::<TestEnum>().unwrap();
        assert_eq!(actual, TestEnum::Variant1("test".to_string(), 10));
    }

    #[test]
    fn deserialize_unit_enum() {
        #[derive(Deserialize, Debug, PartialEq)]
        pub enum TestEnum {
            Variant1,
            Variant2,
        }

        let value = [(BoltString::from("Variant1"), BoltType::Null(BoltNull))]
            .into_iter()
            .collect::<BoltMap>();
        let actual = value.to::<TestEnum>().unwrap();
        assert_eq!(actual, TestEnum::Variant1);
    }
}
