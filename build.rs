use prost_build;
use prost_build::Config;

fn main() -> std::io::Result<()> {
    let is_primary_env = env!("CARGO_PRIMARY_PACKAGE", "0");
    if is_primary_env == "1" {
        let protos = [
            "./src/osm/pbf/format/fileformat.proto",
            "./src/osm/pbf/format/osmformat.proto"
        ];

        let includes = [
            "src/"
        ];
        let mut config = Config::new();
        config.out_dir("./src/osm/pbf/osmpbf");
        config.compile_protos(&protos, &includes)?;
        std::fs::rename("./src/osm/pbf/osmpbf/osmpbf.rs", "./src/osm/pbf/osmpbf/mod.rs")
    } else {
        Ok(())
    }
}