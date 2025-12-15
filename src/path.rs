use std::path::PathBuf;

use crate::error::Maybe;

#[allow(clippy::unnecessary_wraps)]
pub fn bytes2path(bytes: &[u8]) -> Maybe<PathBuf> {
    #[cfg(unix)]
    {
        use std::ffi::OsStr;
        use std::os::unix::prelude::*;
        Ok(PathBuf::from(OsStr::from_bytes(bytes)))
    }
    #[cfg(windows)]
    {
        use crate::error::fail;
        use std::str;
        match str::from_utf8(bytes) {
            Ok(s) => Ok(PathBuf::from(s)),
            Err(..) => fail!("invalid non-unicode path"),
        }
    }
}
