pub type Fail = git2::Error;

pub type Attempt = Result<(), Fail>;

pub type Maybe<T> = Result<T, Fail>;

pub fn inner_fail(message: &str) -> Fail {
    git2::Error::from_str(message)
}

pub fn fail<T>(message: &str) -> Maybe<T> {
    Err(inner_fail(message))
}
