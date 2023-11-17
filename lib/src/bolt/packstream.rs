use bytes::Bytes;
use serde::de::DeserializeOwned;

#[path = "de.rs"]
pub mod de;

pub fn from_bytes<T>(mut bytes: Bytes) -> Result<T, de::Error>
where
    T: DeserializeOwned,
{
    let de = de::Deserializer::new(&mut bytes);
    let value = T::deserialize(de)?;

    Ok(value)
}
