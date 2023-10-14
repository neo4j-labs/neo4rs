use crate::{
    cenum,
    types::{
        serde::{builder::SetOnce, BoltKind},
        BoltFloat, BoltInteger, BoltPoint2D, BoltPoint3D,
    },
    Point2D, Point3D,
};

use std::{fmt, marker::PhantomData, result::Result};

use serde::{
    de::{
        DeserializeSeed, Deserializer, EnumAccess, Error, IntoDeserializer, MapAccess, SeqAccess,
        VariantAccess, Visitor,
    },
    forward_to_deserialize_any, Deserialize,
};

impl<'de> Deserialize<'de> for Point2D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BoltPoint2D::deserialize(deserializer).map(Point2D::new)
    }
}

impl<'de> Deserialize<'de> for Point3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BoltPoint3D::deserialize(deserializer).map(Point3D::new)
    }
}

cenum!(Field { SrId, X, Y, Z });

#[derive(Clone, Debug, Default)]
struct BoltPointBuilder {
    sr_id: SetOnce<BoltInteger>,
    x: SetOnce<BoltFloat>,
    y: SetOnce<BoltFloat>,
    z: SetOnce<BoltFloat>,
}

impl BoltPointBuilder {
    fn sr_id<E: Error>(&mut self, sr_id: impl FnOnce() -> Result<BoltInteger, E>) -> Result<(), E> {
        match self.sr_id.try_insert_with(sr_id)? {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field(Field::SrId.name())),
        }
    }

    fn x<E: Error>(&mut self, x: impl FnOnce() -> Result<BoltFloat, E>) -> Result<(), E> {
        match self.x.try_insert_with(x)? {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field(Field::X.name())),
        }
    }

    fn y<E: Error>(&mut self, y: impl FnOnce() -> Result<BoltFloat, E>) -> Result<(), E> {
        match self.y.try_insert_with(y)? {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field(Field::Y.name())),
        }
    }

    fn z<E: Error>(&mut self, z: impl FnOnce() -> Result<BoltFloat, E>) -> Result<(), E> {
        match self.z.try_insert_with(z)? {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::duplicate_field(Field::Z.name())),
        }
    }

    fn build<P: FromBuilder, E: Error>(self) -> Result<P, E> {
        P::build(self)
    }
}

trait FromBuilder: Sized {
    fn build<E: Error>(builder: BoltPointBuilder) -> Result<Self, E>;
}

impl FromBuilder for BoltPoint2D {
    fn build<E: Error>(builder: BoltPointBuilder) -> Result<Self, E> {
        if builder.z.is_set() {
            return Err(Error::unknown_field("z", &Field::NAMES[..3]));
        }
        let sr_id = builder
            .sr_id
            .ok_or_else(|| Error::missing_field(Field::SrId.name()))?;
        let x = builder
            .x
            .ok_or_else(|| Error::missing_field(Field::X.name()))?;
        let y = builder
            .y
            .ok_or_else(|| Error::missing_field(Field::Y.name()))?;

        Ok(BoltPoint2D { sr_id, x, y })
    }
}

impl FromBuilder for BoltPoint3D {
    fn build<E: Error>(builder: BoltPointBuilder) -> Result<Self, E> {
        let sr_id = builder
            .sr_id
            .ok_or_else(|| Error::missing_field(Field::SrId.name()))?;
        let x = builder
            .x
            .ok_or_else(|| Error::missing_field(Field::X.name()))?;
        let y = builder
            .y
            .ok_or_else(|| Error::missing_field(Field::Y.name()))?;
        let z = builder
            .z
            .ok_or_else(|| Error::missing_field(Field::Z.name()))?;

        Ok(BoltPoint3D { sr_id, x, y, z })
    }
}

pub struct BoltPointVisitor<P, E>(PhantomData<(P, E)>);

impl BoltPointVisitor<(), ()> {
    pub fn _2d<E: Error>() -> BoltPointVisitor<BoltPoint2D, E> {
        BoltPointVisitor(PhantomData)
    }

