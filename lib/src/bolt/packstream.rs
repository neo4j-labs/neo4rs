use bytes::Bytes;
use serde::de::DeserializeOwned;

#[path = "de.rs"]
pub mod de;
#[path = "ser.rs"]
pub mod ser;

pub fn from_bytes<T>(mut bytes: Bytes) -> Result<T, de::Error>
where
    T: DeserializeOwned,
{
    let de = de::Deserializer::new(&mut bytes);
    let value = T::deserialize(de)?;

    Ok(value)
}

pub fn to_bytes<T>(value: &T) -> Result<Bytes, ser::Error>
where
    T: serde::Serialize,
{
    let mut ser = ser::Serializer::empty();
    value.serialize(&mut ser)?;

    Ok(ser.end())
}
