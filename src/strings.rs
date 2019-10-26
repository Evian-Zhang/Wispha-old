use crate::wispha::core::WisphaEntryType;

pub const DEFAULT_ENTRY_TYPE: WisphaEntryType = WisphaEntryType::File;
pub const DEFAULT_NAME: &str = "default name";
pub const DEFAULT_PATH: &str = "default path";
pub const DEFAULT_FILE_PATH: &str = "default file path";

pub const DEFAULT_FILE_NAME_STR: &str = "LOOKME.wispha";

pub const LINE_SEPARATOR: &str = "\n";

pub const BEGIN_MARK: &str = "+";

pub const DIRECTORY_TYPE: &str = "directory";
pub const FILE_TYPE: &str = "file";
pub const PROGRAM_ENTRY_TYPE: &str = "program entry";

pub const ABSOLUTE_PATH_HEADER: &str = "file path";
pub const NAME_HEADER: &str = "name";
pub const ENTRY_TYPE_HEADER: &str = "entry type";
pub const DESCRIPTION_HEADER: &str = "description";

pub const ENTRY_FILE_PATH_HEADER: &str = "entry file path";
pub const SUB_ENTRIES_HEADER: &str = "subentry";

pub const ROOT_DIR: &str = "$ROOT_DIR";
pub const ROOT_DIR_VAR: &str = "WISPHA_ROOT_DIR";

pub const CONFIG_FILE_NAME: &str = ".wispharc";