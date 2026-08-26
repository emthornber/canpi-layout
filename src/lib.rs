//! # canpi-layout
//!
//! A module to provide functionality to read and validate the canpi layout
//! display configuration and the CBUS linkage items.
//!
//! 13 July, 2026 - E M Thornber
//! Created
//!
//! 7 August, 2026 - E M Thornber
//! Added file path of layout definition file to LocalPanel struct
//!

use schemars::Schema;
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use serde_json::Value;

use core::str;
use glob::MatchOptions;
use glob::glob_with;
use std::path::PathBuf;
use std::{fs::File, io::BufReader, path::Path, string::String};

use log::error;
use thiserror::Error;

fn create_json_schema(schema: Schema) -> Value {
    let schema_string = serde_json::to_string(&schema).unwrap();
    let json_value: Value =
        serde_json::from_slice(schema_string.as_bytes()).expect("convert schema to json");
    json_value
}

#[derive(Error, Debug)]
/// Categorizes the cause of errors when processing the configuration files
pub enum LayoutError {
    /// The error was caused by a failure to read the configuration file
    #[error("cannot open configuration file")]
    Io(#[from] std::io::Error),
    /// The error was caused by failure to validate JSON input
    #[error("JSON input '{0}' failed to validate against schema")]
    Schema(String),
    /// The error was caused by a failure to deserialize the JSON
    #[error("cannot deserialize configuration file")]
    Json(#[from] serde_json::Error),
    /// The error was caused when reading the diagram definition file list
    #[error("cannot read diagram definition file names")]
    Glob(#[from] glob::GlobError),
    /// The error was caused by a lack of item definitions
    #[error("CBus structure not properly initialised")]
    CBus(),
}

impl std::convert::From<jsonschema::ReferencingError> for LayoutError {
    fn from(err: jsonschema::ReferencingError) -> Self {
        LayoutError::Schema(err.to_string())
    }
}

///
/// CBus Link Definitions
///
#[derive(Clone, Deserialize, Debug, JsonSchema, PartialEq)]
/// Defines the possible states of a CBus event
pub enum State {
    /// The state of the event is unknown
    UNKN,
    /// The state of the event is zero
    ZERO,
    /// The state of the event is one
    ONE,
}

#[derive(Clone, Deserialize, Debug, JsonSchema)]
/// Definition of a CBus Link item, which is a name, possibly a CBus event, and
/// its current state
pub struct CBusLink {
    /// Name of the CBus event used by Panel details
    pub name: String,
    /// Definition of the event to be sent / received from the CBus
    pub event: Option<String>,
    /// Current state of event.
    pub state: State,
}

///
/// Panel Definitions
///
/// Coordinates of a tile in the mosaic
#[derive(Clone, Deserialize, Debug, JsonSchema, PartialEq)]
pub struct Tile {
    x_coord: u32,
    y_coord: u32,
}

/// Names of the CBus events that indicate Normal and Reverse positions
/// of a turnout
#[derive(Clone, Deserialize, Debug, JsonSchema, PartialEq)]
pub struct TOState {
    normal: String,
    reverse: String,
}

/// Type of switch - either a toggle switch or a push button
#[derive(Clone, Deserialize, Debug, JsonSchema, PartialEq)]
pub enum SwitchType {
    Toggle,
    PushButton,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug, JsonSchema)]
/// Definition of a control in the mosaic
pub struct Control {
    /// Coordinates of the control in the mosaic
    tile: Tile,
    /// Name of the control in the mosaic
    name: String,
    /// Type of switch
    switch: SwitchType,
    /// Name of CBus event to use when the control is activated
    action: String,
    /// Names of the CBus events that indicate turnout position (Normal and Reverse)
    tostate: TOState,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug, JsonSchema)]
/// Definition of the mosaic for the panel
pub struct Mosaic {
    /// Width of the mosaic in tiles
    width: u32,
    /// Height of the mosaic in tiles
    height: u32,
    /// Size of each (square) tile in pixels
    tilesize: u32,
    /// Colour of the tile background as a hexadecimal string
    /// (e.g. "#000000" for black)
    colour: String,
    /// Margin around the mosaic when drawn (pixels)
    margins: u32,
    /// Width of black border around the mosaic (pixels)
    border: u32,
    /// Title of the layout represented by the Panel
    title: String,
}

/// Definition of the direction of a track in the mosaic
/// (e.g. North-South, East-West, etc.)
#[derive(Clone, Deserialize, Debug, JsonSchema, PartialEq)]
pub enum Direction {
    EW,
    NE,
    NS,
    NW,
    SE,
    SW,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug, JsonSchema)]
