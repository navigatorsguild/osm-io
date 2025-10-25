# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

osm-io is a Rust library for reading and writing OpenStreetMap (OSM) data. It focuses on two primary formats:
- **OSM PBF** (`*.osm.pbf`): A highly efficient binary format for transmitting OSM data
- **apidb**: PostgreSQL database schema used by the OSM website

The library is designed to handle planet-scale datasets (e.g., planet.osm.pbf) with careful attention to memory management and parallel processing.

## Build Commands

```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run a specific test
cargo test test_pbf_reader

# Run benchmarks
cargo bench

# Build documentation
cargo doc --open

# Check code with clippy
cargo clippy
```

## Code Architecture

### Module Structure

The codebase is organized under `src/osm/` with the following key modules:

- **`model/`**: Core OSM data structures (`Node`, `Way`, `Relation`, `Element`, `Tag`, `BoundingBox`, etc.)
- **`pbf/`**: PBF format reading and writing
- **`apidb_dump/`**: PostgreSQL apidb dump reading and writing
- **`converters/`**: Utility converters (e.g., timestamp formatting)

### Key Concepts

#### Element Hierarchy
The `Element` enum (in `model/element.rs`) is the central type representing OSM objects:
- `Element::Node`: Point features with lat/lon coordinates
- `Element::Way`: Polylines/polygons composed of node references
- `Element::Relation`: Collections of nodes, ways, and other relations
- `Element::Sentinel`: Special marker used for iteration control

#### PBF Reading/Writing Pipeline

**Sequential Reading:**
1. `pbf::reader::Reader` opens a `*.osm.pbf` file
2. `BlobIterator` reads file blobs
3. `FileBlockIterator` decodes blobs into data blocks
4. `ElementIterator` yields `Element` objects

**Parallel Reading:**
Use `Reader::elements_parallel()` which leverages the thread pool from `command-executor` crate to process multiple blobs concurrently.

**Sequential Writing:**
1. Create `pbf::writer::Writer` with `FileInfo` and compression type
2. Call `write_header()`
3. Call `write_element()` for each element
4. Call `close()` to finalize

**Parallel Writing:**
Use `pbf::parallel_writer::ParallelWriter` which accumulates elements and writes them in parallel using thread pools.

#### apidb Dump Reading/Writing

**Reading:**
- `apidb_dump::read::Reader` processes PostgreSQL dumps created with `pg_dump --format d`
- Input must be sorted by primary keys (the Reader handles this automatically using external sorting)
- Uses table-specific record types (`NodeRecord`, `WayRecord`, `RelationRecord`, etc.)
- Assembles complete `Element` objects from normalized database tables

**Writing:**
- `apidb_dump::write::Writer` creates directory-format dumps compatible with `pg_restore`
- Decomposes `Element` objects into normalized tables
- Maintains referential integrity for node_tags, way_nodes, way_tags, relation_members, etc.

### Protobuf Code Generation

The PBF format uses Protocol Buffers. The `build.rs` script:
- When building as primary package: generates Rust code from `.proto` files in `src/osm/pbf/format/`
- When used as a dependency: copies pre-generated code from `src/osm/pbf/generated/prost-osmpbf.rs`

### Memory and Parallelism

- **Element Accumulator**: `pbf::element_accumulator::ElementAccumulator` batches elements for efficient parallel writing
- **Thread Pools**: Parallel operations use `command-executor` crate's thread pools
- **String Tables**: `pbf::string_table_builder::StringTableBuilder` deduplicates strings in PBF blocks for compression

## Testing

- Test files are in `tests/` directory
- Fixtures (sample OSM data) are in `tests/fixtures/`
- Test naming convention: `test_<component>_<scenario>.rs`
- Example tests cover full pipelines (e.g., `test_pbf_reader_apidb_dump_writer_pipe.rs`)

## Examples

Examples in `examples/` demonstrate:
- `pbf-io.rs`: Basic PBF reading/writing with filtering
- `parallel-pbf-io.rs`: Parallel PBF processing
- `pbf-to-apidb.rs`: Converting PBF to apidb dump
- `apidb-to-pbf.rs`: Converting apidb dump to PBF
- `count-pbf-elements.rs`: Sequential element counting
- `parallel-count-pbf-elements.rs`: Parallel element counting
- `pbf-info.rs`: Extract metadata from PBF files

Run examples with:
```bash
cargo run --example pbf-io
cargo run --example parallel-count-pbf-elements
```

## Common Patterns

### Reading and Processing PBF Files
```rust
let reader = pbf::reader::Reader::new(&path)?;
for element in reader.elements()? {
    match element {
        Element::Node { node } => { /* process node */ }
        Element::Way { way } => { /* process way */ }
        Element::Relation { relation } => { /* process relation */ }
        Element::Sentinel => { /* end marker */ }
    }
}
```

### Writing PBF Files
```rust
let mut file_info = FileInfo::default();
file_info.with_writingprogram_str("my-program");
let mut writer = pbf::writer::Writer::from_file_info(
    output_path,
    file_info,
    CompressionType::Zlib,
)?;
writer.write_header()?;
writer.write_element(element)?;
writer.close()?;
```

### Converting Between Formats
Use the converters in examples as templates. The general pattern:
1. Create reader for source format
2. Create writer for target format
3. Iterate through reader, write to writer
4. Handle metadata appropriately (FileInfo for PBF, table schemas for apidb)

## Binary Tool

The crate includes a binary tool at `src/bin/osmio.rs`. Build with:
```bash
cargo build --release --bin osmio
```
