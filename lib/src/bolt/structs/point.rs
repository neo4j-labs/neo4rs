use std::fmt;

use serde::de::{Deserialize, Deserializer, VariantAccess};
use thiserror::Error;

use super::de::impl_visitor;

/// A representation of a single location in 2-dimensional space.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point2D {
    srid: Crs,
    x: f64,
    y: f64,
}

impl Point2D {
    /// The Coordinate Reference System of this point.
    pub fn srid(&self) -> Crs {
        self.srid
    }

    /// The x coordinate of this point.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// The y coordinate of this point.
    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn to_point(self) -> Point {
        match self.srid {
            Crs::Wgs842D => Point::Wgs842d(Wgs842d {
                longitude: self.x,
                latitude: self.y,
            }),
            Crs::Wgs843D => Point::Wgs843d(Wgs843d {
                longitude: self.x,
                latitude: self.y,
                height: 0.0,
            }),
            Crs::Cartesian2D => Point::Cartesian2d(Cartesian2d {
                x: self.x,
                y: self.y,
            }),
            Crs::Cartesian3D => Point::Cartesian3d(Cartesian3d {
                x: self.x,
                y: self.y,
                z: 0.0,
            }),
        }
    }

    pub fn as_nav(&self) -> Result<nav_types::WGS84<f64>, ConversionError> {
        self.to_point().as_nav()
    }
}

/// A representation of a single location in 3-dimensional space.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point3D {
    srid: Crs,
    x: f64,
    y: f64,
    z: f64,
}

impl Point3D {
    /// The Coordinate Reference System of this point.
    pub fn srid(&self) -> Crs {
        self.srid
    }

    /// The x coordinate of this point.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// The y coordinate of this point.
    pub fn y(&self) -> f64 {
        self.y
    }

    /// The z coordinate of this point.
    pub fn z(&self) -> f64 {
        self.z
    }

    pub fn to_point(self) -> Point {
        match self.srid {
            Crs::Wgs842D => Point::Wgs842d(Wgs842d {
                longitude: self.x,
                latitude: self.y,
            }),
            Crs::Wgs843D => Point::Wgs843d(Wgs843d {
                longitude: self.x,
                latitude: self.y,
                height: self.z,
            }),
            Crs::Cartesian2D => Point::Cartesian2d(Cartesian2d {
                x: self.x,
                y: self.y,
            }),
            Crs::Cartesian3D => Point::Cartesian3d(Cartesian3d {
                x: self.x,
                y: self.y,
                z: self.z,
            }),
        }
    }

    pub fn as_nav(&self) -> Result<nav_types::WGS84<f64>, ConversionError> {
        self.to_point().as_nav()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum ConversionError {
    #[error("The coordinate system is not in {0}")]
    WrongSystem(Crs),
    #[error("The point is not defined on the {0} ellipsoid.")]
    UndefinedPosition(Crs),
}

/// A Coordinate Reference System.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Crs {
    Wgs842D,
    Wgs843D,
    Cartesian2D,
    Cartesian3D,
}

impl Crs {
    pub fn from_srid(srid: u16) -> Option<Self> {
        match srid {
            4326 => Some(Crs::Wgs842D),
            4979 => Some(Crs::Wgs843D),
            7203 => Some(Crs::Cartesian2D),
            9157 => Some(Crs::Cartesian3D),
            _ => None,
        }
    }

