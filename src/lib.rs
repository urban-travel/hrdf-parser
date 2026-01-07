#![doc = include_str!("../README.md")]
mod error;
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
    use test_log::test;

    #[test(tokio::test)]
    async fn url_not_found() {
        let hrdf = Hrdf::new(
            Version::V_5_40_41_2_0_6,
            "https://data.opentransportdata.swiss/test-should-not-exists",
            true,
            None,
        )
        .await;
        match hrdf {
            Ok(_) => panic!("should be an error"),
            Err(err) => {
                assert!(
                    err.to_string().to_lowercase().contains("404 not found"),
                    "The error should indicate '404 Not Found'"
                );
            }
        }
    }

    #[test(tokio::test)]
    #[ignore]
    async fn parsing_2020() {
        let _hrdf = Hrdf::new(Version::V_5_40_41_2_0_4, "https://archive.opentransportdata.swiss/timetable_hrdf/timetable-2020-hrdf-54/OeV_Sammlung_CH_HRDF_5_40_41_2020_20201207_074253.zip", true, None)
            .await
            .unwrap();
    }

    #[test(tokio::test)]
    #[ignore]
    async fn parsing_2021() {
        let _hrdf = Hrdf::new(Version::V_5_40_41_2_0_5, "https://archive.opentransportdata.swiss/timetable_hrdf/timetable-2021-hrdf-54/OeV_Sammlung_CH_HRDF_5_40_41_2021_20211204_201926.zip", true, None)
            .await
            .unwrap();
    }

    // #[test(tokio::test)]
    // #[ignore]
    // async fn parsing_2022() {
    //     let _hrdf = Hrdf::new(Version::V_5_40_41_2_0_6, "https://archive.opentransportdata.swiss/timetable_hrdf/timetable-2022-hrdf-54/OeV_Sammlung_CH_HRDF_5_40_41_2022_20221207_205110.zip", true, None)
    //         .await
    //         .unwrap();
    // }

    // #[test(tokio::test)]
    // #[ignore]
    // async fn parsing_2023() {
    //     let _hrdf = Hrdf::new(
    //         Version::V_5_40_41_2_0_6,
    //         "https://archive.opentransportdata.swiss/timetable_hrdf/timetable-2023-hrdf-54/OeV_Sammlung_CH_HRDF_5_40_41_2023_20231206_204217.zip",
    //         true,
    //         None,
    //     )
    //     .await
    //     .unwrap();
    // }
    //
    // #[test(tokio::test)]
    // #[ignore]
    // async fn parsing_2024() {
    //     let _hrdf = Hrdf::new(
    //         Version::V_5_40_41_2_0_6,
    //         "https://archive.opentransportdata.swiss/timetable_hrdf/timetable-2024-hrdf-54/OeV_Sammlung_CH_HRDF_5_40_41_2024_20241213_205621.zip",
    //         true,
    //         None,
    //     )
    //     .await
    //     .unwrap();
    // }
    //
    // #[test(tokio::test)]
    // #[ignore]
    // async fn parsing_2025() {
    //     let _hrdf = Hrdf::new(
    //         Version::V_5_40_41_2_0_7,
    //         "https://archive.opentransportdata.swiss/timetable_hrdf/timetable-2025-hrdf-54/OeV_Sammlung_CH_HRDF_5_40_41_2025_20251209_205941.zip",
    //         true,
    //         None,
    //     )
    //     .await
    //     .unwrap();
    // }
}
