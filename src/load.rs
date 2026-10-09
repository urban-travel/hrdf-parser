use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::{Datelike, NaiveDate, Weekday};

use crate::{Error, Result, Timetable, parse};

/// Where a timetable comes from and how it is cached. Created by [`Timetable::year`],
/// [`Timetable::date`], [`Timetable::url`] or [`Timetable::path`].
pub struct Loader {
    source: Source,
    cache_dir: PathBuf,
    force_rebuild: bool,
}

enum Source {
    Url(String),
    Path(PathBuf),
}

impl Timetable {
    /// Timetable year `year` runs from mid-December of `year - 1` to mid-December of `year`.
    pub fn year(year: i32) -> Loader {
        Self::url(url_for_year(year))
    }

    /// The timetable in force on `date`.
    pub fn date(date: NaiveDate) -> Loader {
        Self::year(timetable_year(date))
    }

    /// An HRDF zip archive to download.
    pub fn url(url: impl Into<String>) -> Loader {
        Loader::new(Source::Url(url.into()))
    }

    /// A local HRDF zip archive.
    pub fn path(path: impl Into<PathBuf>) -> Loader {
        Loader::new(Source::Path(path.into()))
    }
}

impl Loader {
    fn new(source: Source) -> Self {
        Self {
            source,
            cache_dir: PathBuf::from("."),
            force_rebuild: false,
        }
    }

    /// Where the downloaded archive and the parsed timetable are kept (default: current directory).
    pub fn cache_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = dir.into();
        self
    }

    /// Download and parse again even when a cached copy exists.
    pub fn force_rebuild(mut self, yes: bool) -> Self {
        self.force_rebuild = yes;
        self
    }

    pub async fn load(self) -> Result<Timetable> {
        fs::create_dir_all(&self.cache_dir)?;
        let (zip, name) = match self.source {
            Source::Path(path) => {
                let name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into();
                (path, name)
            }
            Source::Url(url) => {
                let name: String = url
                    .trim_start_matches("https://")
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                    .collect();
                let zip = self.cache_dir.join(format!("{name}.zip"));
                if self.force_rebuild || !zip.exists() {
                    download(&url, &zip).await?;
                }
                (zip, name)
            }
        };

        let cache = (self.cache_dir).join(format!("{name}-{}.bin", env!("CARGO_PKG_VERSION")));
        if !self.force_rebuild
            && let Ok(bytes) = fs::read(&cache)
        {
            match postcard::from_bytes(&bytes) {
                Ok(timetable) => return Ok(timetable),
                Err(e) => log::warn!("Ignoring unreadable cache {}: {e}", cache.display()),
            }
        }

        let timetable = parse::timetable(&zip)?;
        let saved = postcard::to_stdvec(&timetable)
            .map_err(Error::from)
            .and_then(|bytes| Ok(fs::write(&cache, bytes)?));
        if let Err(e) = saved {
            log::warn!("Could not write cache {}: {e}", cache.display());
        }
        Ok(timetable)
    }
}

async fn download(url: &str, path: &Path) -> Result<()> {
    log::info!("Downloading {url}");
    let mut response = reqwest::get(url).await?.error_for_status()?;
    let part = path.with_extension("part");
    let mut file = fs::File::create(&part)?;
    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
    }
    fs::rename(part, path)?;
    Ok(())
}

/// Timetables change on the second Sunday of December.
fn timetable_year(date: NaiveDate) -> i32 {
    let change = NaiveDate::from_weekday_of_month_opt(date.year(), 12, Weekday::Sun, 2).unwrap();
    date.year() + i32::from(date >= change)
}

fn url_for_year(year: i32) -> String {
    let archive = |file: &str| {
        format!(
            "https://archive.opentransportdata.swiss/timetable_hrdf/timetable-{year}-hrdf-54/{file}"
        )
    };
    match year {
        2022 => archive("OeV_Sammlung_CH_HRDF_5_40_41_2022_20221207_205110.zip"),
        2023 => archive("OeV_Sammlung_CH_HRDF_5_40_41_2023_20231206_204217.zip"),
        2024 => archive("OeV_Sammlung_CH_HRDF_5_40_41_2024_20241213_205621.zip"),
        2025 => archive("OeV_Sammlung_CH_HRDF_5_40_41_2025_20251205_205244.zip"),
        _ => format!(
            "https://data.opentransportdata.swiss/en/dataset/timetable-54-{year}-hrdf/permalink"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timetable_year_changes_on_the_second_sunday_of_december() {
        let date = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        assert_eq!(timetable_year(date(2025, 12, 13)), 2025);
        assert_eq!(timetable_year(date(2025, 12, 14)), 2026);
        assert_eq!(timetable_year(date(2026, 6, 1)), 2026);
        assert_eq!(timetable_year(date(2026, 12, 13)), 2027);
    }
}
