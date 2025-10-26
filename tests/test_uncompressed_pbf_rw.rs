mod common;

use std::path::PathBuf;
use osm_io::osm::model::element::Element;
use osm_io::osm::pbf;
use osm_io::osm::pbf::compression_type::CompressionType;
use osm_io::osm::pbf::file_info::FileInfo;

#[test]
fn test_uncompressed_pbf_read_write() {
    common::setup();

    let input_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf");
    let output_path = PathBuf::from("./target/results/test-uncompressed-output.osm.pbf");

    // Write uncompressed PBF file
    let reader = pbf::reader::Reader::new(&input_path).unwrap();
    let mut file_info = FileInfo::default();
    file_info.with_writingprogram_str("test_uncompressed_pbf_rw");
    let mut writer = pbf::writer::Writer::from_file_info(
        output_path.clone(),
        file_info,
        CompressionType::Uncompressed,
    ).unwrap();

    writer.write_header().unwrap();

    let mut written_count = 0;
    for element in reader.elements().unwrap() {
        writer.write_element(element).unwrap();
        written_count += 1;
    }

    writer.close().unwrap();

    // Read back the uncompressed file
    let reader2 = pbf::reader::Reader::new(&output_path).unwrap();
    let mut readback_count = 0;
    for element in reader2.elements().unwrap() {
        match element {
            Element::Sentinel => {}
            _ => readback_count += 1,
        }
    }

    assert_eq!(written_count, readback_count,
               "Element count mismatch: written {} but read back {}",
               written_count, readback_count);
}

#[test]
fn test_uncompressed_pbf_roundtrip() {
    common::setup();

    let input_path = PathBuf::from("./tests/fixtures/niue-230109.osm.pbf");
    let temp_path = PathBuf::from("./target/results/test-roundtrip-temp.osm.pbf");

    // First write: read compressed, write uncompressed
    let reader1 = pbf::reader::Reader::new(&input_path).unwrap();
    let mut file_info1 = FileInfo::default();
    file_info1.with_writingprogram_str("test_roundtrip_step1");
    let mut writer1 = pbf::writer::Writer::from_file_info(
        temp_path.clone(),
        file_info1,
        CompressionType::Uncompressed,
    ).unwrap();

    writer1.write_header().unwrap();
    for element in reader1.elements().unwrap() {
        writer1.write_element(element).unwrap();
    }
    writer1.close().unwrap();

    // Second write: read uncompressed, write uncompressed again
    let output_path = PathBuf::from("./target/results/test-roundtrip-output.osm.pbf");
    let reader2 = pbf::reader::Reader::new(&temp_path).unwrap();
    let mut file_info2 = FileInfo::default();
    file_info2.with_writingprogram_str("test_roundtrip_step2");
    let mut writer2 = pbf::writer::Writer::from_file_info(
        output_path.clone(),
        file_info2,
        CompressionType::Uncompressed,
    ).unwrap();

    writer2.write_header().unwrap();
    let mut count2 = 0;
    for element in reader2.elements().unwrap() {
        writer2.write_element(element).unwrap();
        count2 += 1;
    }
    writer2.close().unwrap();

    // Third read: verify the final output
    let reader3 = pbf::reader::Reader::new(&output_path).unwrap();
    let mut count3 = 0;
    for _element in reader3.elements().unwrap() {
        count3 += 1;
    }

    assert_eq!(count2, count3,
               "Roundtrip failed: step2 wrote {} but step3 read {}",
               count2, count3);
}
