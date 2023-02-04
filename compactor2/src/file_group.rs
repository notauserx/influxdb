//! Utilities for working with groups of [`ParquetFile`]

use data_types::{ParquetFile, Timestamp};
use std::cmp::{max, min};

/// Represent the min/max time range for a group of [`ParquetFile`]s
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilesTimeRange {
    /// min of min_time, across all files
    min_time: Timestamp,
    /// max of max_time, across all files
    max_time: Timestamp,
}

impl FilesTimeRange {
    /// Computes the min/max times across all `files`. Returns `None` if there are no files
    pub fn try_new<'a>(files: impl IntoIterator<Item = &'a ParquetFile>) -> Option<Self> {
        let mut files = files.into_iter();
        let first_file = files.next()?;

        let mut min_time = first_file.min_time;
        let mut max_time = first_file.max_time;
        for file in files {
            min_time = min(min_time, file.min_time);
            max_time = max(max_time, file.max_time);
        }

        Some(Self { min_time, max_time })
    }

    /// Does the min/max time of `file` fall anywhere within min/max
    /// range of self?
    pub fn contains(&self, file: &ParquetFile) -> bool {
        file.min_time <= self.max_time && file.max_time >= self.min_time
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::ParquetFileBuilder;

    #[test]
    fn files_time_range_empty() {
        assert_eq!(FilesTimeRange::try_new(&(vec![] as Vec<ParquetFile>)), None);
    }

    #[test]
    fn files_time_range_non_empty() {
        let files = vec![
            ParquetFileBuilder::new(1).with_time_range(101, 200).build(),
            ParquetFileBuilder::new(2).with_time_range(201, 300).build(),
            ParquetFileBuilder::new(3).with_time_range(100, 250).build(),
        ];
        let range = FilesTimeRange::try_new(&files).unwrap();
        // all input files should be in the range for sure
        assert!(files.iter().all(|f| range.contains(f)));

        // range is 100 --> 300

        // too low
        assert!(!range.contains(&ParquetFileBuilder::new(4).with_time_range(50, 99).build()));
        // on edge
        assert!(range.contains(&ParquetFileBuilder::new(4).with_time_range(50, 100).build()));
        assert!(range.contains(&ParquetFileBuilder::new(4).with_time_range(50, 101).build()));
        // fully within
        assert!(range.contains(&ParquetFileBuilder::new(4).with_time_range(100, 250).build()));

        // on edge going out
        assert!(range.contains(&ParquetFileBuilder::new(4).with_time_range(250, 300).build()));
        // on edge
        assert!(range.contains(&ParquetFileBuilder::new(4).with_time_range(300, 350).build()));
        // too high
        assert!(!range.contains(&ParquetFileBuilder::new(4).with_time_range(301, 350).build()));
    }
}
