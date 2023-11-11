use log::debug;

pub fn resolve_base(specified_base: &Option<String>) -> Result<String, ()> {
    debug!("resolve_base");
    if let Some(specified_base) = specified_base {
        if specified_base.len() > 0 {
            return Ok(specified_base.to_string());
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
        return Ok(process_base);
    }

    return Ok("main".to_string());
}
