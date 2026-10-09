[![Crates.io](https://img.shields.io/crates/v/hrdf-parser.svg)](https://crates.io/crates/hrdf-parser)
[![Documentation](https://docs.rs/hrdf-parser/badge.svg)](https://docs.rs/hrdf-parser/)
[![Codecov](https://codecov.io/gh/urban-travel/hrdf-parser/graph/badge.svg?token=IO4NWPBZHQ)](https://codecov.io/gh/urban-travel/hrdf-parser)
[![Dependency status](https://deps.rs/repo/github/urban-travel/hrdf-parser/status.svg)](https://deps.rs/repo/github/urban-travel/hrdf-parser)

# HRDF Parser

Parses the routing-relevant part of Swiss HRDF timetables (HRDF 5.40.41, realisation
specification 2.0.7) published on [opentransportdata.swiss](https://data.opentransportdata.swiss/).

Authors:

* Florian Burgener
* Orestis Malaspinas

## Prerequisites

* Rust Toolchain (<https://www.rust-lang.org/tools/install>)
* OpenSSL (`apt install libssl-dev` on Ubuntu)

## Usage

```rust,no_run
# #[tokio::main(flavor = "current_thread")]
# async fn main() -> Result<(), hrdf_parser::Error> {
use hrdf_parser::Timetable;

// Timetable 2027 runs from 2026-12-13 to 2027-12-11. Also: Timetable::date, ::url, ::path.
let timetable = Timetable::year(2027).cache_dir("cache").load().await?;
println!("{} journeys", timetable.journeys.len());
# Ok(())
# }
```

The archive is downloaded once and the parsed timetable is cached next to it, so later loads take
about two seconds instead of thirty. `force_rebuild(true)` downloads and parses again.

`cargo run --release --example load -- path/to/hrdf.zip` prints what an archive contains.

## Files read

| File | Content |
|---|---|
| ECKDATEN | timetable period |
| BITFELD | service days |
| BAHNHOF, BFKOORD_LV95, BFKOORD_WGS | stops, names, coordinates |
| KMINFO | stops where changing vehicles is forbidden |
| UMSTEIGB, UMSTEIGV, UMSTEIGL, UMSTEIGZ | minimum transfer times (stop, administrations, lines, journey pairs) |
| METABHF | footpaths between stops |
| DURCHBI | through services |
| FPLAN | journeys: stop times, boarding/alighting bans, service days per section, cyclic journeys, line, direction |
| LINIE, RICHTUNG, BETRIEB_DE | line names, direction texts, operator names |
| GLEISE_LV95 | the platform (track, bus bay...) of each call |
| ZUGART | English names of the transport categories |

Not read: ATTRIBUT, BETRIEB_EN/FR/IT, BFPRIOS, BHFART, BITFIELD (same as BITFELD), FEIERTAG,
GLEISE_WGS (same as GLEISE_LV95 with other coordinates), INFOTEXT_*, ZEITVS.
