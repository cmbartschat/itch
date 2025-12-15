pub type Fail = git2::Error;

pub type Attempt = Result<(), Fail>;

pub type Maybe<T> = Result<T, Fail>;

pub use crate::fail;
pub use crate::inner_fail;

mod mac {
    #[macro_export]
    macro_rules! inner_fail {
        ($msg:expr $(,)?) => {
            $crate::error::Fail::from_str($msg)
        };
    }

    #[macro_export]
    macro_rules! fail {
        ($msg:expr $(,)?) => {
            Err($crate::error::inner_fail!($msg))
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test1() {
        let _f: Fail = inner_fail!("x");
        let _g: Attempt = fail!("y");
    }
}
