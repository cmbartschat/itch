use std::{ffi::OsStr, path::PathBuf};

use git2::Error;

pub fn bytes2path(bytes: &[u8]) -> Result<PathBuf, Error> {
    #[cfg(unix)]
    {
        use std::os::unix::prelude::*;
        Ok(PathBuf::from(OsStr::from_bytes(bytes)))
    }
    #[cfg(windows)]
    {
        use std::str;
        match str::from_utf8(bytes) {
            Ok(s) => Ok(PathBuf::from(s)),
            Err(..) => Err("invalid non-unicode path".into()),
        }
    }
}
