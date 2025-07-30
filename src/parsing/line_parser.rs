// 1 file(s).
// File(s) read by the parser:
// LINIE

use crate::{
    Result,
    error::ErrorKind,
    models::{Color, Line, Model},
    parsing::{
        ColumnDefinition, ExpectedType, FastRowMatcher, FileParser, ParsedValue, RowDefinition,
        RowParser,
    },
    storage::ResourceStorage,
};

pub fn parse(path: &str) -> Result<ResourceStorage<Line>> {
    log::info!("Parsing LINIE...");
    const ROW_A: i32 = 1;
    const ROW_B: i32 = 2;
    const ROW_C: i32 = 3;
    const ROW_D: i32 = 4;
    const ROW_E: i32 = 5;

    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is used to create a Line instance.
        RowDefinition::new(ROW_A, Box::new(FastRowMatcher::new(9, 1, "K", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(11, -1, ExpectedType::String),
        ]),
        // This row contains the short name.
        RowDefinition::new(ROW_B, Box::new(FastRowMatcher::new(9, 3, "N T", true)), vec![
            ColumnDefinition::new(13, -1, ExpectedType::String),
        ]),
        // This row contains the text color.
        RowDefinition::new(ROW_C, Box::new(FastRowMatcher::new(9, 1, "F", true)), vec![
            ColumnDefinition::new(11, 13, ExpectedType::Integer16),
            ColumnDefinition::new(15, 17, ExpectedType::Integer16),
            ColumnDefinition::new(19, 21, ExpectedType::Integer16),
        ]),
        // This row contains the background color.
        RowDefinition::new(ROW_D, Box::new(FastRowMatcher::new(9, 1, "B", true)), vec![
            ColumnDefinition::new(11, 13, ExpectedType::Integer16),
            ColumnDefinition::new(15, 17, ExpectedType::Integer16),
            ColumnDefinition::new(19, 21, ExpectedType::Integer16),
        ]),
        // This row contains the short name.
        RowDefinition::new(ROW_E, Box::new(FastRowMatcher::new(9, 3, "L T", true)), vec![
            ColumnDefinition::new(13, -1, ExpectedType::String),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/LINIE"), row_parser)?;

    let mut data = Vec::new();

    for x in parser.parse() {
        let (id, _, values) = x?;
        match id {
            ROW_A => {
                data.push(create_instance(values));
            }
            _ => {
                let line = data.last_mut().ok_or(ErrorKind::RowMissing { typ: "A" })?;

                match id {
                    ROW_B => set_short_name(values, line),
                    ROW_C => set_text_color(values, line),
                    ROW_D => set_background_color(values, line),
                    ROW_E => set_long_name(values, line),
                    _ => unreachable!(),
                }
            }
        }
    }

    let data = Line::vec_to_map(data);

    Ok(ResourceStorage::new(data))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(mut values: Vec<ParsedValue>) -> Line {
    let id: i32 = values.remove(0).into();
    let name: String = values.remove(0).into();

    Line::new(id, name)
}

fn set_short_name(mut values: Vec<ParsedValue>, line: &mut Line) {
    let short_name: String = values.remove(0).into();

    line.set_short_name(short_name);
}

fn set_long_name(mut values: Vec<ParsedValue>, line: &mut Line) {
    let long_name: String = values.remove(0).into();

    line.set_long_name(long_name);
}

fn set_text_color(mut values: Vec<ParsedValue>, line: &mut Line) {
    let r: i16 = values.remove(0).into();
    let g: i16 = values.remove(0).into();
    let b: i16 = values.remove(0).into();

    line.set_text_color(Color::new(r, g, b));
}

fn set_background_color(mut values: Vec<ParsedValue>, line: &mut Line) {
    let r: i16 = values.remove(0).into();
    let g: i16 = values.remove(0).into();
    let b: i16 = values.remove(0).into();

    line.set_background_color(Color::new(r, g, b));
}