    pub fn _3d<E: Error>() -> BoltPointVisitor<BoltPoint3D, E> {
        BoltPointVisitor(PhantomData)
    }
}

impl<'de, P: FromBuilder, E: Error> Visitor<'de> for BoltPointVisitor<P, E> {
    type Value = P;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "struct {}", std::any::type_name::<P>())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: ::serde::de::MapAccess<'de>,
    {
        let mut point = BoltPointBuilder::default();

        while let Some(key) = map.next_key::<Field>()? {
            match key {
                Field::SrId => point.sr_id(|| map.next_value())?,
                Field::X => point.x(|| map.next_value())?,
                Field::Y => point.y(|| map.next_value())?,
                Field::Z => point.z(|| map.next_value())?,
            }
        }

        let point = point.build()?;
        Ok(point)
    }
}

impl<'de> Deserialize<'de> for BoltPoint2D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "BoltPoint2D",
            &Field::NAMES[..3],
            BoltPointVisitor::_2d::<D::Error>(),
        )
    }
}

impl<'de> Deserialize<'de> for BoltPoint3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "BoltPoint3D",
            Field::NAMES,
            BoltPointVisitor::_3d::<D::Error>(),
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct BoltPoint<'de> {
    sr_id: &'de BoltInteger,
    x: &'de BoltFloat,
    y: &'de BoltFloat,
    z: Option<&'de BoltFloat>,
}

impl<'de> From<&'de BoltPoint2D> for BoltPoint<'de> {
    fn from(point: &'de BoltPoint2D) -> Self {
        Self {
            sr_id: &point.sr_id,
            x: &point.x,
            y: &point.y,
            z: None,
        }
    }
}

impl<'de> From<&'de BoltPoint3D> for BoltPoint<'de> {
    fn from(point: &'de BoltPoint3D) -> Self {
        Self {
            sr_id: &point.sr_id,
            x: &point.x,
            y: &point.y,
            z: Some(&point.z),
        }
    }
}

struct BoltPointData<'de, I, E> {
    point: BoltPoint<'de>,
    fields: I,
    next_field: Option<Field>,
    _error: PhantomData<E>,
}

impl<'de, I, E> BoltPointData<'de, I, E> {
    fn new(point: BoltPoint<'de>, fields: I) -> Self {
        Self {
            point,
            fields,
            next_field: None,
            _error: PhantomData,
        }
    }
}

impl<'de, E: Error, I: Iterator<Item = Result<Field, &'static str>>> MapAccess<'de>
    for BoltPointData<'de, I, E>
{
    type Error = E;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.fields.next() {
            Some(Ok(field)) => {
                self.next_field = Some(field);
                seed.deserialize(field.into_deserializer()).map(Some)
            }
            Some(Err(field)) => seed.deserialize(field.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.next_field.take() {
            Some(field) => match field {
                Field::SrId => seed.deserialize(self.point.sr_id.value.into_deserializer()),
                Field::X => seed.deserialize(self.point.x.value.into_deserializer()),
                Field::Y => seed.deserialize(self.point.y.value.into_deserializer()),
                Field::Z => match self.point.z {
                    Some(z) => seed.deserialize(z.value.into_deserializer()),
                    None => Err(Error::unknown_field("z", &Field::NAMES[..3])),
                },
            },
            None => seed.deserialize(().into_deserializer()),
        }
    }
}

impl<'de, E: Error, I: Iterator<Item = Result<Field, &'static str>>> SeqAccess<'de>
    for BoltPointData<'de, I, E>
{
    type Error = E;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.next_key::<Field>()?;
        self.next_value_seed(seed).map(Some)
    }
}

impl<'de, E: Error> IntoDeserializer<'de, E> for &'de BoltPoint2D {
    type Deserializer = BoltPointDeserializer<'de, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltPointDeserializer::new(self)
    }
}

impl<'de, E: Error> IntoDeserializer<'de, E> for &'de BoltPoint3D {
    type Deserializer = BoltPointDeserializer<'de, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        BoltPointDeserializer::new(self)
    }
}

