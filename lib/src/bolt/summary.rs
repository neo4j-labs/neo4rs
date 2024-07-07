use std::{fmt, marker::PhantomData};

use serde::{
    de::{self, VariantAccess as _, Visitor},
    Deserialize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Summary<R> {
    Success(Success<R>),
    Ignored,
    Failure(Failure),
}

impl<R: std::fmt::Debug> Summary<R> {
    pub fn into_error(self, msg: &'static str) -> crate::errors::Error {
        match self {
            Summary::Failure(f) => f.into_error(),
            otherwise => crate::Error::UnexpectedMessage(format!(
                "unexpected response for {}: {:?}",
                msg, otherwise
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct Success<R> {
    pub(crate) metadata: R,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Failure {
    pub(crate) code: String,
    pub(crate) message: String,
}

impl Failure {
    pub fn into_error(self) -> crate::errors::Error {
        let Self { code, message } = self;
        crate::errors::Error::Neo4j(crate::errors::Neo4jError::new(code, message))
    }
}

impl<'de, R: Deserialize<'de>> Deserialize<'de> for Summary<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor<R>(PhantomData<R>);

        impl<'de, R: Deserialize<'de>> de::Visitor<'de> for Visitor<R> {
            type Value = Summary<R>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A Bolt summary struct")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: de::EnumAccess<'de>,
            {
                let (tag, data) = data.variant::<u8>()?;
                match tag {
                    0x70 => Ok(Summary::Success(data.newtype_variant::<Success<R>>()?)),
                    0x7E => Ok(Summary::Ignored),
                    0x7F => Ok(Summary::Failure(data.newtype_variant::<Failure>()?)),
                    _ => Err(de::Error::invalid_type(
                        // TODO: proper error
                        de::Unexpected::Other(&format!("struct with tag {tag:02X}")),
                        &self,
                    )),
                }
            }
        }

        deserializer.deserialize_enum(
            "Summary",
            &["Success", "Ignore", "Failure"],
            Visitor(PhantomData),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packstream::bolt;
    use crate::{bolt::MessageResponse as _, BoltMap, BoltString, BoltType};

    #[test]
    fn parse_hello_success() {
        let data = bolt().structure(1, 0x70).null().build();

        let success = Summary::<()>::parse(data).unwrap();

        match success {
            Summary::Success(Success { metadata: () }) => {}
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn parse_ignore() {
        let data = bolt().structure(1, 0x7E).build();

        let success = Summary::<()>::parse(data).unwrap();

        assert_eq!(success, Summary::Ignored);
    }

    #[test]
    fn parse_failure() {
        let data = bolt()
            .structure(1, 0x7F)
            .tiny_map(2)
            .tiny_string("code")
            .string8("Neo.ClientError.Security.Unauthorized")
            .tiny_string("message")
            .string8("The client is unauthorized due to authentication failure.")
            .build();

        let failure = Summary::<()>::parse(data).unwrap();

        let failure = match failure {
            Summary::Failure(failure) => failure,
            _ => panic!("Expected failure"),
        };

        assert_eq!(failure.code, "Neo.ClientError.Security.Unauthorized");
        assert_eq!(
            failure.message,
            "The client is unauthorized due to authentication failure."
        );
    }
}
