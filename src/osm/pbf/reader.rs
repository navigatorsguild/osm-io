use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use command_executor::shutdown_mode::ShutdownMode;
use command_executor::thread_pool_builder::ThreadPoolBuilder;

use crate::osm::model::element::Element;
use crate::osm::pbf::blob_iterator::BlobIterator;
use crate::osm::pbf::element_iterator::ElementIterator;
use crate::osm::pbf::file_block_iterator::FileBlockIterator;
use crate::osm::pbf::file_info::FileInfo;
use crate::osm::pbf::parallel_element_iteration_command::ParallelElementIterationCommand;

#[derive(Debug, Clone)]
pub struct Reader {
    supported_features: Vec<String>,
    path: PathBuf,
    info: FileInfo,
}

/// *.osm.pbf file reader
///
/// Prepare the *.osm.pbf file for reading. The actual reading is performed by associated iterators.
impl Reader {
    /// Create a new Reader
    ///
    /// * path - a path to a valid *.osm.pbf file
    /// Example:
    /// ```
    /// use std::path::PathBuf;
    /// use osm_io::osm::pbf::reader::Reader;
    /// let input_path = PathBuf::from("./planet.osm.pbf");
    /// let reader = Reader::new(&input_path);
    /// ```
    pub fn new(path: &PathBuf) -> Result<Reader, anyhow::Error> {
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
            anyhow!("Failed to parse file header")
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

    pub(crate) fn blobs(&self) -> Result<BlobIterator, anyhow::Error> {
        BlobIterator::new(self.path.clone())
    }

    /// Low level [FileBlockIterator] used to access the sequence of underlying PBF blocks
    pub fn blocks(&self) -> Result<FileBlockIterator, anyhow::Error> {
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

    /// Iterator used to iterate over elements.
    /// Example:
    /// ```
    /// use std::path::PathBuf;
    /// use osm_io::osm::model::element::Element;
    /// use osm_io::osm::pbf;
    /// fn example() -> Result<(), anyhow::Error> {
    ///     let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    ///     let reader = pbf::reader::Reader::new(&input_path)?;
    ///
    ///     let mut nodes = 0 as usize;
    ///     let mut ways = 0 as usize;
    ///     let mut relations = 0 as usize;
    ///
    ///     for element in reader.elements()? {
    ///         match element {
    ///             Element::Node { node } => {
    ///                 nodes += 1;
    ///             }
    ///             Element::Way { way } => {
    ///                 ways += 1;
    ///             }
    ///             Element::Relation { relation } => {
    ///                 relations += 1;
    ///             }
    ///             Element::Sentinel => {
    ///             }
    ///         }
    ///     }
    ///
    ///     println!("nodes: {}", nodes);
    ///     println!("ways: {}", ways);
    ///     println!("relations: {}", relations);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn elements(&self) -> Result<ElementIterator, anyhow::Error> {
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

    /// Parallel iteration over elements in a *.osm.pbf file
    ///
    /// Note that because of the parallel access the order of elements enforced by *.osm.pbf format
    /// is lost.
    /// Example:
    /// ```
    /// use std::path::PathBuf;
    /// use std::sync::Arc;
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// use osm_io::osm::model::element::Element;
    /// use osm_io::osm::pbf;
    /// fn example() -> Result<(), anyhow::Error> {
    ///     let input_path = PathBuf::from("./tests/fixtures/malta-230109.osm.pbf");
    ///     let reader = pbf::reader::Reader::new(&input_path)?;
    ///
    ///     let nodes = Arc::new(AtomicUsize::new(0));
    ///     let ways = Arc::new(AtomicUsize::new(0));
    ///     let relations = Arc::new(AtomicUsize::new(0));
    ///
    ///     let nodes_clone = nodes.clone();
    ///     let ways_clone = ways.clone();
    ///     let relations_clone = relations.clone();
    ///
    ///     reader.parallel_for_each(4, move |element| {
    ///         match element {
    ///             Element::Node { node: _ } => {
    ///                 nodes.fetch_add(1, Ordering::Relaxed);
    ///             }
    ///             Element::Way { .. } => {
    ///                 ways.fetch_add(1, Ordering::Relaxed);
    ///             }
    ///             Element::Relation { .. } => {
    ///                 relations.fetch_add(1, Ordering::Relaxed);
    ///             }
    ///             Element::Sentinel => {}
    ///             }
    ///             Ok(())
    ///         },
    ///     )?;
    ///
    ///     println!("nodes: {}", nodes_clone.load(Ordering::Relaxed));
    ///     println!("ways: {}", ways_clone.load(Ordering::Relaxed));
    ///     println!("relations: {}", relations_clone.load(Ordering::Relaxed));
    ///     Ok(())
    /// }
    /// ```
    pub fn parallel_for_each(&self, tasks: usize, f: impl Fn(Element) -> Result<(), anyhow::Error> + Send + Sync + 'static) -> Result<(), anyhow::Error> {
        let mut iteration_pool = ThreadPoolBuilder::new()
            .with_tasks(tasks)
            .with_queue_size(1024)
            .with_shutdown_mode(ShutdownMode::CompletePending)
            .with_name_str("parallel-element-iterator")
            .build()?;

        let f_wrapper = Arc::new(f);
        for blob_desc in self.blobs()? {
            let f_wrapper_clone = f_wrapper.clone();
            iteration_pool.submit(
                Box::new(
                    ParallelElementIterationCommand::new(blob_desc, f_wrapper_clone)
                )
            );
        }

        iteration_pool.shutdown();
        iteration_pool.join()?;
        Ok(())
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

    fn verify_supported_features(supported_features: &Vec<String>, required_features: &Vec<String>) -> Result<(), anyhow::Error> {
        let missing_features = Self::find_missing_features(supported_features, required_features);
        if missing_features.is_empty() {
            Ok(())
        } else {
            Err(
                anyhow!("Unsupported features: {missing_features:?}")
            )
        }
    }

    /// List the features supported by this [Reader]
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
