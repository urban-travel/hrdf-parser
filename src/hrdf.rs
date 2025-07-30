use std::{
    env,
    fs::{self, File},
    io::{BufReader, Cursor},
    path::Path,
    time::Instant,
};

use crate::{Result, models::Version, storage::DataStorage};
use bincode::config;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;
use zip::ZipArchive;

#[derive(Debug, Serialize, Deserialize)]
pub struct Hrdf {
    data_storage: DataStorage,
}

impl Hrdf {
    /// Loads and parses the data.<br>
    /// If an URL is provided, the HRDF archive (ZIP file) is downloaded automatically. If a path is provided, it must absolutely point to an HRDF archive (ZIP file).<br>
    /// The ZIP archive is automatically decompressed into the temp_dir of the OS folder.
    pub async fn new(
        version: Version,
        url_or_path: &str,
        force_rebuild_cache: bool,
        cache_prefix: Option<String>,
    ) -> Result<Self> {
        let now = Instant::now();

        let unique_filename = format!("{:x}", Sha256::digest(url_or_path.as_bytes()));
        let cache_path = format!(
            "{}/{unique_filename}.cache",
            cache_prefix.unwrap_or(String::from("./"))
        )
        .replace("//", "/");

        let hrdf = if Path::new(&cache_path).exists() && !force_rebuild_cache {
            // Loading from cache.
            log::info!("Loading HRDF data from cache ({cache_path})...");

            // If loading from cache fails, None is returned.
            Hrdf::load_from_cache(&cache_path).ok()
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
                let compressed_data_path = env::temp_dir()
                    .join(format!("{unique_filename}.zip"))
                    .into_os_string()
                    .into_string()
                    .expect("Could not convert to string.");

                if !Path::new(&compressed_data_path).exists() {
                    // The data must be downloaded.
                    log::info!("Downloading HRDF data to {compressed_data_path}...");
                    let response = reqwest::get(url_or_path).await?;
                    let mut file = std::fs::File::create(&compressed_data_path)?;
                    let mut content = Cursor::new(response.bytes().await?);
                    std::io::copy(&mut content, &mut file)?;
                }

                compressed_data_path
            } else {
                url_or_path.to_string()
            };

            let decompressed_data_path = env::temp_dir()
                .join(unique_filename)
                .into_os_string()
                .into_string()
                .expect("Could not convert to string.");

            if !Path::new(&decompressed_data_path).exists() {
                // The data must be decompressed.
                log::info!("Unzipping HRDF archive into {decompressed_data_path}...");
                let file = File::open(&compressed_data_path)?;
                let mut archive = ZipArchive::new(BufReader::new(file))?;
                archive.extract(&decompressed_data_path)?;
            }

            log::info!("Parsing HRDF data from {decompressed_data_path}...");

            let hrdf = Self {
                data_storage: DataStorage::new(version, &decompressed_data_path)?,
            };

            log::info!("Building cache...");
            hrdf.build_cache(&cache_path)?;
            hrdf
        };

        let elapsed = now.elapsed();

        log::info!("HRDF data loaded in {:.2?}!", elapsed);

        Ok(hrdf)
    }

    // Getters/Setters

    pub fn data_storage(&self) -> &DataStorage {
        &self.data_storage
    }

    // Functions

    pub fn build_cache(&self, path: &str) -> Result<()> {
        let data = bincode::serde::encode_to_vec(self, config::standard())?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn load_from_cache(path: &str) -> Result<Self> {
        let data = fs::read(path)?;
        let (hrdf, _) = bincode::serde::decode_from_slice(&data, config::standard())?;
        Ok(hrdf)
    }
}
