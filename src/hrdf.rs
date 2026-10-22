use std::{
    env,
    fs::{self, File},
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
    time::Instant,
};
use std::collections::HashMap;
use std::time::Duration;
use crate::{error::{HResult, HrdfError}, models::Version, storage::DataStorage, Line, Stop};
use bincode::config;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;
use zip::ZipArchive;

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum RemovableTypes {
    Line,
    Stop,
    TransportType,
    TransportCompany,
}

pub enum ModifiableTypes<'a> {
    Line{modifications: HashMap<&'a i32, Line>},
    Stop{modifications: HashMap<&'a i32, Stop>},
    TransportType,
    TransportCompany,
}

pub enum AddableTypes {
    Line{add_list: Vec<Line>},
    Stop{add_list: Vec<Stop>},
    TransportType,
    TransportCompany,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hrdf {
    data_storage: DataStorage,
}

impl Hrdf {
    /// Loads and parses an HRDF archive.
    /// If an URL is provided, the HRDF archive (ZIP file) is downloaded automatically. If a path is provided, it must absolutely point to an HRDF archive (ZIP file).
    /// The ZIP archive is automatically decompressed into the temp_dir of the OS folder.
    pub async fn new(
        version: Version,
        url_or_path: &str,
        force_rebuild_cache: bool,
        cache_prefix: Option<String>,
    ) -> HResult<Self> {
        let now = Instant::now();

        let unique_filename = format!("{:x}", Sha256::digest(url_or_path.as_bytes()));
        let cache_path = PathBuf::from(&cache_prefix.unwrap_or(String::from("./")))
            .join(format!("{unique_filename}.cache"));

        let hrdf = if cache_path.exists() && !force_rebuild_cache {
            // Loading from cache.
            log::info!("Loading HRDF data from cache ({cache_path:?})...");

            // If loading from cache fails, None is returned.
            Self::load_from_cache(&cache_path).ok()
        } else {
            // No loading from cache.
            None
        };

        let hrdf = if let Some(hrdf) = hrdf {
            // The cache has been loaded without error.
            hrdf
        } else {
            // The cache must be built.
            // If cache loading has failed, the cache must be rebuilt.
            let compressed_data_path = if Url::parse(url_or_path).is_ok() {
                let compressed_data_path = env::temp_dir().join(format!("{unique_filename}.zip"));

                if !compressed_data_path.exists() {
                    // The data must be downloaded.
                    log::info!("Downloading HRDF data to {compressed_data_path:?}...");
                    let response = reqwest::get(url_or_path).await?.error_for_status()?;
                    let mut file = std::fs::File::create(&compressed_data_path)?;
                    let mut content = Cursor::new(response.bytes().await?);
                    std::io::copy(&mut content, &mut file)?;
                }

                compressed_data_path
            } else {
                PathBuf::from(url_or_path)
            };

            let decompressed_data_path = env::temp_dir().join(unique_filename);

            if !decompressed_data_path.exists() {
                // The data must be decompressed.
                log::info!("Unzipping HRDF archive into {decompressed_data_path:?}...");
                let file = File::open(&compressed_data_path)?;
                let mut archive = ZipArchive::new(BufReader::new(file))?;
                archive.extract(&decompressed_data_path)?;
            }

            log::info!("Parsing HRDF data from {decompressed_data_path:?}...");

            let hrdf = Self {
                data_storage: DataStorage::new(version, &decompressed_data_path)?,
            };

            log::info!("Building cache...");
            hrdf.build_cache(&cache_path)?;
            hrdf
        };

        let elapsed = now.elapsed();

        log::info!("HRDF data loaded in {elapsed:.2?}!");

        Ok(hrdf)
    }

    /// Tries to load an HRDF archive for a specific date by picking the archive which
    /// date range contains the date.
    /// `force_rebuild_cache` and `cache_prefix` are option related to the caching of data.
    pub async fn try_from_date(
        date: NaiveDate,
        force_rebuild_cache: bool,
        cache_prefix: Option<String>,
    ) -> HResult<Self> {
        let url = Version::try_url(date)?;
        let version = Version::try_from(date)?;
        log::info!("Loading Hrdf Version ({version}) and Date ({date}) from url: {url}.");
        Self::new(version, &url, force_rebuild_cache, cache_prefix).await
    }

    /// Tries to load an HRDF archive for a specific year (which is understood as the validity year).
    /// For example year 2026 ranes from (15.12.2025 to 14.12.2026).
    /// `force_rebuild_cache` and `cache_prefix` are option related to the caching of data.
    pub async fn try_from_year(
        year: i32,
        force_rebuild_cache: bool,
        cache_prefix: Option<String>,
    ) -> HResult<Self> {
        let date = NaiveDate::from_ymd_opt(year, 1, 1).ok_or_else(|| HrdfError::InvalidYear)?;
        Self::try_from_date(date, force_rebuild_cache, cache_prefix).await
    }

    // Getters/Setters
    pub fn data_storage(&self) -> &DataStorage {
        &self.data_storage
    }

    // Functions
    pub fn build_cache(&self, path: &Path) -> HResult<()> {
        let data = bincode::serde::encode_to_vec(self, config::standard())?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn load_from_cache(path: &Path) -> HResult<Self> {
        let data = fs::read(path)?;
        let (hrdf, _) = bincode::serde::decode_from_slice(&data, config::standard())?;
        Ok(hrdf)
    }

    pub fn filter(mut self, elements_to_remove: &HashMap<RemovableTypes, Vec<String>>) -> HResult<Self> {
        self.data_storage = self.data_storage.filter(elements_to_remove);
        Ok(self)
    }

    pub fn modify(mut self, elements_to_modify: &ModifiableTypes) -> HResult<Self> {
        self.data_storage = self.data_storage.modify(elements_to_modify);
        Ok(self)
    }

    pub fn add(mut self, elements_to_add: &AddableTypes) -> HResult<Self> {
        self.data_storage = self.data_storage.add(elements_to_add);
        Ok(self)
    }

    pub fn export_csv_time_graph(&self, filter_administration: Option<String>, start_date: &NaiveDate, duration: &Duration, out_file_name: &str){
        let out_file = File::create(out_file_name).unwrap();
        let mut csv_writer = csv::Writer::from_writer(out_file);
        self.data_storage.export_to_csv_time_graph(&mut csv_writer, filter_administration, start_date, duration);
        csv_writer.flush().unwrap();
    }
}
