use std::path::PathBuf;

use crate::error::{fail, Maybe};

pub fn bytes2path(bytes: &[u8]) -> Maybe<PathBuf> {
    #[cfg(unix)]
    {
        use std::os::unix::prelude::*;
        Ok(PathBuf::from(OsStr::from_bytes(bytes)))
    }
    #[cfg(windows)]
    {
// windows!
        use std::str;
        match str::from_utf8(bytes) {
            Ok(s) => Ok(PathBuf::from(s)),
            Err(..) => fail("invalid non-unicode path"),
        }
    }
}
