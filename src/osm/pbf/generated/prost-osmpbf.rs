#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Blob {
    /// When compressed, the uncompressed size
    #[prost(int32, optional, tag = "2")]
    pub raw_size: ::core::option::Option<i32>,
    #[prost(oneof = "blob::Data", tags = "1, 3, 4, 5, 6, 7")]
    pub data: ::core::option::Option<blob::Data>,
}
/// Nested message and enum types in `Blob`.
pub mod blob {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Data {
        /// No compression
        #[prost(bytes, tag = "1")]
        Raw(::prost::alloc::vec::Vec<u8>),
        /// Possible compressed versions of the data.
        #[prost(bytes, tag = "3")]
        ZlibData(::prost::alloc::vec::Vec<u8>),
        /// For LZMA compressed data (optional)
        #[prost(bytes, tag = "4")]
        LzmaData(::prost::alloc::vec::Vec<u8>),
        /// Formerly used for bzip2 compressed data. Deprecated in 2010.
        ///
        /// Don't reuse this tag number.
        #[prost(bytes, tag = "5")]
        ObsoleteBzip2Data(::prost::alloc::vec::Vec<u8>),
        /// For LZ4 compressed data (optional)
        #[prost(bytes, tag = "6")]
        Lz4Data(::prost::alloc::vec::Vec<u8>),
        /// For ZSTD compressed data (optional)
        #[prost(bytes, tag = "7")]
        ZstdData(::prost::alloc::vec::Vec<u8>),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlobHeader {
    #[prost(string, required, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(bytes = "vec", optional, tag = "2")]
    pub indexdata: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(int32, required, tag = "3")]
    pub datasize: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeaderBlock {
    #[prost(message, optional, tag = "1")]
    pub bbox: ::core::option::Option<HeaderBBox>,
    /// Additional tags to aid in parsing this dataset
    #[prost(string, repeated, tag = "4")]
    pub required_features: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "5")]
    pub optional_features: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "16")]
    pub writingprogram: ::core::option::Option<::prost::alloc::string::String>,
    /// From the bbox field.
    #[prost(string, optional, tag = "17")]
    pub source: ::core::option::Option<::prost::alloc::string::String>,
    /// Replication timestamp, expressed in seconds since the epoch,
    /// otherwise the same value as in the "timestamp=..." field
    /// in the state.txt file used by Osmosis.
    #[prost(int64, optional, tag = "32")]
    pub osmosis_replication_timestamp: ::core::option::Option<i64>,
    /// Replication sequence number (sequenceNumber in state.txt).
    #[prost(int64, optional, tag = "33")]
    pub osmosis_replication_sequence_number: ::core::option::Option<i64>,
    /// Replication base URL (from Osmosis' configuration.txt file).
    #[prost(string, optional, tag = "34")]
    pub osmosis_replication_base_url: ::core::option::Option<
        ::prost::alloc::string::String,
    >,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeaderBBox {
    #[prost(sint64, required, tag = "1")]
    pub left: i64,
    #[prost(sint64, required, tag = "2")]
    pub right: i64,
    #[prost(sint64, required, tag = "3")]
    pub top: i64,
    #[prost(sint64, required, tag = "4")]
    pub bottom: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrimitiveBlock {
    #[prost(message, required, tag = "1")]
    pub stringtable: StringTable,
    #[prost(message, repeated, tag = "2")]
    pub primitivegroup: ::prost::alloc::vec::Vec<PrimitiveGroup>,
    /// Granularity, units of nanodegrees, used to store coordinates in this block.
    #[prost(int32, optional, tag = "17", default = "100")]
    pub granularity: ::core::option::Option<i32>,
    /// Offset value between the output coordinates and the granularity grid in units of nanodegrees.
    #[prost(int64, optional, tag = "19", default = "0")]
    pub lat_offset: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "20", default = "0")]
    pub lon_offset: ::core::option::Option<i64>,
    /// Granularity of dates, normally represented in units of milliseconds since the 1970 epoch.
    #[prost(int32, optional, tag = "18", default = "1000")]
    pub date_granularity: ::core::option::Option<i32>,
}
/// Group of OSMPrimitives. All primitives in a group must be the same type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrimitiveGroup {
    #[prost(message, repeated, tag = "1")]
    pub nodes: ::prost::alloc::vec::Vec<Node>,
    #[prost(message, optional, tag = "2")]
    pub dense: ::core::option::Option<DenseNodes>,
    #[prost(message, repeated, tag = "3")]
    pub ways: ::prost::alloc::vec::Vec<Way>,
    #[prost(message, repeated, tag = "4")]
    pub relations: ::prost::alloc::vec::Vec<Relation>,
    #[prost(message, repeated, tag = "5")]
    pub changesets: ::prost::alloc::vec::Vec<ChangeSet>,
}
/// * String table, contains the common strings in each block.
///
/// Note that we reserve index '0' as a delimiter, so the entry at that
/// index in the table is ALWAYS blank and unused.
///
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StringTable {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub s: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// Optional metadata that may be included into each primitive.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Info {
    #[prost(int32, optional, tag = "1", default = "-1")]
    pub version: ::core::option::Option<i32>,
    #[prost(int64, optional, tag = "2")]
    pub timestamp: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "3")]
    pub changeset: ::core::option::Option<i64>,
    #[prost(int32, optional, tag = "4")]
    pub uid: ::core::option::Option<i32>,
    /// String IDs
    #[prost(uint32, optional, tag = "5")]
    pub user_sid: ::core::option::Option<u32>,
    /// The visible flag is used to store history information. It indicates that
    /// the current object version has been created by a delete operation on the
    /// OSM API.
    /// When a writer sets this flag, it MUST add a required_features tag with
    /// value "HistoricalInformation" to the HeaderBlock.
    /// If this flag is not available for some object it MUST be assumed to be
    /// true if the file has the required_features tag "HistoricalInformation"
    /// set.
    #[prost(bool, optional, tag = "6")]
    pub visible: ::core::option::Option<bool>,
}
/// * Optional metadata that may be included into each primitive. Special dense format used in DenseNodes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenseInfo {
    #[prost(int32, repeated, tag = "1")]
    pub version: ::prost::alloc::vec::Vec<i32>,
    /// DELTA coded
    #[prost(sint64, repeated, tag = "2")]
    pub timestamp: ::prost::alloc::vec::Vec<i64>,
    /// DELTA coded
    #[prost(sint64, repeated, tag = "3")]
    pub changeset: ::prost::alloc::vec::Vec<i64>,
    /// DELTA coded
    #[prost(sint32, repeated, tag = "4")]
    pub uid: ::prost::alloc::vec::Vec<i32>,
    /// String IDs for usernames. DELTA coded
    #[prost(sint32, repeated, tag = "5")]
    pub user_sid: ::prost::alloc::vec::Vec<i32>,
    /// The visible flag is used to store history information. It indicates that
    /// the current object version has been created by a delete operation on the
    /// OSM API.
    /// When a writer sets this flag, it MUST add a required_features tag with
    /// value "HistoricalInformation" to the HeaderBlock.
    /// If this flag is not available for some object it MUST be assumed to be
    /// true if the file has the required_features tag "HistoricalInformation"
    /// set.
    #[prost(bool, repeated, tag = "6")]
    pub visible: ::prost::alloc::vec::Vec<bool>,
}
/// This is kept for backwards compatibility but not used anywhere.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChangeSet {
    #[prost(int64, required, tag = "1")]
    pub id: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Node {
    #[prost(sint64, required, tag = "1")]
    pub id: i64,
    /// Parallel arrays.
    ///
    /// String IDs.
    #[prost(uint32, repeated, tag = "2")]
    pub keys: ::prost::alloc::vec::Vec<u32>,
    /// String IDs.
    #[prost(uint32, repeated, tag = "3")]
    pub vals: ::prost::alloc::vec::Vec<u32>,
    /// May be omitted in omitmeta
    #[prost(message, optional, tag = "4")]
    pub info: ::core::option::Option<Info>,
    #[prost(sint64, required, tag = "8")]
    pub lat: i64,
    #[prost(sint64, required, tag = "9")]
    pub lon: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenseNodes {
    /// DELTA coded
    #[prost(sint64, repeated, tag = "1")]
    pub id: ::prost::alloc::vec::Vec<i64>,
    #[prost(message, optional, tag = "5")]
    pub denseinfo: ::core::option::Option<DenseInfo>,
    /// DELTA coded
    #[prost(sint64, repeated, tag = "8")]
    pub lat: ::prost::alloc::vec::Vec<i64>,
    /// DELTA coded
    #[prost(sint64, repeated, tag = "9")]
    pub lon: ::prost::alloc::vec::Vec<i64>,
    /// Special packing of keys and vals into one array. May be empty if all nodes in this block are tagless.
    #[prost(int32, repeated, tag = "10")]
    pub keys_vals: ::prost::alloc::vec::Vec<i32>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Way {
    #[prost(int64, required, tag = "1")]
    pub id: i64,
    /// Parallel arrays.
    #[prost(uint32, repeated, tag = "2")]
    pub keys: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag = "3")]
    pub vals: ::prost::alloc::vec::Vec<u32>,
    #[prost(message, optional, tag = "4")]
    pub info: ::core::option::Option<Info>,
    /// DELTA coded
    #[prost(sint64, repeated, tag = "8")]
    pub refs: ::prost::alloc::vec::Vec<i64>,
    /// The following two fields are optional. They are only used in a special
    /// format where node locations are also added to the ways. This makes the
    /// files larger, but allows creating way geometries directly.
    ///
    /// If this is used, you MUST set the optional_features tag "LocationsOnWays"
    /// and the number of values in refs, lat, and lon MUST be the same.
    ///
    /// DELTA coded, optional
    #[prost(sint64, repeated, tag = "9")]
    pub lat: ::prost::alloc::vec::Vec<i64>,
    /// DELTA coded, optional
    #[prost(sint64, repeated, tag = "10")]
    pub lon: ::prost::alloc::vec::Vec<i64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Relation {
    #[prost(int64, required, tag = "1")]
    pub id: i64,
    /// Parallel arrays.
    #[prost(uint32, repeated, tag = "2")]
    pub keys: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, repeated, tag = "3")]
    pub vals: ::prost::alloc::vec::Vec<u32>,
    #[prost(message, optional, tag = "4")]
    pub info: ::core::option::Option<Info>,
    /// Parallel arrays
    ///
    /// This should have been defined as uint32 for consistency, but it is now too late to change it
    #[prost(int32, repeated, tag = "8")]
    pub roles_sid: ::prost::alloc::vec::Vec<i32>,
    /// DELTA encoded
    #[prost(sint64, repeated, tag = "9")]
    pub memids: ::prost::alloc::vec::Vec<i64>,
    #[prost(enumeration = "relation::MemberType", repeated, tag = "10")]
    pub types: ::prost::alloc::vec::Vec<i32>,
}
/// Nested message and enum types in `Relation`.
pub mod relation {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum MemberType {
        Node = 0,
        Way = 1,
        Relation = 2,
    }
    impl MemberType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                MemberType::Node => "NODE",
                MemberType::Way => "WAY",
                MemberType::Relation => "RELATION",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "NODE" => Some(Self::Node),
                "WAY" => Some(Self::Way),
                "RELATION" => Some(Self::Relation),
                _ => None,
            }
        }
    }
}
