use std::fmt::Debug;

use ::serde::ser::{
    Error as SerError, Serialize, SerializeMap, SerializeSeq, SerializeStruct,
    SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
    Serializer,
};

#[derive(Debug, PartialEq, Eq)]
pub struct AsMap<'a, T: ?Sized>(pub &'a T);

impl<'a, T: ?Sized> Copy for AsMap<'a, T> {}

impl<'a, T: ?Sized> Clone for AsMap<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: ?Sized + Serialize> Serialize for AsMap<'a, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        as_map(serializer, self.0)
    }
}

pub fn as_map<S, T>(serializer: S, value: &T) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ?Sized + Serialize,
{
    let as_map = AsMapSerializer::new(serializer)?;
    value.serialize(as_map)
}

pub(crate) struct AsMapSerializer<S> {
    inner: S,
}

impl<S: Serializer> AsMapSerializer<S> {
    pub(crate) fn new(inner: S) -> Result<Self, S::Error> {
        Ok(Self { inner })
    }

    pub(crate) fn into_inner(self) -> S {
        self.inner
    }

    fn inner(&mut self) -> &mut S {
        &mut self.inner
    }
}

impl<S: Serializer> Serializer for AsMapSerializer<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    type SerializeSeq = OuterMapSerializer<S::SerializeMap>;
    type SerializeTuple = OuterMapSerializer<S::SerializeMap>;
    type SerializeTupleStruct = OuterMapSerializer<S::SerializeMap>;
    type SerializeTupleVariant = OuterMapSerializer<S::SerializeMap>;
    type SerializeMap = ForwardingSerializer<S::SerializeMap>;
    type SerializeStruct = ForwardingSerializer<S::SerializeStruct>;
    type SerializeStructVariant = ForwardingSerializer<S::SerializeStructVariant>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: bool"))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: i8"))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: i16"))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: i32"))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: i64"))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: u8"))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: u16"))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: u32"))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: u64"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: f64"))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: char"))
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: str"))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Self::Error::custom("not a valid map entry: bytes"))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_map(Some(0))?.end()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_map(Some(0))?.end()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_map(Some(0))?.end()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.inner.serialize_map(Some(0))?.end()
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let is_human_readable = self.inner.is_human_readable();
        let inner = self.inner.serialize_map(None)?;
        let inner = InnerMapSerializer::new(inner, is_human_readable);
        Ok(OuterMapSerializer { inner })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let is_human_readable = self.inner.is_human_readable();
        let inner = self.inner.serialize_map(None)?;
        let inner = InnerMapSerializer::new(inner, is_human_readable);
        Ok(OuterMapSerializer { inner })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        let is_human_readable = self.inner.is_human_readable();
        let inner = self.inner.serialize_map(None)?;
        let inner = InnerMapSerializer::new(inner, is_human_readable);
        Ok(OuterMapSerializer { inner })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let is_human_readable = self.inner.is_human_readable();
        let inner = self.inner.serialize_map(None)?;
        let inner = InnerMapSerializer::new(inner, is_human_readable);
        Ok(OuterMapSerializer { inner })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let inner = self.inner.serialize_map(len)?;
        Ok(ForwardingSerializer { inner })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let inner = self.inner.serialize_struct(name, len)?;
        Ok(ForwardingSerializer { inner })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let inner = self
            .inner
            .serialize_struct_variant(name, variant_index, variant, len)?;
        Ok(ForwardingSerializer { inner })
    }

    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }
}

pub(crate) struct OuterMapSerializer<S> {
    inner: InnerMapSerializer<S>,
}

