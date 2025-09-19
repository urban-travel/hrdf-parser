/// # Direction parsing
///
/// This file contains direction informations: namely the Direction ID (that is also contained in
/// the FPLAN file) and the direction Text which gives the last stop of the traject. From
/// [https://opentransportdata.swiss/en/cookbook/hafas-rohdaten-format-hrdf/#Technical_description_What_is_in_the_HRDF_files_contents](HRDF the docs) we have:
///
/// `R000011 Esslingen    % Richtung 11 nach Esslingen`
///
/// that the direction 11 (R is for Richtung) travels to Esslingen
///
/// 1 file(s).
/// File(s) read by the parser:
/// RICHTUNG
use std::error::Error;

use nom::{IResult, Parser, character::char};
use rustc_hash::FxHashMap;

use crate::{
    models::{Direction, Model},
    parsing::helpers::{direction_parser, read_lines, string_till_eol_parser},
    storage::ResourceStorage,
};

type DirectionAndTypeConverter = (ResourceStorage<Direction>, FxHashMap<String, i32>);

pub fn parse_direction_row(input: &str) -> IResult<&str, (String, i32, String)> {
    let (res, ((prefix, id), _, name)) =
        (direction_parser(), char(' '), string_till_eol_parser()).parse(input)?;
    Ok((res, (prefix, id, name)))
}

fn parse_line(
    line: &str,
    pk_type_converter: &mut FxHashMap<String, i32>,
) -> Result<Direction, Box<dyn Error>> {
    let (_, (prefix, id, name)) =
        parse_direction_row(line).map_err(|e| format!("Failed to parse line '{}': {}", line, e))?;
    let legacy_id = format!("{prefix}{id}");
    if let Some(previous) = pk_type_converter.insert(legacy_id.clone(), id) {
        log::warn!(
            "Warning: previous id {previous} for {legacy_id}. The legacy_id, {legacy_id} is not unique."
        );
    }
    Ok(Direction::new(id, name))
}

pub fn parse(path: &str) -> Result<DirectionAndTypeConverter, Box<dyn Error>> {
    log::info!("Parsing RICHTUNG...");

    let lines = read_lines(&format!("{path}/RICHTUNG"), 0)?;
    let mut pk_type_converter = FxHashMap::default();
    let directions = lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse_line(&line, &mut pk_type_converter))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let directions = Direction::vec_to_map(directions);
    Ok((ResourceStorage::new(directions), pk_type_converter))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::parsing::tests::get_json_values;
    use pretty_assertions::assert_eq;

    #[test]
    fn row_parser_v207() {
        let input = "R000008 Winterthur";
        let (_, (prefix, id, name)) = parse_direction_row(input).unwrap();
        assert_eq!("R", prefix);
        assert_eq!(8, id);
        assert_eq!("Winterthur", name);

        let input = "R000192 Saas-Fee, Parkhaus";
        let (_, (prefix, id, name)) = parse_direction_row(input).unwrap();
        assert_eq!("R", prefix);
        assert_eq!(192, id);
        assert_eq!("Saas-Fee, Parkhaus", name);

        let input = "R002609 Hégenheim - Collège des Trois Pays";
        let (_, (prefix, id, name)) = parse_direction_row(input).unwrap();
        assert_eq!("R", prefix);
        assert_eq!(2609, id);
        assert_eq!("Hégenheim - Collège des Trois Pays", name);
    }

    #[test]
    fn type_converter_v207() {
        let rows = vec![
            "R000008 Winterthur".to_string(),
            "R000192 Saas-Fee, Parkhaus".to_string(),
            "R002609 Hégenheim - Collège des Trois Pays".to_string(),
        ];
        let mut pk_type_converter = FxHashMap::default();
        let directions = rows
            .into_iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| parse_line(&line, &mut pk_type_converter))
            .collect::<Result<Vec<_>, Box<dyn Error>>>()
            .unwrap();
        let directions = Direction::vec_to_map(directions);
        println!("LET'S GO: {pk_type_converter:?}");
        assert_eq!(*pk_type_converter.get("R8").unwrap(), 8);
        assert_eq!(*pk_type_converter.get("R192").unwrap(), 192);
        assert_eq!(*pk_type_converter.get("R2609").unwrap(), 2609);
        let attribute = directions.get(&8).unwrap();
        let reference = r#"
            {
                "id":8,
                "name":"Winterthur"
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        let attribute = directions.get(&192).unwrap();
        let reference = r#"
            {
                "id":192,
                "name":"Saas-Fee, Parkhaus"
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        let attribute = directions.get(&2609).unwrap();
        let reference = r#"
            {
                "id":2609,
                "name":"Hégenheim - Collège des Trois Pays"
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
    }
}
