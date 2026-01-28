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
    use chrono::NaiveDate;
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

    // #[test(tokio::test)]
    // #[ignore]
    // async fn parsing_2020() {
    //     let _hrdf = Hrdf::try_from_date(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(), true, None)
    //         .await
    //         .unwrap();
    // }
    //
    // #[test(tokio::test)]
    // #[ignore]
    // async fn parsing_2021() {
    //     let _hrdf = Hrdf::try_from_date(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(), true, None)
    //         .await
    //         .unwrap();
    // }

    #[test(tokio::test)]
    #[ignore]
    async fn parsing_2022() {
        let _hrdf = Hrdf::try_from_date(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(), true, None)
            .await
            .unwrap();
    }

    #[test(tokio::test)]
    #[ignore]
    async fn parsing_2023() {
        let _hrdf = Hrdf::try_from_date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(), true, None)
            .await
            .unwrap();
    }

    #[test(tokio::test)]
    #[ignore]
    async fn parsing_2024() {
        let _hrdf = Hrdf::try_from_date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), true, None)
            .await
            .unwrap();
    }

    #[test(tokio::test)]
    #[ignore]
    async fn parsing_2025() {
        let _hrdf = Hrdf::try_from_date(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), true, None)
            .await
            .unwrap();
    }

    #[test(tokio::test)]
    #[ignore]
    async fn parsing_2026() {
        let _hrdf = Hrdf::try_from_date(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), true, None)
            .await
            .unwrap();
    }

    #[test(tokio::test)]
    #[ignore]
    async fn parsing_from_year_2026() {
        let _hrdf = Hrdf::try_from_year(2026, false, None).await.unwrap();
    }
}
