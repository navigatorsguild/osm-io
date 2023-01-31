use std::io::Cursor;

use prost::Message;

pub(crate) mod osmpbf {
    include!(concat!(env!("OUT_DIR"), "/osmpbf.rs"));
}

pub mod osm;
mod error;
