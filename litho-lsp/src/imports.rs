use std::collections::HashMap;

use litho_types::Import;
use smol_str::SmolStr;

pub type Imports = HashMap<String, Import>;
pub type ResolvedImports = HashMap<String, Result<SmolStr, String>>;
