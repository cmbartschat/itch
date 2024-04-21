use std::env;

use git2::{
    Cred, CredentialType, Error, FetchOptions, ProxyOptions, PushOptions, Remote, RemoteCallbacks,
};

use crate::ctx::Ctx;

fn setup_remote_callbacks<'a>(ctx: &'a Ctx) -> RemoteCallbacks<'a> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks
        .push_update_reference(|_, status| {
            if let Some(error_message) = status {
                Err(Error::from_str(error_message))
            } else {
                Ok(())
            }
        })
        .credentials(|url, username_from_url, allowed_types| {
            if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                Cred::credential_helper(&ctx.repo.config()?, url, username_from_url)
            } else if allowed_types.contains(CredentialType::SSH_KEY) {
                Cred::ssh_key(
                    username_from_url.unwrap(),
                    None,
                    std::path::Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
                    Some("git2023"),
                )
            } else {
                todo!("support for auth type: {allowed_types:?}");
            }
        });

    callbacks
}

fn setup_push_options<'a>(ctx: &'a Ctx) -> PushOptions<'a> {
    let mut options = PushOptions::new();
    options
        .proxy_options(ProxyOptions::new())
        .remote_callbacks(setup_remote_callbacks(ctx));

    options
}

fn setup_fetch_options<'a>(ctx: &'a Ctx) -> FetchOptions<'a> {
    let mut options = FetchOptions::new();
    options
        .proxy_options(ProxyOptions::new())
        .remote_callbacks(setup_remote_callbacks(ctx));

    options
}

fn get_remote(ctx: &Ctx) -> Result<Option<Remote>, Error> {
    let remotes = ctx.repo.remotes()?;
    if remotes.is_empty() {
        return Ok(None);
    }
    if remotes.len() != 1 {
        return Err(Error::from_str("Expected exactly 1 remote."));
    }
    let remote = ctx.repo.find_remote(remotes.get(0).unwrap())?;

    return Ok(Some(remote));
}

pub fn push_branch(ctx: &Ctx, branch: &str) -> Result<(), Error> {
    let remote = get_remote(ctx)?;
    if let Some(mut remote) = remote {
        let remote_prefix = "cmb-";
        let refspec = format!(
            "+refs/heads/{}:refs/heads/{}{}",
            branch, remote_prefix, branch
        );
        remote.push(&[refspec], Some(&mut setup_push_options(ctx)))?;
    }
    Ok(())
}

pub fn pull_main(ctx: &Ctx) -> Result<(), Error> {
    let remote = get_remote(ctx)?;
    if let Some(mut remote) = remote {
        remote.fetch(
            &["main"],
            Some(&mut setup_fetch_options(ctx)),
            Some("Fetch main"),
        )?;
    }
    Ok(())
}

pub fn push_main(ctx: &Ctx) -> Result<(), Error> {
    let remote = get_remote(ctx)?;
    if let Some(mut remote) = remote {
        remote.push(&["refs/heads/main"], Some(&mut setup_push_options(ctx)))?;
    }
    Ok(())
}