    pub fn to_srid(self) -> u16 {
        match self {
            Crs::Wgs842D => 4326,
            Crs::Wgs843D => 4979,
            Crs::Cartesian2D => 7203,
            Crs::Cartesian3D => 9157,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "WGS-84" => Some(Crs::Wgs842D),
            "WGS-84-3D" => Some(Crs::Wgs843D),
            "cartesian" => Some(Crs::Cartesian2D),
            "cartesian-3D" => Some(Crs::Cartesian3D),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Crs::Wgs842D => "WGS-84",
            Crs::Wgs843D => "WGS-84-3D",
            Crs::Cartesian2D => "cartesian",
            Crs::Cartesian3D => "cartesian-3D",
        }
    }
}

impl fmt::Display for Crs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Wgs842d {
    pub longitude: f64,
    pub latitude: f64,
}

impl Wgs842d {
    pub fn to_nav(self) -> Option<nav_types::WGS84<f64>> {
        nav_types::WGS84::try_from_degrees_and_meters(self.latitude, self.longitude, 0.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Wgs843d {
    pub longitude: f64,
    pub latitude: f64,
    pub height: f64,
}

impl Wgs843d {
    pub fn to_nav(self) -> Option<nav_types::WGS84<f64>> {
        nav_types::WGS84::try_from_degrees_and_meters(self.latitude, self.longitude, self.height)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Cartesian2d {
    pub x: f64,
    pub y: f64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Cartesian3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Point {
    Wgs842d(Wgs842d),
    Wgs843d(Wgs843d),
    Cartesian2d(Cartesian2d),
    Cartesian3d(Cartesian3d),
}

impl Point {
    pub fn as_nav(&self) -> Result<nav_types::WGS84<f64>, ConversionError> {
        match self {
            Point::Wgs842d(p) => p
                .to_nav()
                .ok_or(ConversionError::UndefinedPosition(Crs::Wgs842D)),
            Point::Wgs843d(p) => p
                .to_nav()
                .ok_or(ConversionError::UndefinedPosition(Crs::Wgs843D)),
            Point::Cartesian2d(_) => Err(ConversionError::WrongSystem(Crs::Wgs842D)),
            Point::Cartesian3d(_) => Err(ConversionError::WrongSystem(Crs::Wgs843D)),
        }
    }

    pub fn x(&self) -> f64 {
        match self {
            Point::Wgs842d(p) => p.longitude,
            Point::Wgs843d(p) => p.longitude,
            Point::Cartesian2d(p) => p.x,
            Point::Cartesian3d(p) => p.x,
        }
    }

    pub fn y(&self) -> f64 {
        match self {
            Point::Wgs842d(p) => p.latitude,
            Point::Wgs843d(p) => p.latitude,
            Point::Cartesian2d(p) => p.y,
            Point::Cartesian3d(p) => p.y,
        }
    }

    pub fn z(&self) -> f64 {
        match self {
            Point::Wgs843d(p) => p.height,
            Point::Cartesian3d(p) => p.z,
            _ => 0.0,
        }
    }
}

impl_visitor!(Point2D(srid, x, y) == 0x58);
impl_visitor!(Point3D(srid, x, y, z) == 0x59);

impl<'de> Deserialize<'de> for Crs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Vis;

        impl<'de> serde::de::Visitor<'de> for Vis {
            type Value = Crs;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid Coordinate Reference System identifier")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                u16::try_from(v)
                    .ok()
                    .and_then(Crs::from_srid)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                u16::try_from(v)
                    .ok()
                    .and_then(Crs::from_srid)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Crs::from_name(v).ok_or_else(|| {
                    serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self)
                })
            }
        }

        deserializer.deserialize_u16(Vis)
    }
}

impl<'de> Deserialize<'de> for Point2D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Point2D", &[], Self::visitor())
    }
}

impl<'de> Deserialize<'de> for Point3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Point3D", &[], Self::visitor())
    }
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Vis;

        impl<'de> serde::de::Visitor<'de> for Vis {
            type Value = Point;

            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                formatter.write_str(concat!("a valid Point2D or Point3D struct"))
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: ::serde::de::EnumAccess<'de>,
            {
                let (tag, data) = ::serde::de::EnumAccess::variant::<u8>(data)?;
                if tag == 0x58 {
                    data.struct_variant(&[], Point2D::visitor())
                        .map(|p| p.to_point())
                } else if tag == 0x59 {
                    data.struct_variant(&[], Point3D::visitor())
                        .map(|p| p.to_point())
                } else {
                    Err(serde::de::Error::invalid_type(
                        serde::de::Unexpected::Other(&format!("struct with tag {:02X}", tag)),
                        &self,
                    ))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: ::serde::de::SeqAccess<'de>,
            {
                let len = seq.size_hint().unwrap_or_default();

                if len == 3 {
                    let srid = seq
                        .next_element()?
                        .ok_or_else(|| ::serde::de::Error::missing_field("srid"))?;
                    let x = seq
                        .next_element()?
                        .ok_or_else(|| ::serde::de::Error::missing_field("x"))?;
                    let y = seq
                        .next_element()?
                        .ok_or_else(|| ::serde::de::Error::missing_field("y"))?;
                    Ok(Point2D { srid, x, y }.to_point())
                } else if len == 4 {
                    let srid = seq
                        .next_element()?
                        .ok_or_else(|| ::serde::de::Error::missing_field("srid"))?;
                    let x = seq
                        .next_element()?
                        .ok_or_else(|| ::serde::de::Error::missing_field("x"))?;
                    let y = seq
                        .next_element()?
                        .ok_or_else(|| ::serde::de::Error::missing_field("y"))?;
                    let z = seq
                        .next_element()?
                        .ok_or_else(|| ::serde::de::Error::missing_field("z"))?;
                    Ok(Point3D { srid, x, y, z }.to_point())
                } else {
                    Err(::serde::de::Error::invalid_length(
                        len,
                        &"a sequence of length 3 or 4",
                    ))
                }
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: ::serde::de::MapAccess<'de>,
            {
                let tag = map.next_key::<u8>()?;
                match tag {
                    Some(0x58) => map.next_value::<Point2D>().map(|p| p.to_point()),
                    Some(0x59) => map.next_value::<Point3D>().map(|p| p.to_point()),
                    Some(tag) => {
                        return Err(serde::de::Error::invalid_type(
                            serde::de::Unexpected::Other(&format!("struct with tag {:02X}", tag)),
                            &"a Bolt struct (tag {:02X})",
                        ))
                    }
                    None => Err(serde::de::Error::missing_field("tag")),
                }
            }
        }

        deserializer.deserialize_struct("Point", &[], Vis)
    }
}

#[cfg(test)]
mod tests {
    use crate::packstream::{bolt, from_bytes_ref, Data};

    use super::*;

    macro_rules! assert_approx_eq {
        ($left:expr, $right:expr) => {
            assert!(($left - $right).abs() <= (f64::EPSILON * 10.0));
        };
    }

    #[test]
    fn deserialize() {
        let data = bolt()
            .structure(3, 0x58)
            .int16(4326)
            .float(12.994341)
            .float(55.611784)
            .build();
        let mut data = Data::new(data);
        let point: Point2D = from_bytes_ref(&mut data).unwrap();

        assert_eq!(point.srid(), Crs::Wgs842D);
        assert_approx_eq!(point.x(), 12.994341);
        assert_approx_eq!(point.y(), 55.611784);

        let Point::Wgs842d(point) = point.to_point() else {
            panic!("point is not a wgs84 2d point: {:?}", point.to_point())
        };

        assert_approx_eq!(point.longitude, 12.994341);
        assert_approx_eq!(point.latitude, 55.611784);

        let malmoe = point.to_nav().unwrap();

        assert_approx_eq!(malmoe.longitude_degrees(), 12.994341);
        assert_approx_eq!(malmoe.latitude_degrees(), 55.611784);

        let data = bolt()
            .structure(3, 0x58)
            .int16(4326)
            .float(12.564590)
            .float(55.672874)
            .build();
        let mut data = Data::new(data);
        let Point::Wgs842d(point) = from_bytes_ref(&mut data).unwrap() else {
            panic!()
        };

        let copenhagen = point.to_nav().unwrap();

        assert_approx_eq!(copenhagen.longitude_degrees(), 12.564590);
        assert_approx_eq!(copenhagen.latitude_degrees(), 55.672874);

        assert_approx_eq!(copenhagen.distance(&malmoe).round(), 27842.0);
    }
}
