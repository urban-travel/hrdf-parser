mod hrdf;
mod models;
mod parsing;
mod storage;
mod utils;

pub use hrdf::Hrdf;
pub use models::*;
pub use storage::DataStorage;
pub use utils::timetable_end_date;
pub use utils::timetable_start_date;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parsing() {
        let _hrdf = Hrdf::new(
            Version::V_5_40_41_2_0_6,
            "https://opentransportdata.swiss/dataset/eb7fc17b-5676-47f8-a8a1-dde9b00f76c7/resource/b027ea9b-0c00-46d0-b98e-fa5197c8e533/download/oev_sammlung_ch_hrdf_5_40_41_2024_20240910_161007.zip",
            false,
        )
        .await
        .unwrap();
    }
}
