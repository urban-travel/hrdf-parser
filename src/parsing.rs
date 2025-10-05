mod attribute_parser;
mod bit_field_parser;
mod direction_parser;
mod error;
mod exchange_administration_parser;
mod exchange_journey_parser;
mod exchange_line_parser;
mod helpers;
mod holiday_parser;
mod information_text_parser;
mod journey_parser;
mod line_parser;
mod platform_parser;
mod stop_connection_parser;
mod stop_parser;
mod through_service_parser;
mod timetable_metadata_parser;
mod transport_company_parser;
mod transport_type_parser;

pub use attribute_parser::parse as load_attributes;
pub use bit_field_parser::parse as load_bit_fields;
pub use direction_parser::parse as load_directions;
pub use exchange_administration_parser::parse as load_exchange_times_administration;
pub use exchange_journey_parser::parse as load_exchange_times_journey;
pub use exchange_line_parser::parse as load_exchange_times_line;
pub use holiday_parser::parse as load_holidays;
pub use information_text_parser::parse as load_information_texts;
pub use journey_parser::parse as load_journeys;
pub use line_parser::parse as load_lines;
pub use platform_parser::parse as load_platforms;
pub use stop_connection_parser::parse as load_stop_connections;
pub use stop_parser::parse as load_stops;
pub use through_service_parser::parse as load_through_service;
pub use timetable_metadata_parser::parse as load_timetable_metadata;
pub use transport_company_parser::parse as load_transport_companies;
pub use transport_type_parser::parse as load_transport_types;

#[cfg(test)]
mod tests {
    use std::error::Error;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use serde::{Deserialize, Serialize};

    pub(crate) fn get_json_values<F>(
        lhs: &F,
        rhs: &str,
    ) -> Result<(serde_json::Value, serde_json::Value), Box<dyn Error>>
    where
        for<'a> F: Serialize + Deserialize<'a>,
    {
        let serialized = serde_json::to_string(&lhs)?;
        println!("{serialized:#?}");
        let reference = serde_json::to_string(&serde_json::from_str::<F>(rhs)?)?;
        Ok((
            serialized.parse::<serde_json::Value>()?,
            reference.parse::<serde_json::Value>()?,
        ))
    }
}
