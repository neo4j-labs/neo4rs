use serde::de::{Deserialize, Deserializer, Error};

use super::de::{impl_visitor, impl_visitor_ref};

use super::{
    urel::UnboundRelationship, urel::UnboundRelationshipRef, Node, NodeRef, Relationship,
    RelationshipRef,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathRef<'de> {
    nodes: Vec<NodeRef<'de>>,
    rels: Vec<UnboundRelationshipRef<'de>>,
    indices: Vec<isize>,
}

/// A node within the graph.
impl<'de> PathRef<'de> {
    /// Returns the start [`Node`] of this path.
    pub fn start(&self) -> &NodeRef<'de> {
        &self.nodes[0]
    }

    /// Returns the end [`Node`] of this path.
    pub fn end(&self) -> &NodeRef<'de> {
        self.nodes().last().unwrap()
    }

    /// Returns the number of segments in this path, which will be the same as the number of relationships.
    pub fn len(&self) -> usize {
        self.indices.len() / 2
    }

    /// Returns true if this path has no segments.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Returns a reference to the [`Node`] with the given id if it is contained in this path.
    pub fn get_node_by_id(&self, id: u64) -> Option<&NodeRef<'de>> {
        self.nodes.iter().find(|o| o.id() == id)
    }

    /// Returns a [`Relationship`] with the given id if it is contained in this path.
    pub fn get_relationship_by_id(&self, id: u64) -> Option<RelationshipRef<'de>> {
        self.relationships().find(|o| o.id() == id)
    }

    #[cfg(test)]
    fn get_unbounded_relationship_by_id(&self, id: u64) -> Option<&UnboundRelationshipRef<'de>> {
        self.rels.iter().find(|o| o.id() == id)
    }

    /// Returns an [`Iterator`] over the nodes in this path.
    /// The nodes will appear in the same order as they appear in the path
    pub fn nodes<'a>(
        &'a self,
    ) -> impl ExactSizeIterator<Item = &'a NodeRef<'de>> + DoubleEndedIterator + 'a {
        NodesIter::new(&self.nodes, &self.indices)
    }

    /// Returns an [`Iterator`] over the relationships in this path.
    /// The relationships will appear in the same order as they appear in the path.
    /// Note that this iterator does not return references but owned types.
    /// To iterate over the individual segments of this path and delay creating new relationships,
    /// use [`Path::segments`].
    pub fn relationships<'a>(&'a self) -> impl ExactSizeIterator<Item = RelationshipRef<'de>> + 'a {
        SegmentsIter::new(&self.nodes, &self.rels, &self.indices).map(|o| o.relationship)
    }

    pub fn segments<'a>(
        &'a self,
    ) -> impl ExactSizeIterator<Item = Segment<'a, NodeRef<'de>, RelationshipRef<'de>>> + 'a {
        SegmentsIter::new(&self.nodes, &self.rels, &self.indices)
    }

    pub fn to_owned(&self) -> Path {
        Path {
            nodes: self.nodes.iter().map(NodeRef::to_owned).collect(),
            rels: self
                .rels
                .iter()
                .map(UnboundRelationshipRef::to_owned)
                .collect(),
            indices: self.indices.clone(),
        }
    }

    pub fn into_owned(self) -> Path {
        Path {
            nodes: self.nodes.into_iter().map(NodeRef::into_owned).collect(),
            rels: self
                .rels
                .into_iter()
                .map(UnboundRelationshipRef::into_owned)
                .collect(),
            indices: self.indices,
        }
    }
}

/// A segment combines a relationship in a path with a start and end node that describe the traversal direction for that relationship.
/// For example, the path `(n1)-[r1]->(n2)<-[r2]-(n3)` contains two segments:
/// Segment 1: `(n1)-[r1]->(n2)`, Segment 2: `(n2)<-[r2]-(n3)`
#[derive(Clone, Debug, PartialEq)]
pub struct Segment<'path, N: 'path, R: 'path> {
    /// The [`Node`] at the start of the segment.
    pub start: &'path N,
    /// The [`Relationship`] connecting the two nodes.
    /// The [`Relationship::start_node_id()`] might be different from the [`Segment::start`] field
    /// of this segment if the relationship was traversed in inverse order.
    pub relationship: R,
    /// The [`Node`] at the end of the segment.
    pub end: &'path N,
}

pub(super) trait FromUndirected<N> {
    type Undirected;

    fn from_undirected(start: &N, end: &N, rel: &Self::Undirected) -> Self;
}

pub(super) struct SegmentsIter<'a, N: 'a, R: FromUndirected<N> + 'a> {
    nodes: &'a [N],
    rels: &'a [R::Undirected],
    indices: std::slice::ChunksExact<'a, isize>,
    last_node: usize,
}

impl<'a, N: 'a, R: FromUndirected<N> + 'a> SegmentsIter<'a, N, R> {
    fn new(nodes: &'a [N], rels: &'a [R::Undirected], indices: &'a [isize]) -> Self {
        Self {
            nodes,
            rels,
            indices: indices.chunks_exact(2),
            last_node: 0,
        }
    }

