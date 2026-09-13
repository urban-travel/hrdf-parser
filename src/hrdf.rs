use std::{
    env,
    fs::{self, File},
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
    time::Instant,
};

use crate::{
    error::{HResult, HrdfError},
    models::Version,
    storage::DataStorage,
};
use bincode::config;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;
use zip::ZipArchive;

#[derive(Debug, Serialize, Deserialize)]
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
                    let mut content = Cursor::new(response.bytes().await?);
                    // Download to a temp path and rename into place only on success, so a partial download can't be mistaken for a complete one later.
                    let tmp_path = env::temp_dir().join(format!("{unique_filename}.zip.part"));
                    let mut file = std::fs::File::create(&tmp_path)?;
                    std::io::copy(&mut content, &mut file)?;
                    drop(file);
                    fs::rename(&tmp_path, &compressed_data_path)?;
                }

                compressed_data_path
            } else {
                PathBuf::from(url_or_path)
            };

            let decompressed_data_path = env::temp_dir().join(&unique_filename);

            if !decompressed_data_path.exists() {
                // The data must be decompressed.
                log::info!("Unzipping HRDF archive into {decompressed_data_path:?}...");
                let file = File::open(&compressed_data_path)?;
                let mut archive = ZipArchive::new(BufReader::new(file))?;
                // Same reasoning as above: extract to a temp dir and rename it into place only once extraction fully succeeds.
                let tmp_extract_path = env::temp_dir().join(format!("{unique_filename}.part"));
                if tmp_extract_path.exists() {
                    fs::remove_dir_all(&tmp_extract_path)?;
                }
                archive.extract(&tmp_extract_path)?;
                fs::rename(&tmp_extract_path, &decompressed_data_path)?;
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
        // Write to a temp path and rename into place only on success, so a process killed mid-write can't leave a truncated cache file at `path`.
        let tmp_path = path.with_extension("cache.part");
        fs::write(&tmp_path, data)?;
        fs::rename(&tmp_path, path)?;
        Ok(())
    }

    pub fn load_from_cache(path: &Path) -> HResult<Self> {
        let data = fs::read(path)?;
        let (hrdf, _) = bincode::serde::decode_from_slice(&data, config::standard())?;
        Ok(hrdf)
    }
}