pub struct BoltPointDeserializer<'de, E>(BoltPoint<'de>, PhantomData<E>);

impl<'de, E: Error> BoltPointDeserializer<'de, E> {
    pub(crate) fn new(point: impl Into<BoltPoint<'de>>) -> Self {
        Self(point.into(), PhantomData)
    }
}

impl<'de, E: Error> Deserializer<'de> for BoltPointDeserializer<'de, E> {
    type Error = E;

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let fields = match self.0.z {
            None => &Field::VARIANTS[..3],
            Some(_) => Field::VARIANTS,
        };
        visitor.visit_map(BoltPointData::new(self.0, fields.iter().copied().map(Ok)))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let fields = match len {
            2 => [Field::X, Field::Y].as_slice(),
            3 => match self.0.z {
                None => &[Field::SrId, Field::X, Field::Y],
                Some(_) => &[Field::X, Field::Y, Field::Z],
            },
            4 => Field::VARIANTS,
            _ => return Err(Error::invalid_length(len, &"2, 3 or 4")),
        };

        visitor.visit_seq(BoltPointData::new(self.0, fields.iter().copied().map(Ok)))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let fields = fields.iter().map(|o| match Field::from_str(o) {
            Some(field) => Ok(field),
            None => Err(*o),
        });

        visitor.visit_map(BoltPointData::new(self.0, fields))
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
        visitor.visit_enum(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq identifier newtype_struct
    }
}

impl<'de, E: Error> EnumAccess<'de> for BoltPointDeserializer<'de, E> {
    type Error = E;

    type Variant = BoltPointDeserializer<'de, E>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let kind = match self.0.z {
            None => BoltKind::Point2D,
            Some(_) => BoltKind::Point3D,
        };
        let val = seed.deserialize(kind.into_deserializer())?;
        Ok((val, self))
    }
}