impl<S: SerializeMap> SerializeSeq for OuterMapSerializer<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        SerializeSeq::serialize_element(&mut &mut self.inner, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<S: SerializeMap> SerializeTuple for OuterMapSerializer<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        SerializeTuple::serialize_element(&mut &mut self.inner, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<S: SerializeMap> SerializeTupleStruct for OuterMapSerializer<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        SerializeTupleStruct::serialize_field(&mut &mut self.inner, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<S: SerializeMap> SerializeTupleVariant for OuterMapSerializer<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        SerializeTupleVariant::serialize_field(&mut &mut self.inner, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

pub(crate) struct InnerMapSerializer<S> {
    inner: S,
    is_human_readable: bool,
    expect_key: bool,
}

impl<S: SerializeMap> InnerMapSerializer<S> {
    pub(crate) fn new(inner: S, is_human_readable: bool) -> Self {
        Self {
            inner,
            is_human_readable,
            expect_key: true,
        }
    }

    pub(crate) fn into_inner(self) -> S {
        self.inner
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        self.inner.end()
    }

    fn inner(&mut self) -> &mut S {
        &mut self.inner
    }
}

impl<S: SerializeMap> InnerMapSerializer<S> {
    fn either<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), S::Error> {
        if self.expect_key {
            self.expect_key = false;
            self.inner.serialize_key(value)?;
        } else {
            self.expect_key = true;
            self.inner.serialize_value(value)?;
        }
        Ok(())
    }

    fn key<T>(&mut self, key: &T) -> Result<(), S::Error>
    where
        T: Serialize + ?Sized,
    {
        if !self.expect_key {
            return Err(SerError::custom("MapValueNotString"));
        }

        self.expect_key = false;
        self.inner.serialize_key(key)?;
        Ok(())
    }

    fn value<T>(&mut self, value: &T) -> Result<(), S::Error>
    where
        T: Serialize + Debug + ?Sized,
    {
        if self.expect_key {
            return Err(SerError::custom(format!("MapKeyNotString: {:?}", value)));
        }

        self.expect_key = true;
        self.inner.serialize_value(value)?;
        Ok(())
    }
}

impl<'a, S: SerializeMap> Serializer for &'a mut InnerMapSerializer<S> {
    type Ok = ();
    type Error = S::Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.value(&v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.either(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.value(v)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        eprintln!("DEBUGPRINT[1]: map.rs:525 (after fn serialize_map(self, _len: Option<usiz…)");
        eprintln!("DEBUGPRINT[2]: map.rs:525: _len={:#?}", _len);
        let len = _len.ok_or_else(|| SerError::custom("inner map with unknown length"))?;
        self.inner.serialize_entry("$$__MAP__$$", &len)?;
        self.expect_key = true;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        eprintln!("DEBUGPRINT[3]: map.rs:534 (after ) -> Result<Self::SerializeStruct, Self:…)");
        self.serialize_map(Some(_len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        eprintln!("DEBUGPRINT[4]: map.rs:545 (after ) -> Result<Self::SerializeStructVariant…)");
        self.serialize_map(Some(_len))
    }

    fn is_human_readable(&self) -> bool {
        self.is_human_readable
    }
}

// impl<S: SerializeMap> SerializeSeq for InnerMapSerializer<S> {
//     type Ok = ();
//     type Error = S::Error;

//     fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
//     where
//         T: Serialize,
//     {
//         value.serialize(self)?;
//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(())
//     }
// }

impl<'a, S: SerializeMap> SerializeSeq for &'a mut InnerMapSerializer<S> {
    type Ok = ();
    type Error = S::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// impl<S: SerializeMap> SerializeTuple for InnerMapSerializer<S> {
//     type Ok = ();
//     type Error = S::Error;

//     fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
//     where
//         T: Serialize,
//     {
//         value.serialize(self)?;
//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(())
//     }
// }

impl<'a, S: SerializeMap> SerializeTuple for &'a mut InnerMapSerializer<S> {
    type Ok = ();
    type Error = S::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// impl<S: SerializeMap> SerializeTupleStruct for InnerMapSerializer<S> {
//     type Ok = ();
//     type Error = S::Error;

//     fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
//     where
//         T: Serialize,
//     {
//         value.serialize(self)?;
//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(())
//     }
// }

impl<'a, S: SerializeMap> SerializeTupleStruct for &'a mut InnerMapSerializer<S> {
    type Ok = ();
    type Error = S::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// impl<S: SerializeMap> SerializeMap for InnerMapSerializer<S> {
//     type Ok = ();
//     type Error = S::Error;

//     fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
//     where
//         T: Serialize,
//     {
//         key.serialize(self)?;
//         Ok(())
//     }

//     fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
//     where
//         T: Serialize,
//     {
//         value.serialize(self)?;
//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(())
//     }
// }

impl<'a, S: SerializeMap> SerializeMap for &'a mut InnerMapSerializer<S> {
    type Ok = ();
    type Error = S::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        key.serialize(&mut **self)?;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.expect_key = true;
        Ok(())
    }
}

// impl<S: SerializeMap> SerializeTupleVariant for InnerMapSerializer<S> {
//     type Ok = ();
//     type Error = S::Error;

//     fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
//     where
//         T: Serialize,
//     {
//         value.serialize(self)?;
//         Ok(())
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(())
//     }
// }

impl<'a, S: SerializeMap> SerializeTupleVariant for &'a mut InnerMapSerializer<S> {
    type Ok = ();
    type Error = S::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// impl<S: SerializeMap> SerializeStruct for InnerMapSerializer<S> {
//     type Ok = ();
//     type Error = S::Error;

//     fn serialize_field<T: ?Sized>(
//         &mut self,
//         _key: &'static str,
//         value: &T,
//     ) -> Result<(), Self::Error>
//     where
//         T: Serialize,
//     {
//         value.serialize(self)
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(())
//     }
// }

impl<'a, S: SerializeMap> SerializeStruct for &'a mut InnerMapSerializer<S> {
    type Ok = ();
    type Error = S::Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// impl<S: SerializeMap> SerializeStructVariant for InnerMapSerializer<S> {
//     type Ok = ();
//     type Error = S::Error;

//     fn serialize_field<T: ?Sized>(
//         &mut self,
//         _key: &'static str,
//         value: &T,
//     ) -> Result<(), Self::Error>
//     where
//         T: Serialize,
//     {
//         value.serialize(self)
//     }

//     fn end(self) -> Result<Self::Ok, Self::Error> {
//         Ok(())
//     }
// }

impl<'a, S: SerializeMap> SerializeStructVariant for &'a mut InnerMapSerializer<S> {
    type Ok = ();
    type Error = S::Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

pub(crate) struct ForwardingSerializer<S> {
    inner: S,
}

impl<S: SerializeMap> SerializeMap for ForwardingSerializer<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.inner.serialize_key(key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.inner.serialize_value(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<S: SerializeStruct> SerializeStruct for ForwardingSerializer<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.inner.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

impl<S: SerializeStructVariant> SerializeStructVariant for ForwardingSerializer<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        self.inner.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.inner.end()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bytes::Bytes;

    use crate::packstream::bolt;

    use super::{
        super::{super::from_bytes, Error as OurError, Serializer as OurSerializer},
        *,
    };

    fn serialize<T: ?Sized + Serialize>(value: &T) -> Result<Bytes, OurError> {
        let mut inner = OurSerializer::empty();
        as_map(&mut inner, value)?;
        let content = inner.end();
        Ok(content)
    }

    fn assert<T: ?Sized + Serialize>(
        value: &T,
        expected: impl IntoIterator<Item = (&'static str, i32)>,
    ) {
        let expected = expected
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect::<BTreeMap<_, _>>();

        let actual = serialize(value).unwrap();
        let actual: BTreeMap<String, i32> = from_bytes(actual).unwrap();

        assert_eq!(expected, actual);
    }

    fn assert_empty<T: ?Sized + Serialize>(value: &T) {
        assert(value, []);
    }

    #[test]
    fn serialize_empty_array() {
        assert_empty::<[i32; 0]>(&[]);
    }

    #[test]
    fn serialize_none() {
        assert_empty(&None::<i32>);
    }

    #[test]
    fn serialize_unit() {
        assert_empty(&());
    }

    fn assert_single_entry<T: ?Sized + Serialize>(value: &T) {
        assert(value, [("key", 42)]);
    }

    #[test]
    fn serialize_single_pair() {
        assert_single_entry(&("key", 42));
    }

    #[test]
    fn serialize_pair_with_unit() {
        assert_single_entry(&("key", 42, ()));
    }

    #[test]
    fn serialize_nested_pair_with_unit() {
        assert_single_entry(&(("key", 42), ()));
    }

    #[test]
    fn serialize_array_of_pair() {
        assert_single_entry(&[("key", 42)]);
    }

    fn assert_two_entries<T: ?Sized + Serialize>(value: &T) {
        assert(value, [("key1", 42), ("key2", 1337)]);
    }

    #[test]
    fn serialize_two_pairs() {
        assert_two_entries(&("key1", 42, ("key2", 1337)));
    }

    #[test]
    fn serialize_two_pairs_with_unit() {
        assert_two_entries(&("key1", 42, ("key2", 1337, ())));
    }

    #[test]
    fn serialize_two_nested_pairs_with_unit() {
        assert_two_entries(&("key1", 42, (("key2", 1337), ())));
    }

    #[test]
    fn serialize_two_doubly_nested_pairs_with_unit() {
        assert_two_entries(&(("key1", 42), (("key2", 1337), ())));
    }

    #[test]
    fn serialize_two_tuples() {
        assert_two_entries(&(("key1", 42), ("key2", 1337)));
    }

    #[test]
    fn serialize_two_tuples_with_unit() {
        assert_two_entries(&(("key1", 42), ("key2", 1337, ())));
    }

    #[test]
    fn serialize_array_of_two_tuples() {
        assert_two_entries(&[("key1", 42), ("key2", 1337)]);
    }

    #[test]
    fn serialize_map_as_is() {
        assert_two_entries(
            &[("key1", 42), ("key2", 1337)]
                .into_iter()
                .collect::<BTreeMap<_, _>>(),
        );
    }

    #[test]
    fn serialize_nested_map() {
        let value = (
            "key1",
            42,
            (
                "key2",
                [("key3", 1337), ("key4", 1338), ("key5", 1339)]
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
            ),
        );
        let actual = serialize(&value).unwrap();

        let expected = bolt()
            .tiny_map(2)
            .tiny_string("key1")
            .tiny_int(42)
            .tiny_string("key2")
            .tiny_map(3)
            .tiny_string("key3")
            .int16(1337)
            .tiny_string("key4")
            .int16(1338)
            .tiny_string("key5")
            .int16(1339)
            .build();

        assert_eq!(expected, actual);
    }
}
