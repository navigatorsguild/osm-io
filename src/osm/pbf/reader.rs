use std::collections::HashSet;
use std::path::PathBuf;
use rayon::iter::{IterBridge, ParallelBridge};
use crate::error::{GenericError, OsmIoError};
use crate::osm::pbf::file_info::FileInfo;
use crate::osm::pbf::blob_iterator::BlobIterator;
use crate::osm::pbf::element_iterator::ElementIterator;
use crate::osm::pbf::file_block_iterator::FileBlockIterator;

#[derive(Debug, Clone)]
pub struct Reader {
    supported_features: Vec<String>,
    path: PathBuf,
    info: FileInfo,
}

impl Reader {
    pub fn new(path: PathBuf) -> Result<Reader, GenericError> {
        let supported_features = vec![
            "OsmSchema-V0.6".to_string(),
            "DenseNodes".to_string(),
            "HistoricalInformation".to_string(),
            "Sort.Type_then_ID".to_string(),
        ];

        let mut reader = Reader {
            supported_features,
            path: path.clone(),
            info: Default::default(),
        };
        let mut block_iterator = reader.clone().blocks()?;
        let file_block = block_iterator.next().ok_or(
            OsmIoError::as_generic(format!("Failed to parse file header"))
        )?;
        let osm_header = file_block.as_osm_header()?;
        reader.info = osm_header.info().clone();

        Self::verify_supported_features(
            &reader.supported_features,
            &reader.info().required_features(),
        )?;


        Ok(
            reader
        )
    }

    pub fn blobs(&self) -> Result<BlobIterator, GenericError> {
        BlobIterator::new(self.path.clone())
    }

    pub fn parallel_blobs(&self) -> Result<IterBridge<BlobIterator>, GenericError> {
        match BlobIterator::new(self.path.clone()) {
            Ok(mut iterator) => {
                // skip the header. doesn't make sense to include the header in parallel iteration
                iterator.next();
                Ok(iterator.par_bridge())
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn blocks(&self) -> Result<FileBlockIterator, GenericError> {
        match self.blobs() {
            Ok(blob_iterator) => {
                Ok(
                   FileBlockIterator::new(blob_iterator)
                )
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn elements(&self) -> Result<ElementIterator, GenericError> {
        match self.blocks() {
            Ok(file_block_iterator) => {
                Ok(
                   ElementIterator::new(file_block_iterator)
                )
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    fn find_missing_features(supported_features: &Vec<String>, required_features: &Vec<String>) -> Vec<String> {
        let supported: HashSet<&String> = supported_features.into_iter().collect::<HashSet<&String>>();
        let required: HashSet<&String> = required_features.into_iter().collect::<HashSet<&String>>();
        if let true = required.is_subset(&supported) {
            vec![]
        } else {
            required.difference(&supported).into_iter().map(|e| e.clone().clone()).collect::<Vec<String>>()
        }
    }

    fn verify_supported_features(supported_features: &Vec<String>, required_features: &Vec<String>) -> Result<(), GenericError> {
        let missing_features = Self::find_missing_features(supported_features, required_features);
        if missing_features.is_empty() {
            Ok(())
        } else {
            Err(
                OsmIoError::as_generic(format!("Unsupported features: {missing_features:?}"))
            )
        }
    }

    pub fn supported_features(&self) -> &Vec<String> {
        &self.supported_features
    }

    pub fn info(&self) -> &FileInfo {
        &self.info
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_missing_features() {
        let mut supported = vec!["a".to_string(), "b".to_string()];
        let mut required = vec!["a".to_string(), "b".to_string()];
        let mut missing = Reader::find_missing_features(&supported, &required);
        assert!(missing.is_empty());

        supported = vec!["a".to_string()];
        required = vec!["a".to_string(), "b".to_string()];
        missing = Reader::find_missing_features(&supported, &required);
        assert_eq!(missing, vec!["b".to_string()]);


        supported = vec!["a".to_string(), "b".to_string()];
        required = vec!["a".to_string()];
        missing = Reader::find_missing_features(&supported, &required);
        assert!(missing.is_empty());
    }
}
