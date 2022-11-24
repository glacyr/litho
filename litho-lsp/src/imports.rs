use std::collections::HashMap;
use std::time::Duration;

use smol_str::SmolStr;

pub type Imports = HashMap<String, Duration>;
pub type ResolvedImports = HashMap<String, Result<SmolStr, String>>;
