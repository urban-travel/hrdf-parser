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

    // #[test(tokio::test)]
    // async fn parsing_2024() {
    //     let _hrdf = Hrdf::new(
    //         Version::V_5_40_41_2_0_6,
    //         "https://data.opentransportdata.swiss/en/dataset/timetable-54-2024-hrdf/permalink",
    //         true,
    //         None,
    //     )
    //     .await
    //     .unwrap();
    // }
    //
    #[test(tokio::test)]
    async fn parsing_2025() {
        let _hrdf = Hrdf::new(
            Version::V_5_40_41_2_0_7,
            "https://data.opentransportdata.swiss/en/dataset/timetable-54-2025-hrdf/permalink",
            true,
            None,
        )
        .await
        .unwrap();
    }
}
