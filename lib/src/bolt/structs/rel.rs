use serde::de::{Deserialize, DeserializeOwned, Deserializer};

use crate::{
    bolt::{Node, NodeRef},
    packstream::{de, from_bytes, from_bytes_ref, from_bytes_seed, Data},
};

use super::de::{impl_visitor, impl_visitor_ref, Keys, Single};

/// A relationship within the graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationshipRef<'de> {
    id: u64,
    start_node_id: u64,
    end_node_id: u64,
    r#type: &'de str,
    properties: Data,
    element_id: Option<&'de str>,
    start_node_element_id: Option<&'de str>,
    end_node_element_id: Option<&'de str>,
}

impl<'de> RelationshipRef<'de> {
    /// An id for this relationship.
    ///
    /// Ids are guaranteed to remain stable for the duration of the session
    /// they were found in, but may be re-used for other entities after that.
    /// As such, if you want a public identity to use for your entities,
    /// attaching an explicit 'id' property or similar persistent
    /// and unique identifier is a better choice.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// A unique id for this relationship.
    ///
    /// It is recommended to attach an explicit 'id' property or similar
    /// persistent and unique identifier if you want a public identity
    /// to use for your entities.
    pub fn element_id(&self) -> Option<&'de str> {
        self.element_id
    }

    /// The id of the node where this relationship starts.
    pub fn start_node_id(&self) -> u64 {
        self.start_node_id
    }

    /// A unique id for the node where this relationship starts.
    pub fn start_node_element_id(&self) -> Option<&'de str> {
        self.start_node_element_id
    }

    /// The id of the node where this relationship ends.
    pub fn end_node_id(&self) -> u64 {
        self.end_node_id
    }

    /// A unique id for the node where this relationship ends.
    pub fn end_node_element_id(&self) -> Option<&'de str> {
        self.end_node_element_id
    }

    /// The type of this relationship.
    pub fn typ(&self) -> &'de str {
        self.r#type
    }

    /// Get the names of the properties of this node
    pub fn keys(&mut self) -> Vec<&str> {
        self.to::<Keys>().expect("properties should be a map").0
    }

    /// Get an attribute of this node and deserialize it into custom type that implements [`serde::Deserialize`]
    pub fn get<'this, T: Deserialize<'this> + 'this>(
        &'this mut self,
        key: &str,
    ) -> Result<Option<T>, de::Error> {
        self.properties.reset();
        from_bytes_seed(&mut self.properties, Single::new(key))
    }

    /// Deserialize the node to a type that implements [`serde::Deserialize`].
    /// The target type may borrow data from the node's properties.
    pub fn to<'this, T: Deserialize<'this> + 'this>(&'this mut self) -> Result<T, de::Error>
    where
        'de: 'this,
    {
        self.properties.reset();
        from_bytes_ref(&mut self.properties)
    }

    /// Convert the node into a type that implements [`serde::Deserialize`].
    /// The target type must not borrow data from the node's properties.
    pub fn into<T: DeserializeOwned>(self) -> Result<T, de::Error> {
        from_bytes(self.properties.into_inner())
    }

    pub fn to_owned(&self) -> Relationship {
        Relationship {
            id: self.id,
            start_node_id: self.start_node_id,
            end_node_id: self.end_node_id,
            r#type: self.r#type.to_owned(),
            properties: self.properties.clone(),
            element_id: self.element_id.map(ToOwned::to_owned),
            start_node_element_id: self.start_node_element_id.map(ToOwned::to_owned),
            end_node_element_id: self.end_node_element_id.map(ToOwned::to_owned),
        }
    }

    pub fn into_owned(self) -> Relationship {
        Relationship {
            id: self.id,
            start_node_id: self.start_node_id,
            end_node_id: self.end_node_id,
            r#type: self.r#type.to_owned(),
            properties: self.properties,
            element_id: self.element_id.map(ToOwned::to_owned),
            start_node_element_id: self.start_node_element_id.map(ToOwned::to_owned),
            end_node_element_id: self.end_node_element_id.map(ToOwned::to_owned),
        }
    }
}

impl<'de> super::path::FromUndirected<NodeRef<'de>> for RelationshipRef<'de> {
    type Undirected = super::urel::UnboundRelationshipRef<'de>;

    fn from_undirected(start: &NodeRef<'de>, end: &NodeRef<'de>, rel: &Self::Undirected) -> Self {
        Self {
            id: rel.id(),
            start_node_id: start.id(),
            end_node_id: end.id(),
            r#type: rel.typ(),
            properties: rel.properties.clone(),
            element_id: rel.element_id(),
            start_node_element_id: start.element_id(),
            end_node_element_id: end.element_id(),
        }
    }
}

