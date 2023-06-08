# osm-io

This crate provides tools used to manipulate the [Open Street Map](https://wiki.openstreetmap.org/wiki/Main_Page)
data.

The basic OSM data model is broadly defined by [OSM XML](https://wiki.openstreetmap.org/wiki/OSM_XML)
with Node and Way objects defining geometry and Relation and Tag objects providing unstructured
extension mechanisms. Due to different uses of the OSM data, variety of
[data formats](https://wiki.openstreetmap.org/wiki/Databases_and_data_access_APIs) emerged to
store and transmit it.
Our main focus here are the following formats:
* [apidb](https://wiki.openstreetmap.org/wiki/Openstreetmap-website/Database_schema) - database
schema backing [OSM](https://www.openstreetmap.org/) website
* [*.osm.pbf](https://wiki.openstreetmap.org/wiki/PBF_Format) - a very efficient data format
used to transmit OSM data that can be downloaded from http://download.geofabrik.de/ or from
https://planet.openstreetmap.org/pbf/.

The goal at this stage is to be able to load large *.osm.pbf files, such as planet.osm.pbf into a
Postgresql OSM database (apidb schema) and to dump an entire Postgres OSM database into a
planet.osm.pbf file. See [osm-admin](https://github.com/navigatorsguild/osm-admin)

Because the data sets are very large, a special attention is given to maintaining control over
memory size and utilizing multiple CPU cores whenever is possible.

## Roadmap
* implement *.osm.pbf reader and writer - Done
* implement *.osm.pbf parallel reader and parallel writer - Done
* implement apidb reader and writer - Done
* provide basic filtering (see example below) - Done
* convert between *.osm.pbf and apidb and vice versa - Done see examples.
* [S2](http://s2geometry.io/) indexing - index the entire OSM dataset by S2 cells for farther
processing
* context indexing - index the entire OSM dataset by relations between its objects. So, for
example, it would be possible to efficiently discard all Nodes that belong to a deleted Way.

## Issues
Issues are welcome and appreciated. Please submit to https://github.com/navigatorsguild/osm-io/issues

## Examples
Example for filtering out nodes from *.osm.pbf extract
```rust
use std::path::PathBuf;

use anyhow;
use benchmark_rs::stopwatch::StopWatch;
use simple_logger::SimpleLogger;

use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;

pub fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new().init()?;
    log::info!("Started pbf io pipeline");
    let mut stopwatch = StopWatch::new();
    stopwatch.start();
    let input_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/niue-230109.osm.pbf");
    let reader = pbf::reader::Reader::new(&input_path)?;
    let mut file_info = FileInfo::default();
    file_info.with_writingprogram_str("pbf-io-example");
    let mut writer = pbf::writer::Writer::from_file_info(
        output_path,
        file_info,
        CompressionType::Zlib,
    )?;

    writer.write_header()?;

    for element in reader.elements()? {
        let mut filter_out = false;
        match &element {
            Element::Node { node } => {
                for tag in node.tags() {
                    if tag.k() == "natural" && tag.v() == "tree" {
                        filter_out = true;
                        break;
                    }
                }
            }
            Element::Way { way: _ } => {}
            Element::Relation { relation: _ } => {}
            Element::Sentinel => {
                filter_out = true;
            }
        }
        if !filter_out {
            writer.write_element(element)?;
        }
    }

    writer.close()?;

    log::info!("Finished pbf io pipeline, time: {}", stopwatch);
    Ok(())
}
```

## Similar Software
* [libosmium](https://osmcode.org/libosmium/) - very fast and very mature with a Python wrapper.
* [osmosis](https://wiki.openstreetmap.org/wiki/Osmosis) - reference implementation for most if
not all features.
* [osmpbf](https://crates.io/crates/osmpbf) - very efficient *.osm.pbf reader written in Rust


License: MIT OR Apache-2.0
