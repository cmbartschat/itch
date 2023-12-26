use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    rc::Rc,
};

use git2::{Commit, Error};
use serde::Deserialize;

use crate::{
    cli::{DeleteArgs, LoadArgs, NewArgs, SaveArgs, SyncArgs},
    command::new::new_command,
    ctx::{init_ctx, Ctx},
};

use axum::{
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use maud::{html, Markup, DOCTYPE};

use super::{
    delete::delete_command, load::load_command, merge::merge_command, prune::prune_command,
    save::save_command, squash::squash_command, sync::sync_command,
};

const STYLES: &'static str = include_str!("ui-styles.css");

fn btn(t: &str, content: &str, disabled: bool) -> Markup {
    html! {
      button.btn type=(t) disabled[disabled] {
         (content)
      }
    }
}

type Args = Option<HashMap<String, String>>;

fn hidden_args(args: &Args) -> Option<Markup> {
    match args {
        None => None,
        Some(map) => Some(html! {
           @for field in map.iter() {
              input type="hidden" name=(field.0) value=(field.1);
           }
        }),
    }
}

fn action_btn(method: &str, action: &str, content: &str, args: &Args, disabled: bool) -> Markup {
    html! {
      form method=(method) action=(action) .inline-form   {
        (hidden_args(args).unwrap_or_default())
        (btn("submit", content, disabled))
      }
    }
}

fn named(name: &str) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("name".to_owned(), name.to_owned());
    map
}

fn branch_entry(name: &str) -> Markup {
    html! {
        li.spaced-across {
          span style="min-width: 8rem" { (name)}
        (action_btn("POST", &format!("/api/load"), "Load", &Some(named(name)), false))
        @if name != "main" {
          (action_btn("POST", &format!("/api/delete"), "Delete", &Some(named(name)), false))
        }
      }
    }
}

struct DashboardInfo {
    current_branch: String,
    unsaved_changes: usize,
    commits_ahead: usize,
    commits_behind: usize,
    other_branches: Vec<String>,
}

async fn render_404() -> Markup {
    html! {
        (DOCTYPE)
        head {
          title {
            "404 | itch ui"
          }
          meta charset="utf-8";
         style {(STYLES)}
        }
        body.spaced-down {
        h1 { "Not found" }

        a href="/" {"Home"}
    }
    }
}

fn count_commits_since(_ctx: &Ctx, older: &Commit, newer: &Commit) -> Result<usize, Error> {
    let mut count: usize = 0;
    let mut current = Rc::from(newer.clone());
    while current.id() != older.id() {
        let next = current.parents().next();
        match next {
            Some(c) => {
                count += 1;
                current = Rc::from(c);
            }
            None => return Err(Error::from_str("Unable to navigate to fork point.")),
        }
    }

    Ok(count)
}

fn load_dashboard_info() -> Result<DashboardInfo, Error> {
    let ctx = init_ctx().unwrap();

    let repo_head = ctx.repo.head()?;

    let head_name_str = repo_head.name().unwrap();

    let head_name = head_name_str[head_name_str.rfind("/").map_or(0, |e| e + 1)..].to_owned();

    let base = "main";

    let base_commit = ctx
        .repo
        .find_branch(&base, git2::BranchType::Local)?
        .into_reference()
        .peel_to_commit()?;

    let head_commit = repo_head.peel_to_commit()?;
    let fork_point = ctx
        .repo
        .find_commit(ctx.repo.merge_base(base_commit.id(), head_commit.id())?)?;

    let base_past_fork = count_commits_since(&ctx, &fork_point, &base_commit)?;
    let head_past_fork = count_commits_since(&ctx, &fork_point, &head_commit)?;

    let other_branches = ctx
        .repo
        .branches(Some(git2::BranchType::Local))?
        .map(|e| e.unwrap().0.name().unwrap().unwrap().to_owned())
        .filter(|e| e != &head_name)
        .collect::<Vec<String>>();

    let unsaved_diff = ctx
        .repo
        .diff_tree_to_workdir(Some(&head_commit.tree()?), None)?;

    Ok(DashboardInfo {
        commits_ahead: head_past_fork,
        commits_behind: base_past_fork,
        current_branch: head_name.to_owned(),
        unsaved_changes: unsaved_diff.deltas().count(),
        other_branches: other_branches,
    })
}

