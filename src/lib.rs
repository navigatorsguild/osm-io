extern crate core;

pub(crate) mod osmpbf {
    include!(concat!(env!("OUT_DIR"), "/osmpbf.rs"));
}

pub mod osm;
