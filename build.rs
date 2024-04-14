use std::path::PathBuf;

// Generate the osmpbf module when developing the osm-io package
// and copy the generated file when using it from other packages
fn main() -> std::io::Result<()> {
    let is_primary_opt = option_env!("CARGO_PRIMARY_PACKAGE");
    match is_primary_opt {
        None => {
            let out_dir = std::env::var("OUT_DIR").unwrap();
            let mut generated_path = PathBuf::from(out_dir);
            generated_path.push("osmpbf.rs");
            std::fs::copy("./src/osm/pbf/generated/prost-osmpbf.rs", generated_path).map(|_| ())
        }
        Some(_) => {
            let protos = [
                "./src/osm/pbf/format/fileformat.proto",
                "./src/osm/pbf/format/osmformat.proto"
            ];

            let includes = [
                "src/"
            ];
            prost_build::compile_protos(&protos, &includes)
        }
    }
}
