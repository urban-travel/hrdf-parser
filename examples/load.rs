//! Loads a local HRDF zip and prints what it contains:
//! `cargo run --release --example load -- path/to/hrdf.zip`

use std::time::Instant;

use hrdf_parser::Timetable;

#[tokio::main]
async fn main() -> Result<(), hrdf_parser::Error> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let path = std::env::args().nth(1).expect("usage: load <hrdf.zip>");
    let started = Instant::now();
    let tt = Timetable::path(&path).cache_dir("cache").load().await?;
    println!(
        "Loaded in {:?}: {} to {}",
        started.elapsed(),
        tt.start,
        tt.end
    );
    println!(
        "{} stops, {} journeys, {} stop times, {} bit fields, {} footpaths, {} through services",
        tt.stops.len(),
        tt.journeys.len(),
        tt.journeys.iter().map(|j| j.stops.len()).sum::<usize>(),
        tt.bit_fields.len(),
        tt.footpaths.len(),
        tt.through_services.len()
    );
    let calls = tt.journeys.iter().flat_map(|j| &j.stops);
    let with_platform = calls.filter(|c| c.platform.is_some()).count();
    println!(
        "{} platforms, {with_platform} stop times with a platform, {} transport names",
        tt.platforms.len(),
        tt.transport_names.len()
    );
    Ok(())
}