pub struct Track {
    /// Coordinates of the track segment in the mosaic
    tile: Tile,
    /// Direction of the track segment in the mosaic
    direction: Direction,
    /// Track circuit name
    label: Option<String>,
    /// Name of the CBus event that shows track circuit state
    tcstate: Option<String>,
    /// Name of the CBus event that shows the occupancy of a specific track section
    spot: Option<String>,
}

/// Definition of the direction of a turnout in the mosaic
/// (e.g. Left, Right, Wye)
#[derive(Clone, Deserialize, Debug, JsonSchema, PartialEq)]
pub enum Hand {
    Left,
    Right,
    Wye,
}

/// Definition of the orientation of a turnout in the mosaic
/// (e.g. North, East, South, West)
#[derive(Clone, Deserialize, Debug, JsonSchema, PartialEq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Deserialize, Debug, JsonSchema, PartialEq)]
pub struct TurnOut {
    /// Coordinates of the turnout in the mosaic
    tile: Tile,
    /// Name of the turnout in the mosaic
    name: String,
    /// Direction of the turnout in the mosaic
    hand: Hand,
    /// Orientation of the turnout in the mosaic
    orientation: Orientation,
    /// Names of the CBus events that indicate turnout position (Normal and Reverse)
    tostate: TOState,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize, Debug, JsonSchema)]
/// Definition of the panel details
pub struct PanelDetails {
    /// Definition of the mosaic dimensions and appearance
    mosaic: Mosaic,
    /// Definitions of the controls in the mosaic
    controls: Vec<Control>,
    /// Definitions of the tracki segments in the mosaic
    tracks: Vec<Track>,
    /// Definitions of the turnouts in the mosaic
    turnouts: Vec<TurnOut>,
}

#[derive(Clone, Deserialize, Debug, JsonSchema)]
/// Definition of the Layout details
pub struct LayoutDetails {
    pub cbus: Option<Vec<CBusLink>>,
    pub panel: Option<PanelDetails>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Layout {
    schema: Value,
    pub layout: Option<LayoutDetails>,
}

impl Layout {
    /// Create a new instance of the structure
    ///
    /// The type definition of Layout is used to create a compiled JSON schema
    /// that will be used to validate the Layout definitions being loaded to
    /// Layout
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(layout_path: P) -> Layout {
        let schema = Self::create_layout_schema();
        let layout = Self::load_layout(layout_path, &schema);
        Layout { schema, layout }
    }

    /// Create a compiled JSON schema from Layout definition
    fn create_layout_schema() -> Value {
        let layout_schema = schema_for!(LayoutDetails);
        create_json_schema(layout_schema)
    }

    /// Load the Layout definitions from `layout_path`
    fn load_layout<P: AsRef<Path> + std::fmt::Debug>(
        layout_path: P,
        schema: &Value,
    ) -> Option<LayoutDetails> {
        let attr = Self::read_layout_file(layout_path, schema);
        match attr {
            Ok(defn) => Some(defn),
            Err(e) => {
                error!("{}", e);
                None
            }
        }
    }

    /// Read the contents of a file as JSON and, if valid against the schema, return an instance
    /// of 'LayoutDetails'
    fn read_layout_file<P: AsRef<Path> + std::fmt::Debug>(
        path: P,
        schema: &Value,
    ) -> Result<LayoutDetails, LayoutError> {
        // Open the file in read-only mode with buffer
        let f = File::open(path.as_ref());
        match f {
            Ok(file) => {
                let reader = BufReader::new(file);
                if let Ok(json_value) = serde_json::from_reader(reader) {
                    if jsonschema::is_valid(schema, &json_value) {
                        // Read the JSON contents of the file as an instance of 'LayoutDetails'.
                        if let Ok(layout) = serde_json::from_value(json_value) {
                            Ok(layout)
                        } else {
                            error!("conversion to struct failed for {:?}", path);
                            Err(LayoutError::Schema(
                                "(failed to convert JSON to struct)".to_string(),
                            ))
                        }
                    } else {
                        let pathstr = path.as_ref().to_str().unwrap();
                        if let Ok(validator) = jsonschema::validator_for(schema) {
                            let result = validator.iter_errors(&json_value);
                            for error in result {
                                error!("{}", error)
                            }
                            error!("{} failed validation", pathstr);
                        }
                        Err(LayoutError::Schema(pathstr.to_string()))
                    }
                } else {
                    error!("reading file {:?} as json failed", path);
                    Err(LayoutError::Schema("(non-utf8 path)".to_string()))
                }
            }
            Err(e) => Err(LayoutError::Io(e)),
        }
    }
}

#[cfg(test)]
mod test_cbus {
    use super::*;
    use env_logger::Target;
    use log::{LevelFilter, error, info};

    const GOOD_ITEM_DATA_1: &str = r#"
        name=101R
        event=N5E5
        state=UNKN
        "#;

