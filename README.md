[![Crates.io](https://img.shields.io/crates/v/hrdf-parser.svg)](https://crates.io/crates/hrdf-parser)
[![Documentation](https://docs.rs/hrdf-parser/badge.svg)](https://docs.rs/hrdf-parser/)
[![Codecov](https://codecov.io/github/florianburgener/hrdf-parser/coverage.svg?branch=master)](https://codecov.io/gh/florianburgener/hrdf-parser)
[![Dependency status](https://deps.rs/repo/github/florianburgener/hrdf-parser/status.svg)](https://deps.rs/repo/github/florianburgener/hrdf-parser)

# HRDF Parser

This library is dedicated to the parsing of the HRDF format. For the moment, it can only parse the Swiss version of the HRDF format.

Author: Florian Burgener

[https://crates.io/crates/hrdf-parser](https://crates.io/crates/hrdf-parser)

## Prerequisites

* Rust Toolchain (https://www.rust-lang.org/tools/install)
* OpenSSL (`apt install libssl-dev` on Ubuntu)

## Installation

```sh
cargo add hrdf-parser
```

## Usage

```rs
let hrdf = Hrdf::new(
    Version::V_5_40_41_2_0_5,
    "https://opentransportdata.swiss/en/dataset/timetable-54-2024-hrdf/permalink",
    false,
    true,
)
.await?;
```

## Supported HRDF format versions

HRDF 5.40.41, V 2.04 (38 fichiers) :
* ATTRIBUT
* ATTRIBUT_DE (file not used)
* ATTRIBUT_EN (file not used)
* ATTRIBUT_FR (file not used)
* ATTRIBUT_IT (file not used)
* BAHNHOF
* BETRIEB_DE
* BETRIEB_EN
* BETRIEB_FR
* BETRIEB_IT
* BFKOORD_LV95
* BFKOORD_WGS
* BFPRIOS
* BHFART (file not used)
* BHFART_60
* BITFELD
* DURCHBI
* ECKDATEN
* FEIERTAG
* FPLAN
* GLEIS
* GLEIS_LV95
* GLEIS_WGS
* GRENZHLT (file not used)
* INFOTEXT_DE
* INFOTEXT_EN
* INFOTEXT_FR
* INFOTEXT_IT
* KMINFO
* LINIE
* METABHF
* RICHTUNG
* UMSTEIGB
* UMSTEIGL
* UMSTEIGV
* UMSTEIGZ
* ZUGART
* ZEITVS (file not used)

HRDF 5.40.41, V 2.04 (this version also contains the 38 files listed above) :
* GLEISE_LV95 (file not used)
* GLEISE_WGS (file not used)