impl<'de, E: Error> VariantAccess<'de> for BoltPointDeserializer<'de, E> {
    type Error = E;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let name = match self.0.z {
            None => "Point2D",
            Some(_) => "Point3D",
        };
        self.deserialize_struct(name, fields, visitor)
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, marker::PhantomData};

    use crate::{types::BoltType, DeError};

    use super::*;

    impl BoltPoint2D {
        pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
        where
            T: Deserialize<'this>,
        {
            T::deserialize(self.into_deserializer())
        }
    }

    impl BoltPoint3D {
        pub(crate) fn to<'this, T>(&'this self) -> Result<T, DeError>
        where
            T: Deserialize<'this>,
        {
            T::deserialize(self.into_deserializer())
        }
    }

    fn test_point2d() -> BoltPoint2D {
        BoltPoint2D {
            sr_id: 420.into(),
            x: BoltFloat::new(42.0),
            y: BoltFloat::new(13.37),
        }
    }

    #[test]
    fn point2d_full_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct P {
            sr_id: u64,
            x: f64,
            y: f64,
        }

        test_extract_point2d(P {
            sr_id: 420,
            x: 42.0,
            y: 13.37,
        });
    }

    #[test]
    fn point2d_xy_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct P {
            x: f64,
            y: f64,
        }

        test_extract_point2d(P { x: 42.0, y: 13.37 });
    }

    #[test]
    fn point2d_with_unit_types() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct P<T> {
            _t: PhantomData<T>,
            _u: (),
        }

        test_extract_point2d(P {
            _t: PhantomData::<i32>,
            _u: (),
        });
    }

    #[test]
    fn point2d_tuple_struct_full() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct P(u64, f64, f64);

        test_extract_point2d(P(420, 42.0, 13.37));
    }

    #[test]
    fn point2d_tuple_struct_xy() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct P(f64, f64);

        test_extract_point2d(P(42.0, 13.37));
    }

    #[test]
    fn point2d_tuple_full() {
        test_extract_point2d((420, 42.0, 13.37));
    }

    #[test]
    fn point2d_tuple_xy() {
        test_extract_point2d((42.0, 13.37));
    }

    fn test_extract_point2d<P: Debug + PartialEq + for<'a> Deserialize<'a>>(expected: P) {
        let point = test_point2d();
        let actual = point.to::<P>().unwrap();
        assert_eq!(actual, expected);
    }

    fn test_point3d() -> BoltPoint3D {
        BoltPoint3D {
            sr_id: 420.into(),
            x: BoltFloat::new(42.0),
            y: BoltFloat::new(13.37),
            z: BoltFloat::new(84.0),
        }
    }

    #[test]
    fn point3d_full_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct P {
            sr_id: u64,
            x: f64,
            y: f64,
            z: f64,
        }

        test_extract_point3d(P {
            sr_id: 420,
            x: 42.0,
            y: 13.37,
            z: 84.0,
        });
    }

    #[test]
    fn point3d_xy_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct P {
            x: f64,
            y: f64,
        }

        test_extract_point3d(P { x: 42.0, y: 13.37 });
    }

    #[test]
    fn point3d_xyz_struct() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct P {
            x: f64,
            y: f64,
            z: f64,
        }

        test_extract_point3d(P {
            x: 42.0,
            y: 13.37,
            z: 84.0,
        });
    }

    #[test]
    fn point3d_with_unit_types() {
        #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
        struct P<T> {
            _t: PhantomData<T>,
            _u: (),
        }

        test_extract_point3d(P {
            _t: PhantomData::<i32>,
            _u: (),
        });
    }

    #[test]
    fn point3d_tuple_struct_full() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct P(u64, f64, f64, f64);

        test_extract_point3d(P(420, 42.0, 13.37, 84.0));
    }

    #[test]
    fn point3d_tuple_struct_xy() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct P(f64, f64);

        test_extract_point3d(P(42.0, 13.37));
    }

    #[test]
    fn point3d_tuple_struct_xyz() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct P(f64, f64, f64);

        test_extract_point3d(P(42.0, 13.37, 84.0));
    }

    #[test]
    fn point3d_tuple_full() {
        test_extract_point3d((420, 42.0, 13.37, 84.0));
    }

    #[test]
    fn point3d_tuple_xy() {
        test_extract_point3d((42.0, 13.37));
    }

    #[test]
    fn point3d_tuple_xyz() {
        test_extract_point3d((42.0, 13.37, 84.0));
    }

    fn test_extract_point3d<P: Debug + PartialEq + for<'a> Deserialize<'a>>(expected: P) {
        let point = test_point3d();
        let actual = point.to::<P>().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn point2d_to_bolt_type() {
        let point = test_point2d();
        let actual = point.to::<BoltType>().unwrap();
        assert_eq!(actual, BoltType::Point2D(point));
    }

    #[test]
    fn point2d_to_bolt_point() {
        let point = test_point2d();
        let actual = point.to::<BoltPoint2D>().unwrap();
        assert_eq!(actual, point);
    }

    #[test]
    fn point2d_to_point() {
        let point = test_point2d();
        let actual = point.to::<Point2D>().unwrap();
        assert_eq!(actual.sr_id(), point.sr_id.value);
        assert_eq!(actual.x(), point.x.value);
        assert_eq!(actual.y(), point.y.value);
    }

    #[test]
    fn point3d_to_bolt_type() {
        let point = test_point3d();
        let actual = point.to::<BoltType>().unwrap();
        assert_eq!(actual, BoltType::Point3D(point));
    }

    #[test]
    fn point3d_to_bolt_point() {
        let point = test_point3d();
        let actual = point.to::<BoltPoint3D>().unwrap();
        assert_eq!(actual, point);
    }

    #[test]
    fn point3d_to_point() {
        let point = test_point3d();
        let actual = point.to::<Point3D>().unwrap();
        assert_eq!(actual.sr_id(), point.sr_id.value);
        assert_eq!(actual.x(), point.x.value);
        assert_eq!(actual.y(), point.y.value);
    }
}
