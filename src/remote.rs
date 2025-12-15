use std::env;

use git2::{
    Cred, CredentialType, FetchOptions, ProxyOptions, PushOptions, Remote, RemoteCallbacks,
};

use crate::{
    ctx::Ctx,
    error::{Attempt, Maybe, fail},
    print::show_warning,
};

fn get_remote_prefix() -> Maybe<String> {
    match env::var("ITCH_REMOTE_PREFIX") {
        Ok(v) => Ok(v),
        Err(env::VarError::NotPresent) => Ok(whoami::username() + "-"),
        Err(env::VarError::NotUnicode(_)) => fail!("Non-unicode remote prefix specified"),
    }
}

fn setup_remote_callbacks(ctx: &Ctx) -> RemoteCallbacks<'_> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks
        .push_update_reference(|_, status| {
            if let Some(error_message) = status {
                fail!(error_message)
            } else {
                Ok(())
            }
        })
        .credentials(|url, username_from_url, allowed_types| {
            if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                Cred::credential_helper(&ctx.repo.config()?, url, username_from_url)
            } else if allowed_types.contains(CredentialType::SSH_KEY) {
                match username_from_url {
                    Some(user) => Cred::ssh_key_from_agent(user),
                    None => fail!("Username not provided, expecting git@ in ssh URLs"),
                }
            } else {
                todo!("support for auth type: {allowed_types:?}");
            }
        });

    callbacks
}

fn setup_push_options(ctx: &Ctx) -> PushOptions<'_> {
    let mut options = PushOptions::new();
    options
        .proxy_options(ProxyOptions::new())
        .remote_callbacks(setup_remote_callbacks(ctx));

    options
}

fn setup_fetch_options(ctx: &Ctx) -> FetchOptions<'_> {
    let mut options = FetchOptions::new();
    options
        .proxy_options(ProxyOptions::new())
        .remote_callbacks(setup_remote_callbacks(ctx));

    options
}

fn get_remote(ctx: &Ctx) -> Maybe<Option<Remote<'_>>> {
    let remotes = ctx.repo.remotes()?;
    if remotes.is_empty() {
        return Ok(None);
    }
    if remotes.len() == 1 {
        return Ok(Some(ctx.repo.find_remote(remotes.get(0).unwrap())?));
    }
    let origin = ctx.repo.find_remote("origin");
    if let Ok(origin) = origin {
        return Ok(Some(origin));
    }
    fail!("Unable to resolve default remote ('origin') out of multiple options")
}

fn force_push_ref(ctx: &Ctx, local_ref: &str, remote_ref: &str) -> Attempt {
    let remote = get_remote(ctx)?;
    if let Some(mut remote) = remote {
        let refspec = format!("+refs/{local_ref}:refs/{remote_ref}");
        remote.push(&[refspec], Some(&mut setup_push_options(ctx)))?;
    }
    Ok(())
}

pub fn push_branch(ctx: &Ctx, branch: &str) -> Attempt {
    if branch == "main" {
        return push_main(ctx);
    }
    let remote_prefix = get_remote_prefix()?;

    force_push_ref(
        ctx,
        format!("heads/{branch}").as_str(),
        format!("heads/{remote_prefix}{branch}").as_str(),
    )
}

pub fn push_tag(ctx: &Ctx, tag: &str) -> Attempt {
    let remote_prefix = get_remote_prefix()?;
    force_push_ref(
        ctx,
        format!("tags/{tag}").as_str(),
        format!("tags/{remote_prefix}{tag}").as_str(),
    )
}

