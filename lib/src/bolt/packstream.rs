use bytes::Bytes;
use serde::de::{Deserialize, DeserializeOwned};

#[path = "de.rs"]
pub mod de;
#[path = "ser.rs"]
pub mod ser;

/// Parse and deserialize a packstream value from the given bytes.
pub fn from_bytes<T>(mut bytes: Bytes) -> Result<T, de::Error>
where
    T: DeserializeOwned,
{
    from_bytes_ref(&mut bytes)
}

/// Parse and deserialize a packstream value from the given bytes.
pub fn from_bytes_ref<'de, T>(bytes: &'de mut Bytes) -> Result<T, de::Error>
where
    T: Deserialize<'de>,
{
    let de = de::Deserializer::new(bytes);
    let value = T::deserialize(de)?;

    Ok(value)
}

/// Serialize and packstream encode the given value.
pub fn to_bytes<T>(value: &T) -> Result<Bytes, ser::Error>
where
    T: serde::Serialize,
{
    let mut ser = ser::Serializer::empty();
    value.serialize(&mut ser)?;

    Ok(ser.end())
}
