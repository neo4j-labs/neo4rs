use crate::routing::{RoutingTable, Server};
use crate::BoltMap;

/// Convert a BoltMap into a RoutingTable
impl From<BoltMap> for RoutingTable {
    fn from(rt: BoltMap) -> Self {
        let ttl = rt.get::<u64>("ttl").unwrap_or(0);
        let db = rt.get::<String>("db").ok().map(|db| db.into());
        let rt_servers = rt.get::<Vec<BoltMap>>("servers").unwrap_or_default();
        let server = rt_servers
            .iter()
            .map(|server| {
                let role = server.get::<String>("role").unwrap_or_default();
                let addresses = server.get::<Vec<String>>("addresses").unwrap_or_default();
                Server { addresses, role }
            })
            .collect::<Vec<Server>>();
        RoutingTable {
            ttl,
            db,
            servers: server,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::BoltRequest;
    use crate::routing::{Route, RouteBuilder, Routing};
    use crate::types::{list, map, string, BoltWireFormat};
    use crate::version::Version;
    use bytes::*;

    #[test]
    fn should_serialize_route() {
        let route = RouteBuilder::new(Routing::Yes(vec![("address".into(), "localhost".into())]), vec![])
            .with_db("neo4j".into())
            .build(Version::V4_3);
        let r = match route {
            BoltRequest::Route(r) => r,
            _ => panic!("Expected Route"),
        };
        let bytes: Bytes = Route::from(r).into_bytes(Version::V4_1).unwrap();

        assert_eq!(
            bytes,
            Bytes::from_static(&[
                0xB3,
                0x66,
                map::TINY | 1,
                string::TINY | 7,
                b'a',
                b'd',
                b'd',
                b'r',
                b'e',
                b's',
                b's',
                string::TINY | 9,
                b'l',
                b'o',
                b'c',
                b'a',
                b'l',
                b'h',
                b'o',
                b's',
                b't',
                list::TINY | 0,
                string::TINY | 5,
                b'n',
                b'e',
                b'o',
                b'4',
                b'j',
            ])
        );
    }
}