pub fn pull_main(ctx: &Ctx) -> Attempt {
    match get_remote(ctx)? {
        None => Ok(()),
        Some(mut remote) => {
            remote.fetch(
                &["main"],
                Some(&mut setup_fetch_options(ctx)),
                Some("Fetch main"),
            )?;

            let local_main = ctx.repo.find_branch("main", git2::BranchType::Local)?;

            let mut local_ref = ctx
                .repo
                .find_branch("main", git2::BranchType::Local)?
                .into_reference();

            let remote_commit = ctx
                .repo
                .reference_to_annotated_commit(&local_main.upstream()?.into_reference())?;

            let analysis = ctx
                .repo
                .merge_analysis_for_ref(&local_ref, &[&remote_commit])?
                .0;

            if analysis.is_up_to_date() {
                Ok(())
            } else if analysis.is_fast_forward() {
                local_ref.set_target(remote_commit.id(), "Sync main")?;
                Ok(())
            } else {
                fail!("Local diverges from remote.")
            }
        }
    }
}

pub fn reset_main_to_remote(ctx: &Ctx) -> Attempt {
    let local_main = ctx.repo.find_branch("main", git2::BranchType::Local)?;

    let remote_commit = ctx
        .repo
        .find_branch("origin/main", git2::BranchType::Remote)?
        .into_reference()
        .peel_to_commit()?;

    let needs_reset = local_main.is_head();

    local_main
        .into_reference()
        .set_target(remote_commit.id(), "Reset main to origin/main")?;

    if needs_reset {
        let object = ctx.repo.head()?.peel_to_commit()?.into_object();
        ctx.repo.reset(&object, git2::ResetType::Hard, None)?;
    }

    Ok(())
}

pub fn push_main(ctx: &Ctx) -> Attempt {
    let remote = get_remote(ctx)?;
    if let Some(mut remote) = remote {
        remote.push(&["refs/heads/main"], Some(&mut setup_push_options(ctx)))?;
    }
    Ok(())
}

pub fn try_push_branch(ctx: &Ctx, name: &str) {
    if let Err(e) = push_branch(ctx, name) {
        show_warning(
            ctx,
            &format!(
                "Failed to update remote; continuing anyway ({})",
                e.message()
            ),
        );
    }
}

pub fn try_push_main(ctx: &Ctx) {
    if let Err(e) = push_main(ctx) {
        show_warning(
            ctx,
            &format!("Failed to push remote; continuing anyway ({})", e.message()),
        );
    }
}

pub fn try_pull_main(ctx: &Ctx) {
    if let Err(e) = pull_main(ctx) {
        show_warning(
            ctx,
            &format!("Failed to pull remote; continuing anyway ({})", e.message()),
        );
    }
}

pub fn connect_remote(ctx: &Ctx, url: &str) -> Attempt {
    match get_remote(ctx) {
        Ok(Some(_)) => {
            return fail!("Already have a remote.");
        }
        Ok(None) => {}
        Err(e) => return Err(e),
    }

    let mut remote = ctx.repo.remote("origin", url)?;

    remote.fetch(
        &["main"],
        Some(&mut setup_fetch_options(ctx)),
        Some("Fetch main"),
    )?;
    Ok(())
}

pub fn disconnect_remote(ctx: &Ctx) -> Attempt {
    match get_remote(ctx)? {
        Some(remote) => ctx.repo.remote_delete(remote.name().unwrap())?,
        None => show_warning(ctx, "No remote to disconnect."),
    }

    Ok(())
}

pub fn delete_remote_branch(ctx: &Ctx, name: &str) -> Attempt {
    if name == "main" {
        return fail!("Refusing to delete main branch.");
    }
    match get_remote(ctx)? {
        Some(mut remote) => {
            let prefix = get_remote_prefix()?;
            remote.push(
                &[format!(":refs/heads/{prefix}{name}")],
                Some(&mut setup_push_options(ctx)),
            )?;
        }
        None => return Ok(()),
    }

    Ok(())
}

pub fn try_delete_remote_branch(ctx: &Ctx, name: &str) {
    if let Err(e) = delete_remote_branch(ctx, name) {
        show_warning(
            ctx,
            &format!("Failed to delete branch on remote ({})", e.message()),
        );
    }
}
