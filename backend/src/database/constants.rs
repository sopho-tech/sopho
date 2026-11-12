#[derive(Debug)]
pub struct QueryResult {
    pub columns: Vec<serde_json::Value>,
    pub data: Vec<serde_json::Value>,
}

pub const POSTGRES_POOL_MAX_CONNECTIONS: u32 = 10;
pub const SQLITE_POOL_MAX_CONNECTIONS: u32 = 4;
pub const CATALOG_SAMPLE_VALUES_PER_COLUMN: usize = 5;
pub const CATALOG_SAMPLE_COLUMNS_PER_QUERY: usize = 50;
pub const CATALOG_TABLE_FETCH_CONCURRENCY: usize = 8;
pub const CARRIER_GRADE_NAT_FIRST_OCTET: u8 = 100;
pub const CARRIER_GRADE_NAT_SECOND_OCTET_MASK: u8 = 0b1100_0000;
pub const CARRIER_GRADE_NAT_SECOND_OCTET_PREFIX: u8 = 0b0100_0000;
pub const THIS_NETWORK_FIRST_OCTET: u8 = 0;
pub const IPV6_UNIQUE_LOCAL_MASK: u16 = 0xfe00;
pub const IPV6_UNIQUE_LOCAL_PREFIX: u16 = 0xfc00;
pub const IPV6_LINK_LOCAL_MASK: u16 = 0xffc0;
pub const IPV6_LINK_LOCAL_PREFIX: u16 = 0xfe80;
pub const IPV6_SITE_LOCAL_PREFIX: u16 = 0xfec0;
pub const BENCHMARKING_FIRST_OCTET: u8 = 198;
pub const BENCHMARKING_SECOND_OCTET_MASK: u8 = 0b1111_1110;
pub const BENCHMARKING_SECOND_OCTET_PREFIX: u8 = 18;
pub const PROTOCOL_ASSIGNMENTS_OCTETS: [u8; 3] = [192, 0, 0];
pub const RESERVED_FIRST_OCTET_MINIMUM: u8 = 240;
pub const UNIX_SOCKET_HOST_PREFIX: char = '/';
pub const IPV6_LITERAL_OPENING_BRACKET: char = '[';
pub const IPV6_LITERAL_CLOSING_BRACKET: char = ']';
pub const IPV6_NAT64_PREFIX: [u16; 6] = [0x0064, 0xff9b, 0, 0, 0, 0];
