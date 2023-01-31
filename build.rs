use prost_build;

fn main() {
    let protos = [
        "./src/osm/pbf/format/fileformat.proto",
        "./src/osm/pbf/format/osmformat.proto"
    ];

    let includes = [
        "src/"
    ];
    prost_build::compile_protos(&protos, &includes).unwrap()
}