fn render_dashboard(info: &DashboardInfo) -> Markup {
    html! {
        (DOCTYPE)
        head {
            title {
               "itch ui"
            }
            meta charset="utf-8";
            style {(STYLES)}
        }
        body.spaced-down {
            header.spaced-across {
                h1 { "itch ui" }
                (action_btn("GET", "/", "Refresh", &None, false))
            }

            p {"On branch " (info.current_branch)}

            div.spaced-across.start {
                div.spaced-across {
                    (action_btn("POST", "/api/merge", "Merge", &None, info.commits_ahead ==0 || info.commits_behind > 0))
                    (info.commits_ahead)
                    " commits ahead"
                    (action_btn("POST", "/api/squash", "Squash", &None, info.commits_ahead < 2))
                }

                div.spaced-across {
                    (action_btn("POST", "/api/sync", "Sync", &None, info.commits_behind == 0))
                    (info.commits_behind)
                    " commits behind"
                }
            }

            @if info.unsaved_changes > 0 {
                form method="POST" action="/api/save" .inline-form.box {
                    div.spaced-across.end {
                        label {
                            "Message"
                            br;
                            input .in name="message";
                        }
                        (btn("submit", "Save", false))
                    }

                    p {(info.unsaved_changes) " changes"}

                }
            }

            h2 {"New Branch" }

            form method="POST" action="/api/new" .inline-form.box  {
                div.spaced-across.end {
                    label {
                        "Name (optional)"
                        br;
                        input .in name="name";
                    }
                    (btn("submit", "New branch", false))
                }
            }


            @if info.other_branches.len() > 0 {
                h2 {"Other branches"}
                ul.spaced-down {
                    @for b in &info.other_branches {
                        (branch_entry(&b))
                    }
                }
            }

            (action_btn("POST", "/api/prune", "Prune empty branches", &None, false))
        }
    }
}

async fn dashboard() -> impl IntoResponse {
    let info = load_dashboard_info().unwrap();

    return render_dashboard(&info);
}

async fn handle_merge() -> impl IntoResponse {
    let ctx = init_ctx().unwrap();
    merge_command(&ctx).unwrap();
    Redirect::to("/")
}

async fn handle_squash() -> impl IntoResponse {
    let ctx = init_ctx().unwrap();
    squash_command(&ctx).unwrap();
    Redirect::to("/")
}

#[derive(Deserialize, Debug)]
struct SaveForm {
    message: String,
}

async fn handle_save(Form(body): Form<SaveForm>) -> impl IntoResponse {
    let ctx = init_ctx().unwrap();
    save_command(
        &ctx,
        &SaveArgs {
            message: vec![body.message],
        },
        true,
    )
    .unwrap();
    Redirect::to("/")
}

async fn handle_sync() -> impl IntoResponse {
    let ctx = init_ctx().unwrap();
    sync_command(&ctx, &SyncArgs { names: vec![] }).unwrap();
    Redirect::to("/")
}

async fn handle_new(Form(body): Form<NewArgs>) -> impl IntoResponse {
    let ctx = init_ctx().unwrap();
    new_command(&ctx, &body).unwrap();

    Redirect::to("/")
}

async fn handle_load(Form(body): Form<LoadArgs>) -> impl IntoResponse {
    let ctx = init_ctx().unwrap();
    load_command(&ctx, &body).unwrap();
    Redirect::to("/")
}

#[derive(Deserialize, Debug)]
struct DeleteForm {
    name: String,
}

async fn handle_delete(Form(body): Form<DeleteForm>) -> impl IntoResponse {
    let ctx = init_ctx().unwrap();
    delete_command(
        &ctx,
        &DeleteArgs {
            names: vec![body.name],
        },
    )
    .unwrap();
    Redirect::to("/")
}

async fn handle_prune() -> impl IntoResponse {
    let ctx = init_ctx().unwrap();
    prune_command(&ctx).unwrap();
    Redirect::to("/")
}

pub async fn ui_command(_ctx: &Ctx) -> Result<(), Error> {
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/api/merge", post(handle_merge))
        .route("/api/squash", post(handle_squash))
        .route("/api/sync", post(handle_sync))
        .route("/api/save", post(handle_save))
        .route("/api/load", post(handle_load))
        .route("/api/delete", post(handle_delete))
        .route("/api/prune", post(handle_prune))
        .route("/api/new", post(handle_new))
        .fallback(render_404);

    let builder = (|| {
        for port in 8000..9000 {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
            match axum::Server::try_bind(&addr) {
                Ok(l) => {
                    return l;
                }
                _ => {}
            }
        }
        panic!("Unable to find unused port");
    })();

    let server = builder.serve(app.into_make_service());

    open::that(format!("http://localhost:{}", server.local_addr().port())).unwrap();

    server.await.unwrap();

    Ok(())
}