    const GOOD_ITEM_DATA_2: &str = r#"
        name=OOU1
        state=ZERO
        "#;

    fn init_logging() {
        let _ = env_logger::builder()
            .target(Target::Stdout)
            .filter_level(LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    #[test]
    fn single_attribute_1() {
        // Some JSON input data as a &str.  Maybe this comes from a file.

        // Initialise Logger
        init_logging();

        // Parse the string of data into an CBusLink object.
        let a: Result<CBusLink, serde_json::Error> = serde_json::from_str(GOOD_ITEM_DATA_1);
        match a {
            Ok(a) => {
                info!("CBusLink is {:#?} ({:#?} {:#?})", a.name, a.event, a.state);
                assert_eq!(a.state, State::UNKN);
            }
            Err(e) => error!("{}: Failed to deserialize", e),
        }
    }

    #[test]
    fn single_attribute_2() {
        // Some JSON input data as a &str.  Maybe this comes from a file.

        // Initialise Logger
        init_logging();

        // Parse the string of data into an CBusLink object.
        let a: Result<CBusLink, serde_json::Error> = serde_json::from_str(GOOD_ITEM_DATA_2);
        match a {
            Ok(a) => {
                info!("CBusLink is {:#?} ({:#?})", a.event, a.state);
                assert_eq!(a.state, State::UNKN);
            }
            Err(e) => error!("{}: Failed to deserialize", e),
        }
    }

    #[test]
    #[ignore = "verbose output"]
    fn view_generated_schema() {
        // Initialise Logger
        init_logging();

        let cbus_schema = schema_for!(CBusLink);
        info!("{}", serde_json::to_string_pretty(&cbus_schema).unwrap());
    }
}

#[cfg(test)]
mod test_layout {
    use super::*;
    use dotenv::dotenv;
    use env_logger::Target;
    use log::{LevelFilter, error, info};
    use std::io::Write;
    use std::{env, fs};

    const GOOD_PANEL_DATA: &str = r#"
    {
        "cbusstates": [
            {
                "name": "101",
                "event": "N5E5",
                "state": "UNKN"
            },
            {
                "name": "101N",
                "event": "N5E6",
                "state": "UNKN"
            },
            {
                "name": "101R",
                "event": "N5E7",
                "state": "UNKN"
            }
        ],
        "panel": {
            "mosaic": {
                "width": 10,
                "height": 5,
                "tilesize": 20,
                "colour": "a4b887",
                "margins": 5,
                "border": 2,
                "title": "Test Panel"
            },
            "controls": [
                {
                    "tile": { "x_coord": 1, "y_coord": 1 },
                    "name": "Control1",
                    "switch": "Toggle",
                    "action": "N5E5",
                    "tostate": { "normal": "N5E6", "reverse": "N5E7" }
                }
            ],
            "tracks": [
                {
                    "tile": { "x_coord": 2, "y_coord": 2 },
                    "direction": "NS",
                    "label": null,
                    "tcstate": null,
                    "spot": null
                }
            ],
            "turnouts": [
                {
                    "tile": { "x_coord": 3, "y_coord": 3 },
                    "name": "Turnout1",
                    "hand": "Left",
                    "orientation": "North",
                    "tostate": { "normal": "N6E7", "reverse": "N6E8" }
                }
            ]
        }
    }"#;

    const BAD_PANEL_DATA: &str = r#"
    {
        cbusstates: [
            {
                "name": "101R",
                "event": "N5E7",
            }
        ],
        "panel": {
            "mosaic": {
                "width": 10,
                "height": 5,
                "tilesize": 20,
                "colour": "a4b887",
                "margins": 5,
                "border": 2,
                "title": "Test Panel"
            },
            "controls": [
                {
                    "tile": { "x_coord": 1, "y_coord": 1 },
                    "name": "Control1",
                    "switch": "Woggle",
                    "action": "N5E5",
                    "state": { "normal": "N5E6", "reverse": "N5E7" }
                }
            ],
            "tracks": [
                {
                    "tile": { "x_coord": 2, "y_coord": 2 },
                    "direction": "NS",
                    "label": null,
                    "tcstate": null,
                    "spot": null
                }
            ],
            "turnouts": [
                {
                    "tile": { "x_coord": 3, "y_coord": 3 },
                    "name": "Turnout1",
                    "hand": "Left",
                    "orientation": "North",
                    "tostate": { "normal": "N6E7", "reverse": "N6E8" }
                }
            ]
        }
    }"#;

    fn init_logging() {
        let _ = env_logger::builder()
            .target(Target::Stdout)
            .filter_level(LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    fn setup_file<P: AsRef<Path> + std::fmt::Debug>(test_file: P, data: &str) {
        if let Ok(mut f) = File::create(&test_file) {
            if let Err(e) = f.write_all(data.as_bytes()) {
                error!("{}: file {:?} write failed", e, test_file);
            }
        } else {
            error!("file {:?} creation failed", test_file);
        }
    }

    fn teardown_file<P: AsRef<Path> + std::fmt::Debug>(test_file: P) {
        if let Err(e) = fs::remove_file(&test_file) {
            error!("{}: file {:?} deletion failed", e, test_file);
        }
    }

    #[test]
    #[ignore = "verbose output"]
    fn view_generated_schema() {
        // Initialise Logger
        init_logging();

        let layout_schema = schema_for!(LayoutDetails);
        info!("{}", serde_json::to_string_pretty(&layout_schema).unwrap());
    }

    #[test]
    #[should_panic]
    fn read_layout_file_missing() {
        let schema = Layout::create_layout_schema();
        let json_file = "tests/nonexistent_file.json";
        let _p = Layout::read_layout_file(json_file, &schema).unwrap();
    }

    #[test]
    fn read_layout_file_not_valid() {
        // Initialise Logger
        init_logging();

        let defn_file = "scratch/bad_layout_data.json";
        setup_file(&defn_file, BAD_PANEL_DATA);
        let schema = Layout::create_layout_schema();
        let bad_result = Layout::read_layout_file(&defn_file, &schema);
        teardown_file(&defn_file);
        match bad_result {
            Ok(_) => assert!(false),
            Err(e) => {
                error!("{}", e);
                assert!(true);
            }
        }
    }

    #[test]
    /// Test creating a LayoutDetails
    fn read_layout_file_validates() {
        // Initialise Logger
        init_logging();

        let defn_file = "scratch/good_layout_data.json";
        setup_file(&defn_file, GOOD_PANEL_DATA);
        let schema = Layout::create_layout_schema();
        let good_result = Layout::read_layout_file(&defn_file, &schema);
        teardown_file(&defn_file);
        match good_result {
            Ok(_) => assert!(true),
            Err(e) => {
                error!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn load_layout_test() {
        // Initialise Logger
        init_logging();

        dotenv().ok();
        if let Ok(layout_file) = env::var("LAYOUT_FILE") {
            let schema = Layout::create_layout_schema();
            let layout_details = Layout::load_layout(layout_file, &schema);
            match layout_details {
                Some(ld) => match ld.panel {
                    Some(pd) => {
                        assert_eq!(pd.mosaic.width, 11);
                    }
                    None => {
                        panic!("Failed to load panel details");
                    }
                },
                None => {
                    panic!("Failed to load layout details");
                }
            }
        }
    }
}

#[allow(dead_code)]
pub struct LocalPanel {
    pub title: String,
    pub layout: LayoutDetails,
    pub file_path: PathBuf,
}

/// Type alias for list of defined layout definitions
pub type LocalPanelVec = Vec<LocalPanel>;

pub struct LPV {
    pub lpv: LocalPanelVec,
}

impl LPV {
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(layout_path: P) -> LPV {
        let mut layouts: LocalPanelVec = LocalPanelVec::new();
        let options = MatchOptions {
            case_sensitive: false,
            require_literal_leading_dot: true,
            require_literal_separator: true,
        };
        if let Some(pattern) = layout_path.as_ref().to_path_buf().join("*.json").to_str() {
            for entry in glob_with(pattern, options).unwrap().flatten() {
                let l = Layout::new(&entry);
                match l.layout {
                    Some(ld) => match ld.panel {
                        Some(ref p) => {
                            let lp: LocalPanel = LocalPanel {
                                title: p.mosaic.title.clone(),
                                layout: ld,
                                file_path: entry,
                            };
                            layouts.push(lp);
                        }
                        None => continue,
                    },
                    None => continue,
                }
            }
        } else {
            // Report glob error
            error!("Bad path to layout definition files");
        }
        LPV { lpv: layouts }
    }
}

#[cfg(test)]
mod test_lpv {
    use super::*;
    use env_logger::Target;
    use log::{LevelFilter, info};

    fn init_logging() {
        let _ = env_logger::builder()
            .target(Target::Stdout)
            .filter_level(LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    #[test]
    fn lpv_bad_path() {
        init_logging();

        let lpv = LPV::new("panel");

        assert_eq!(lpv.lpv.len(), 0);
    }
    #[test]
    fn lpv_good_path() {
        init_logging();

        let lpv = LPV::new("scratch");

        let item = &lpv.lpv[0];
        if let Some(p) = &item.layout.panel {
            info!("The title of the item is '{:#?}'", p.mosaic.title);
            assert_eq!(p.mosaic.title, item.title);
        } else {
            panic!("no panel details available");
        }
        info!("The path of the item is '{:#?}'", item.file_path);
        assert_eq!(lpv.lpv.len(), 1);
    }
}
