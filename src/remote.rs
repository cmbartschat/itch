use std::env;

use git2::{Cred, Error, PushOptions, RemoteCallbacks};

use crate::ctx::Ctx;

pub fn sync_remote(ctx: &Ctx) -> Result<(), Error> {
    let remotes = ctx.repo.remotes()?;
    if remotes.len() != 1 {
        return Err(Error::from_str("Expected exactly 1 remote."));
    }
    let mut remote = ctx.repo.find_remote(remotes.get(0).unwrap())?;

    let head_branch = ctx
        .repo
        .branches(Some(git2::BranchType::Local))?
        .find(|x| x.as_ref().is_ok_and(|f| (&f.0).is_head()));

    if let Some(Ok((branch, _))) = head_branch {
        // let branch = ctx.repo.head("blah", git2::BranchType::Local)?;
        // branch.
        // ctx.repo

        let mut callbacks = RemoteCallbacks::new();

        callbacks.push_update_reference(|name, status| {
            Ok({
                println!("Reference status for {name}, {status:?}");
            })
        });

        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                std::path::Path::new(&format!(
                    "{}/.ssh/id_ed25519.pub",
                    env::var("HOME").unwrap()
                )),
                Some(""),
            )
        });

        let branch_spec = branch.into_reference().name().unwrap().to_string();

        let mut options = PushOptions::new();

        options.remote_callbacks(callbacks);

        remote.push(&[branch_spec], Some(&mut options))?;

        return Ok(());
    } else {
        return Err(Error::from_str("Unable to resolve active branch."));
    }
}
