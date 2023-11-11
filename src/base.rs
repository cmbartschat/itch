use log::debug;

use crate::branch::{string_to_branch, BranchSpec};

pub fn resolve_base(specified_base: Option<String>) -> Result<BranchSpec, ()> {
    debug!("resolve_base");
    if let Some(specified_base) = specified_base {
        if specified_base.len() > 0 {
            return string_to_branch(&specified_base);
        }
    }
    debug!("Checking iBASE");
    let process_base = match std::env::var("iBASE") {
        Ok(v) => {
            if v.len() > 0 {
                Ok(Some(v))
            } else {
                Ok(None)
            }
        }
        Err(std::env::VarError::NotPresent) => Ok(None),
        _ => Err(()),
    }?;

    debug!("iBASE: {:?}", process_base);

    if let Some(process_base) = process_base {
        return string_to_branch(&process_base);
    }

    return Ok(BranchSpec::Local {
        name: "main".to_string(),
    });
}
