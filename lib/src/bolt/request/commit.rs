use serde::{Deserialize, Serialize};

use crate::bolt::{ExpectedResponse, Summary};
use crate::bookmarks::Bookmark;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Commit;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct CommitResponse {
    pub bookmark: Option<String>,
}

impl Bookmark for CommitResponse {
    fn get_bookmark(&self) -> Option<&str> {
        self.bookmark.as_deref()
    }
}

impl Serialize for Commit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit_variant("Request", 0x12, "COMMIT")
    }
}

impl ExpectedResponse for Commit {
    type Response = Summary<CommitResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::MessageResponse;
    use crate::{bolt::Message as _, packstream::bolt};

    #[test]
    fn serialize() {
        let commit = Commit;
        let bytes = commit.to_bytes().unwrap();

        let expected = bolt().structure(0, 0x12).build();

        assert_eq!(bytes, expected);
    }

    #[test]
    fn deserialize() {
        let data = bolt()
            .tiny_map(1)
            .string8("bookmark")
            .string8("example-bookmark:1")
            .build();
        let response = CommitResponse::parse(data).unwrap();

        assert!(response.bookmark.is_some());
        assert_eq!(response.bookmark.unwrap(), "example-bookmark:1");
    }
}