    fn extract_segment(&mut self, rel_and_node: &[isize]) -> Segment<'a, N, R> {
        let next_node_index = NodesIter::<N>::extract_node_index(rel_and_node);

        let rel_index = rel_and_node[0];
        assert_ne!(rel_index, 0, "Relationship index cannot be zero");

        let rel = {
            let (rel_index, start_node_index, end_node_index) = if rel_index > 0 {
                (rel_index as usize, self.last_node, next_node_index)
            } else {
                (rel_index.unsigned_abs(), next_node_index, self.last_node)
            };

            let rel_index = rel_index - 1;

            let start_node = &self.nodes[start_node_index];
            let end_node = &self.nodes[end_node_index];
            let rel = &self.rels[rel_index];

            R::from_undirected(start_node, end_node, rel)
        };

        let segment = {
            let start_node = &self.nodes[self.last_node];
            let end_node = &self.nodes[next_node_index];

            Segment {
                start: start_node,
                relationship: rel,
                end: end_node,
            }
        };

        self.last_node = next_node_index;

        segment
    }
}

impl<'a, N: 'a, R: FromUndirected<N> + 'a> Iterator for SegmentsIter<'a, N, R> {
    type Item = Segment<'a, N, R>;

    fn next(&mut self) -> Option<Self::Item> {
        let rel_and_node = self.indices.next()?;
        Some(self.extract_segment(rel_and_node))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.indices.count()
    }
}

impl<'a, N: 'a, R: FromUndirected<N> + 'a> ExactSizeIterator for SegmentsIter<'a, N, R> {
    fn len(&self) -> usize {
        self.indices.len()
    }
}

struct NodesIter<'a, T: 'a> {
    nodes: &'a [T],
    indices: std::slice::ChunksExact<'a, isize>,
    emit_start: bool,
}

impl<'a, T: 'a> NodesIter<'a, T> {
    fn new(nodes: &'a [T], indices: &'a [isize]) -> Self {
        Self {
            nodes,
            indices: indices.chunks_exact(2),
            emit_start: true,
        }
    }

    fn extract_node_index(rel_and_node: &[isize]) -> usize {
        let node_index = rel_and_node[1];
        usize::try_from(node_index).expect("Node index values must be >= 0")
    }

    fn extract_node(&self, rel_and_node: &[isize]) -> &'a T {
        let index = Self::extract_node_index(rel_and_node);
        &self.nodes[index]
    }
}

impl<'a, T: 'a> Iterator for NodesIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node_index = if self.emit_start {
            self.emit_start = false;
            0
        } else {
            let rel_and_node = self.indices.next()?;
            let node_index = rel_and_node[1];
            usize::try_from(node_index).expect("Node index values must be >= 0")
        };

        Some(&self.nodes[node_index])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let inner = self.indices.size_hint();
        if self.emit_start {
            (inner.0 + 1, inner.1.and_then(|o| o.checked_add(1)))
        } else {
            inner
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.indices.count() + usize::from(self.emit_start)
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let index = match self.indices.last() {
            Some(rel_and_node) => Self::extract_node_index(rel_and_node),
            None if self.emit_start => 0,
            None => return None,
        };
        Some(&self.nodes[index])
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n > 0 {
            let rel_and_node = self
                .indices
                .nth(n - usize::from(std::mem::take(&mut self.emit_start)))?;
            Some(self.extract_node(rel_and_node))
        } else {
            self.next()
        }
    }
}

impl<'a, T: 'a> ExactSizeIterator for NodesIter<'a, T> {
    fn len(&self) -> usize {
        self.indices.len() + usize::from(self.emit_start)
    }
}

impl<'a, T: 'a> DoubleEndedIterator for NodesIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = match self.indices.next_back() {
            Some(rel_and_node) => Self::extract_node_index(rel_and_node),
            None => {
                if self.emit_start {
                    self.emit_start = false;
                    0
                } else {
                    return None;
                }
            }
        };
        Some(&self.nodes[index])
    }
}

