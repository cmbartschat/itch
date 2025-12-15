use git2::{ErrorClass, ErrorCode, Repository, RepositoryInitOptions};

use crate::error::{Attempt, fail, inner_fail};

pub fn init_command() -> Attempt {
    match Repository::open_from_env() {
        Ok(_) => {
            return fail!("a repository already exists in this location");
        }
        Err(e) => match (e.class(), e.code()) {
            (ErrorClass::Repository, ErrorCode::NotFound) => {}
            _ => {
                return fail!("unexpected error checking for existing repository");
            }
        },
    }

    let path =
        std::env::current_dir().map_err(|_| inner_fail!("failed to resolve current directory."))?;
    let mut options = RepositoryInitOptions::new();
    options.initial_head("main");
    let repo = Repository::init_opts(path, &options)?;
    let signature = repo.signature()?;
    let message = "Initial commit";
    let tree_builder = repo.treebuilder(None)?;
    let tree = repo.find_tree(tree_builder.write()?)?;
    repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?;
    Ok(())
}