impl_visitor_ref!(RelationshipRef<'de>(id, start_node_id, end_node_id, r#type, properties, [element_id, start_node_element_id, end_node_element_id]) == 0x52);

impl<'de> Deserialize<'de> for RelationshipRef<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Relationship", &[], Self::visitor())
    }
}

/// A relationship within the graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relationship {
    id: u64,
    start_node_id: u64,
    end_node_id: u64,
    r#type: String,
    properties: Data,
    element_id: Option<String>,
    start_node_element_id: Option<String>,
    end_node_element_id: Option<String>,
}

impl Relationship {
    /// An id for this relationship.
    ///
    /// Ids are guaranteed to remain stable for the duration of the session
    /// they were found in, but may be re-used for other entities after that.
    /// As such, if you want a public identity to use for your entities,
    /// attaching an explicit 'id' property or similar persistent
    /// and unique identifier is a better choice.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// A unique id for this relationship.
    ///
    /// It is recommended to attach an explicit 'id' property or similar
    /// persistent and unique identifier if you want a public identity
    /// to use for your entities.
    pub fn element_id(&self) -> Option<&str> {
        self.element_id.as_deref()
    }

    /// The id of the node where this relationship starts.
    pub fn start_node_id(&self) -> u64 {
        self.start_node_id
    }

    /// A unique id for the node where this relationship starts.
    pub fn start_node_element_id(&self) -> Option<&str> {
        self.start_node_element_id.as_deref()
    }

    /// The id of the node where this relationship ends.
    pub fn end_node_id(&self) -> u64 {
        self.end_node_id
    }

    /// A unique id for the node where this relationship ends.
    pub fn end_node_element_id(&self) -> Option<&str> {
        self.end_node_element_id.as_deref()
    }

    /// The type of this relationship.
    pub fn typ(&self) -> &str {
        &self.r#type
    }

    /// Get the names of the properties of this node
    pub fn keys(&mut self) -> Vec<&str> {
        self.to::<Keys>().expect("properties should be a map").0
    }

    /// Get an attribute of this node and deserialize it into custom type that implements [`serde::Deserialize`]
    pub fn get<'this, T: Deserialize<'this> + 'this>(
        &'this mut self,
        key: &str,
    ) -> Result<Option<T>, de::Error> {
        self.properties.reset();
        from_bytes_seed(&mut self.properties, Single::new(key))
    }

    /// Deserialize the node to a type that implements [`serde::Deserialize`].
    /// The target type may borrow data from the node's properties.
    pub fn to<'this, T: Deserialize<'this> + 'this>(&'this mut self) -> Result<T, de::Error> {
        self.properties.reset();
        from_bytes_ref(&mut self.properties)
    }

    /// Convert the node into a type that implements [`serde::Deserialize`].
    /// The target type must not borrow data from the node's properties.
    pub fn into<T: DeserializeOwned>(self) -> Result<T, de::Error> {
        from_bytes(self.properties.into_inner())
    }
}

impl super::path::FromUndirected<Node> for Relationship {
    type Undirected = super::urel::UnboundRelationship;

    fn from_undirected(start: &Node, end: &Node, rel: &Self::Undirected) -> Self {
        Self {
            id: rel.id(),
            start_node_id: start.id(),
            end_node_id: end.id(),
            r#type: rel.typ().to_owned(),
            properties: rel.properties.clone(),
            element_id: rel.element_id().map(ToOwned::to_owned),
            start_node_element_id: start.element_id().map(ToOwned::to_owned),
            end_node_element_id: end.element_id().map(ToOwned::to_owned),
        }
    }
}

impl_visitor!(
    Relationship(
        id,
        start_node_id,
        end_node_id,
        r#type,
        properties,
        [element_id, start_node_element_id, end_node_element_id]
    ) == 0x52
);

impl<'de> Deserialize<'de> for Relationship {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Relationship", &[], Self::visitor())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use bytes::Bytes;
    use serde::Deserialize;
    use serde_test::{assert_de_tokens, Token};
    use test_case::{test_case, test_matrix};

    use crate::packstream::{bolt, from_bytes_ref, BoltBytesBuilder, Data};

    use super::*;

    #[derive(Deserialize, Debug, PartialEq)]
    struct Owned {
        name: String,
        age: u32,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Borrowed<'a> {
        name: Cow<'a, str>,
        age: u32,
    }