impl_visitor_ref!(PathRef<'de>(nodes, rels, indices) == 0x50);

impl<'de> Deserialize<'de> for PathRef<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_struct("Path", &[], Self::visitor())
            .and_then(|p| {
                if p.nodes.is_empty() {
                    return Err(Error::custom("must have at least one node"));
                }
                if p.indices.len() % 2 != 0 {
                    return Err(Error::custom("indices must be even"));
                }
                Ok(p)
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path {
    pub(crate) nodes: Vec<Node>,
    pub(super) rels: Vec<UnboundRelationship>,
    pub(crate) indices: Vec<isize>,
}

/// A node within the graph.
impl Path {
    /// Returns the start [`Node`] of this path.
    pub fn start(&self) -> &Node {
        &self.nodes[0]
    }

    /// Returns the end [`Node`] of this path.
    pub fn end(&self) -> &Node {
        self.nodes().last().unwrap()
    }

    /// Returns the number of segments in this path, which will be the same as the number of relationships.
    pub fn len(&self) -> usize {
        self.indices.len() / 2
    }

    /// Returns true if this path has no segments.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Returns a reference to the [`Node`] with the given id if it is contained in this path.
    pub fn get_node_by_id(&self, id: u64) -> Option<&Node> {
        self.nodes.iter().find(|o| o.id() == id)
    }

    /// Returns a [`Relationship`] with the given id if it is contained in this path.
    pub fn get_relationship_by_id(&self, id: u64) -> Option<Relationship> {
        self.relationships().find(|o| o.id() == id)
    }

    /// Returns an [`Iterator`] over the nodes in this path.
    /// The nodes will appear in the same order as they appear in the path
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = &'_ Node> + DoubleEndedIterator + '_ {
        NodesIter::new(&self.nodes, &self.indices)
    }

    /// Returns an [`Iterator`] over the relationships in this path.
    /// The relationships will appear in the same order as they appear in the path.
    /// Note that this iterator does not return references but owned types.
    /// To iterate over the individual segments of this path and delay creating new relationships,
    /// use [`Path::segments`].
    pub fn relationships(&self) -> impl ExactSizeIterator<Item = Relationship> + '_ {
        SegmentsIter::new(&self.nodes, &self.rels, &self.indices).map(|o| o.relationship)
    }

    pub fn segments(&self) -> impl ExactSizeIterator<Item = Segment<'_, Node, Relationship>> + '_ {
        SegmentsIter::new(&self.nodes, &self.rels, &self.indices)
    }
}

impl_visitor!(Path(nodes, rels, indices) == 0x50);

impl<'de> Deserialize<'de> for Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_struct("Path", &[], Self::visitor())
            .and_then(|p| {
                if p.nodes.is_empty() {
                    return Err(Error::custom("must have at least one node"));
                }
                if p.indices.len() % 2 != 0 {
                    return Err(Error::custom("indices must be even"));
                }
                Ok(p)
            })
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::packstream::{bolt, from_bytes_ref, Data};

    use super::*;

    #[test]
    fn deserialize() {
        let data = bolt_path();
        let mut data = Data::new(data);
        let path: PathRef = from_bytes_ref(&mut data).unwrap();

        assert_eq!(path.start().id(), 42);
        assert_eq!(path.end().id(), 1);

        assert_eq!(path.len(), 3);

        let nodes: Vec<_> = path.nodes().map(|n| n.id()).collect();
        assert_eq!(nodes, vec![42, 69, 42, 1]);

        let rels: Vec<_> = path
            .relationships()
            .map(|r| (r.start_node_id(), r.id(), r.end_node_id()))
            .collect();
        assert_eq!(rels, vec![(42, 1000, 69), (42, 1000, 69), (42, 1001, 1)]);

        let segments: Vec<_> = path.segments().collect();

        assert_eq!(
            segments,
            vec![
                Segment {
                    start: path.get_node_by_id(42).unwrap(),
                    relationship: RelationshipRef::from_undirected(
                        path.get_node_by_id(42).unwrap(),
                        path.get_node_by_id(69).unwrap(),
                        path.get_unbounded_relationship_by_id(1000).unwrap()
                    ),
                    end: path.get_node_by_id(69).unwrap(),
                },
                Segment {
                    start: path.get_node_by_id(69).unwrap(),
                    relationship: RelationshipRef::from_undirected(
                        path.get_node_by_id(42).unwrap(),
                        path.get_node_by_id(69).unwrap(),
                        path.get_unbounded_relationship_by_id(1000).unwrap()
                    ),
                    end: path.get_node_by_id(42).unwrap(),
                },
                Segment {
                    start: path.get_node_by_id(42).unwrap(),
                    relationship: RelationshipRef::from_undirected(
                        path.get_node_by_id(42).unwrap(),
                        path.get_node_by_id(1).unwrap(),
                        path.get_unbounded_relationship_by_id(1001).unwrap()
                    ),
                    end: path.get_node_by_id(1).unwrap(),
                },
            ]
        )
    }

    /// (42)-[1000]->(69)<-[1000]-(42)-[1001]->(1)
    fn bolt_path() -> Bytes {
        fn node(id: i8) -> Bytes {
            bolt()
                .structure(3, 0x4E)
                .tiny_int(id)
                .tiny_list(0)
                .tiny_map(0)
                .build()
        }

        fn rel(id: i16) -> Bytes {
            bolt()
                .structure(3, 0x72)
                .int16(id)
                .tiny_string("REL")
                .tiny_map(0)
                .build()
        }

        bolt()
            .structure(3, 0x50)
            .tiny_list(3)
            .extend(node(42))
            .extend(node(1))
            .extend(node(69))
            .tiny_list(2)
            .extend(rel(1000))
            .extend(rel(1001))
            .tiny_list(6)
            .tiny_int(1)
            .tiny_int(2)
            .tiny_int(-1)
            .tiny_int(0)
            .tiny_int(2)
            .tiny_int(1)
            .build()
    }
}
