use anyhow::Result;

pub type Attempt = Result<()>;

pub type Maybe<T> = Result<T>;
