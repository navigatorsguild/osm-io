use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum CompressionType {
    Uncompressed,
    Zlib,
}

impl FromStr for CompressionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zlib" => Ok(CompressionType::Zlib),
            "none" | "uncompressed" => Ok(CompressionType::Uncompressed),
            _ => Err(format!(
                "Invalid compression type: '{}'. Valid values are: zlib, none",
                s
            )),
        }
    }
}