    #[test_case(tokens_v4())]
    #[test_case(tagged_tokens_v4())]
    #[test_case(tokens_v5())]
    #[test_case(tagged_tokens_v5())]
    fn tokens((tokens, element_ids): (Vec<Token>, Option<ElementIds<'static>>)) {
        let rel = RelationshipRef {
            id: 42,
            start_node_id: 84,
            end_node_id: 1337,
            r#type: "REL",
            properties: Data::new(properties_data()),
            element_id: element_ids.as_ref().map(|o| o.id),
            start_node_element_id: element_ids.as_ref().map(|o| o.start_node),
            end_node_element_id: element_ids.as_ref().map(|o| o.end_node),
        };

        assert_de_tokens(&rel, &tokens);
    }

    fn tokens_v4() -> (Vec<Token>, Option<ElementIds<'static>>) {
        let properties = properties_data();
        let properties = properties.to_vec();
        let properties = Vec::leak(properties);

        (
            vec![
                Token::Seq { len: Some(5) },
                Token::U8(42),
                Token::U8(84),
                Token::U16(1337),
                Token::BorrowedStr("REL"),
                Token::BorrowedBytes(properties),
                Token::SeqEnd,
            ],
            None,
        )
    }

    fn tagged_tokens_v4() -> (Vec<Token>, Option<ElementIds<'static>>) {
        let (mut tokens, element_ids) = tokens_v4();
        tokens.splice(0..0, [Token::Enum { name: "Bolt" }, Token::U32(0x52)]);
        (tokens, element_ids)
    }

    fn tokens_v5() -> (Vec<Token>, Option<ElementIds<'static>>) {
        let (mut tokens, _) = tokens_v4();
        tokens[0] = Token::Seq { len: Some(8) };
        let seq_end = tokens.len() - 1;
        tokens
            .splice(
                seq_end..seq_end,
                [
                    Token::BorrowedStr("id"),
                    Token::BorrowedStr("start"),
                    Token::BorrowedStr("end"),
                ],
            )
            .for_each(drop);

        (
            tokens,
            Some(ElementIds {
                id: "id",
                start_node: "start",
                end_node: "end",
            }),
        )
    }

    fn tagged_tokens_v5() -> (Vec<Token>, Option<ElementIds<'static>>) {
        let (mut tokens, element_ids) = tokens_v5();
        tokens.splice(0..0, [Token::Enum { name: "Bolt" }, Token::U32(0x52)]);
        (tokens, element_ids)
    }

    #[test_matrix(
        [ bolt_v4(),  bolt_v5() ],
        [   owned(), borrowed() ]
    )]
    fn deserialize<T>((data, element_ids): (Bytes, Option<ElementIds>), expected: T)
    where
        T: std::fmt::Debug + PartialEq + for<'a> Deserialize<'a>,
    {
        let mut data = Data::new(data);
        let mut rel: RelationshipRef = from_bytes_ref(&mut data).unwrap();

        assert_eq!(rel.id(), 42);
        assert_eq!(rel.start_node_id(), 84);
        assert_eq!(rel.end_node_id(), 1337);

        assert_eq!(rel.element_id(), element_ids.as_ref().map(|o| o.id));
        assert_eq!(
            rel.start_node_element_id(),
            element_ids.as_ref().map(|o| o.start_node)
        );
        assert_eq!(
            rel.end_node_element_id(),
            element_ids.as_ref().map(|o| o.end_node)
        );

        assert_eq!(rel.typ(), "REL");

        let properties = properties_data();
        assert_eq!(rel.properties.bytes(), &properties);
        assert_eq!(rel.keys(), &["age", "name"]);

        assert_eq!(rel.get("age").unwrap(), Some(1337));
        assert_eq!(rel.get("name").unwrap(), Some("Alice"));

        assert_eq!(rel.get("missing").unwrap(), None::<String>);

        let actual: T = rel.to().unwrap();
        assert_eq!(actual, expected);

        let actual: T = rel.into().unwrap();
        assert_eq!(actual, expected);
    }

    fn bolt_v4() -> (Bytes, Option<ElementIds<'static>>) {
        (rel_test_data(5).build(), None)
    }

    fn bolt_v5() -> (Bytes, Option<ElementIds<'static>>) {
        (
            rel_test_data(8)
                .tiny_string("id")
                .tiny_string("start")
                .tiny_string("end")
                .build(),
            Some(ElementIds {
                id: "id",
                start_node: "start",
                end_node: "end",
            }),
        )
    }

    fn owned() -> Owned {
        Owned {
            name: "Alice".to_owned(),
            age: 1337,
        }
    }

    fn borrowed() -> Borrowed<'static> {
        Borrowed {
            name: "Alice".into(),
            age: 1337,
        }
    }

    fn rel_test_data(fields: u8) -> BoltBytesBuilder {
        bolt()
            .structure(fields, 0x52)
            .tiny_int(42)
            .tiny_int(84)
            .int16(1337)
            .tiny_string("REL")
            .extend(properties_data())
    }

    fn properties_data() -> Bytes {
        bolt()
            .tiny_map(2)
            .tiny_string("age")
            .int16(1337)
            .tiny_string("name")
            .tiny_string("Alice")
            .build()
    }

    struct ElementIds<'a> {
        id: &'a str,
        start_node: &'a str,
        end_node: &'a str,
    }
}
