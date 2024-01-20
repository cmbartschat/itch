use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    env,
    hash::{Hash, Hasher},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    rc::Rc,
};

use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use git2::{Commit, Error};
use rand::rngs::OsRng;
use serde::Deserialize;

use crate::{
    cli::{DeleteArgs, LoadArgs, NewArgs, SaveArgs},
    command::new::new_command,
    ctx::{init_ctx, Ctx},
    sync::{Conflict, FullSyncArgs, ResolutionChoice, ResolutionMap},
};

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use maud::{html, Markup, PreEscaped, DOCTYPE};

use super::{
    delete::delete_command, load::load_command, merge::merge_command, prune::prune_command,
    save::save_command, squash::squash_command, sync::sync_command,
};

#[derive(Clone)]
struct CsrfState {
    token: String,
}

const STYLES: PreEscaped<&'static str> = PreEscaped(include_str!("ui-styles.css"));

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

fn branch_entry(info: &DashboardInfo, name: &str) -> Markup {
    html! {
        li.spaced-across {
            span.grow .selected[info.current_branch == name] { (name) }
            (action_btn("POST", &format!("/api/load"), "Load", &Some(named(name)), info.current_branch == name))
            (action_btn("POST", &format!("/api/delete"), "Delete", &Some(named(name)), name == "main" || info.current_branch == name))
        }
    }
}

struct DashboardInfo {
    current_branch: String,
    unsaved_changes: usize,
    commits_ahead: usize,
    commits_behind: usize,
    branches: Vec<String>,
    workspace: String,
}

fn common_head_contents() -> Markup {
    html! {
        link rel="shortcut icon" href=(PreEscaped(&format!("data:image/svg+xml,{}", quick_xml::escape::escape(include_str!("ui-favicon.svg"))))) type="image/svg+xml";
        meta name="viewport" content="width=device-width, initial-scale=1.0";
        meta charset="utf-8";
        style {(STYLES)}
    }
}

fn render_message(title: &str, text: Option<&str>) -> impl IntoResponse {
    html! {
        (DOCTYPE)
        head {
            title {
                (title) " | itch ui"
            }
            (common_head_contents())
        }
        body.spaced-down {
            h1 { (title) }

            @if let Some(text) = text {
                p { (text) }
            }

            a href="/" {"Back"}
        }
    }
}

async fn render_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, render_message("Not found", None))
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
    let ctx = init_ctx()?;

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

    let mut branches = ctx
        .repo
        .branches(Some(git2::BranchType::Local))?
        .map(|e| e.unwrap().0.name().unwrap().unwrap().to_owned())
        .collect::<Vec<String>>();

    branches.sort_unstable();

    let unsaved_diff = ctx
        .repo
        .diff_tree_to_workdir(Some(&head_commit.tree()?), None)?;

    Ok(DashboardInfo {
        commits_ahead: head_past_fork,
        commits_behind: base_past_fork,
        current_branch: head_name.to_owned(),
        unsaved_changes: unsaved_diff.deltas().count(),
        branches: branches,
        workspace: ctx
            .repo
            .workdir()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
    })
}

fn render_dashboard(info: &DashboardInfo) -> Markup {
    html! {
        (DOCTYPE)
        head {
            title {
               (info.workspace) " | itch ui"
            }
            (common_head_contents())
            script {
                (PreEscaped(include_str!("ui.js")))
            }
        }
        body.spaced-down {
            header.spaced-across {
                h1 { (info.workspace) }
                a href="/" { "Refresh" }
            }

            div.spaced-across.start {
                div.spaced-down.big-col {
                    h2 { "Branch: " (info.current_branch) }
                    div.spaced-across {
                        (action_btn("POST", "/api/merge", "Merge", &None, info.commits_ahead == 0 || info.commits_behind > 0))
                        @if info.commits_ahead == 0 {
                           "nothing to merge"
                        } @else if info.commits_behind > 0 {
                            "sync before merging"
                        } @else if info.commits_ahead == 1 {
                            "1 commit"
                        } @else {
                            (info.commits_ahead) " commits"
                        }
                    }

                    div.spaced-across {
                        (action_btn("POST", "/api/squash", "Squash", &None, info.commits_ahead < 2))
                        "to single commit"
                    }

                    div.spaced-across {
                        (action_btn("POST", "/api/sync", "Sync", &None, info.commits_behind == 0))
                        @match info.commits_behind {
                            0 => ("already synced"),
                            1 => ("1 commit behind"),
                            n => {(n) " commits behind"},
                         }
                    }

                    form method="POST" action="/api/sync"  {
                        input type="radio" name="example.txt" value="yours" checked;
                        input type="radio" name="example.txt" value="mine";
                        (btn("submit", "Sync", false))
                    }

                    form method="POST" action="/api/save" {
                        div.spaced-across.end {
                            label {
                                "Save message"
                                br;
                                input .in name="message" placeholder="(optional)" disabled[info.unsaved_changes == 0];
                            }
                            (btn("submit", "Save", info.unsaved_changes == 0))
                        }

                        p {(info.unsaved_changes) " changes"}
                    }
                }

                div.spaced-down.big-col {
                    div.spaced-across {
                        h2 {"All Branches"}
                        (action_btn("POST", "/api/prune", "Prune empty", &None, false))
                    }

                    form method="POST" action="/api/new" .inline-form.spaced-across.end  {
                        label.grow {
                            "New branch"
                            br;
                            input .in name="name" placeholder="(optional)";
                        }
                        (btn("submit", "New branch", false))
                    }

                    ul.spaced-down {
                        @for b in &info.branches {
                            (branch_entry(&info, &b))
                        }
                    }
                }
            }
        }
    }
}

async fn dashboard(jar: CookieJar, State(state): State<CsrfState>) -> impl IntoResponse {
    load_dashboard_info()
        .map(|info| {
            let mut csrf = Cookie::new("_csrf", state.token);
            csrf.set_same_site(SameSite::Strict);
            (jar.add(csrf), render_dashboard(&info))
        })
        .map_err(|err| map_error_to_response(err))
}

fn render_sync(conflicts: &Vec<Conflict>) -> Markup {
    html! {
        (DOCTYPE)
        head {
            title {
                "Sync | itch ui"
            }
            (common_head_contents())
        }
        body.spaced-down {
            h1 { "Sync" }

            form method="POST" action="/api/sync"  {
                @for conflict in conflicts {
                    @match conflict {
                      Conflict::MainDeletion(path) => {
                        fieldset {
                            (path) "has changes, but was deleted on main." br;
                        input type="radio" name=(path) value="keep" checked;
                        input type="radio" name=(path) value="delete";
                        }
                      },
                      Conflict::BranchDeletion(path) => {
                        fieldset {
                            (path)
                             "was deleted on your branch, but was modified on main."
                             br;
                        input type="radio" name=(path) value="keep" checked;
                        input type="radio" name=(path) value="delete";
                        }
                        },
                        Conflict::Merge(info) => {
                            fieldset {
                                (info.branch_path) "has conflicts." br;
                                input type="radio" name=(info.branch_path) value="yours" checked;
                                input type="radio" name=(info.branch_path) value="theirs";

                                details {
                                    p {"main content:"}
                                pre {
                                    code {
                                        (info.main_content)
                                    }
                                }
                                p {"branch content:"}
                                pre {
                                    code {
                                        (info.branch_content)
                                    }
                                }
                                }

                            }
                        }
                        Conflict::OpaqueMerge(_, path) => {
                            fieldset {
                                (path) "has conflicts." br;
                                input type="radio" name=(path) value="yours" checked;
                                input type="radio" name=(path) value="theirs";
                            }
                        }

                      }
                }

                (btn("submit", "Sync", false))
            }

            a href="/" {"Back"}
        }
    }
}

async fn sync() -> impl IntoResponse {
    render_sync(&vec![])
}

fn with_ctx<R, T>(callback: T) -> Result<R, Error>
where
    T: FnOnce(&Ctx) -> Result<R, Error>,
{
    let mut ctx = init_ctx()?;
    ctx.set_mode(crate::ctx::Mode::Background);
    callback(&ctx)
}

fn api_handler<R, T>(callback: T) -> impl IntoResponse
where
    T: FnOnce(&Ctx) -> Result<R, Error>,
{
    map_result_to_response(with_ctx(callback))
}

fn map_error_to_response(err: Error) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        render_message("Error", Some(err.message())),
    )
}

fn map_result_to_response<T>(res: Result<T, Error>) -> impl IntoResponse {
    match res {
        Ok(_) => Redirect::to("/").into_response(),
        Err(e) => map_error_to_response(e).into_response(),
    }
}

async fn handle_merge() -> impl IntoResponse {
    api_handler(merge_command)
}

async fn handle_squash() -> impl IntoResponse {
    api_handler(squash_command)
}

#[derive(Deserialize, Debug)]
struct SaveForm {
    message: String,
}

async fn handle_save(Form(body): Form<SaveForm>) -> impl IntoResponse {
    api_handler(|ctx| {
        save_command(
            ctx,
            &SaveArgs {
                message: vec![body.message],
            },
            true,
        )
    })
}

// #[derive(Deserialize, Debug)]
// struct SyncDecision {
//     path: String,
//     yours: Option<bool>,
//     theirs: Option<bool>,
//     edit: Option<String>,
// }

// #[derive(Deserialize, Debug)]
// struct SyncForm {
//     resolutions: Vec<SyncDecision>,
// }

type SyncForm = HashMap<String, String>;

// fn convert_sync_form(body: &SyncForm) -> Result<FullSyncArgs, Error> {
//     let mut resolutions: ResolutionMap = HashMap::new();

//     // if let Some(res) = &body.resolutions {
//     for decision in &body.resolutions {
//         let resolution = match (decision.yours, decision.theirs, &decision.edit) {
//             (Some(true), None, None) => ResolutionChoice::Yours,
//             (None, Some(true), None) => ResolutionChoice::Theirs,
//             (None, None, Some(str)) => ResolutionChoice::Manual(str.clone()),
//             _ => return Err(Error::from_str("Invalid decision specified.")),
//         };
//         resolutions.insert(decision.path.clone(), resolution);
//     }
//     // }

//     return Ok(FullSyncArgs {
//         names: vec![],
//         resolutions: vec![resolutions],
//     });
// }

fn convert_sync_form(body: &SyncForm) -> Result<FullSyncArgs, Error> {
    let mut resolutions: ResolutionMap = HashMap::new();

    // if let Some(res) = &body.resolutions {
    for (key, value) in body.iter() {
        let value = if value == "yours" {
            ResolutionChoice::Yours
        } else if value == "theirs" {
            ResolutionChoice::Theirs
        } else {
            return Err(Error::from_str("Unable to parse choice."));
        };
        resolutions.insert(key.clone(), value);
    }
    // }

    return Ok(FullSyncArgs {
        names: vec![],
        resolutions: vec![resolutions],
    });
}

async fn handle_sync(Form(body): Form<SyncForm>) -> impl IntoResponse {
    match with_ctx(|ctx| {
        let args = convert_sync_form(&body)?;
        sync_command(&ctx, &args)
    }) {
        Ok(details) => {
            if let Some(crate::sync::SyncDetails::Conflicted(d)) = details.get(0) {
                render_sync(&d).into_response()
            } else {
                Redirect::to("/").into_response()
            }
        }
        Err(e) => map_error_to_response(e).into_response(),
    }
}

async fn handle_new(Form(body): Form<NewArgs>) -> impl IntoResponse {
    api_handler(move |ctx| new_command(&ctx, &body))
}

async fn handle_load(Form(body): Form<LoadArgs>) -> impl IntoResponse {
    api_handler(move |ctx| load_command(&ctx, &body))
}

#[derive(Deserialize, Debug)]
struct DeleteForm {
    name: String,
}

async fn handle_delete(Form(body): Form<DeleteForm>) -> impl IntoResponse {
    api_handler(|ctx| {
        delete_command(
            &ctx,
            &DeleteArgs {
                names: vec![body.name],
            },
        )
    })
}

async fn handle_prune() -> impl IntoResponse {
    api_handler(|ctx| prune_command(&ctx))
}

async fn csrf_check<B>(
    State(state): State<CsrfState>,
    jar: CookieJar,
    request: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    match jar.get("_csrf") {
        Some(cookie) if cookie.value() == state.token => next.run(request).await.into_response(),
        _ => (
            StatusCode::UNAUTHORIZED,
            render_message(
                "Invalid Session",
                Some("This can happen after a restart. Please return to the dashboard and retry."),
            ),
        )
            .into_response(),
    }
}

use base64::Engine;
use rand::RngCore;

pub async fn ui_command(_ctx: &Ctx) -> Result<(), Error> {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);

    let state = CsrfState {
        token: base64::engine::general_purpose::STANDARD_NO_PAD.encode(&key),
    };

    let api = Router::new()
        .route("/merge", post(handle_merge))
        .route("/squash", post(handle_squash))
        .route("/sync", post(handle_sync))
        .route("/save", post(handle_save))
        .route("/load", post(handle_load))
        .route("/delete", post(handle_delete))
        .route("/prune", post(handle_prune))
        .route("/new", post(handle_new))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            csrf_check,
        ));

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/sync", get(sync))
        .nest("/api", api)
        .with_state(state)
        .fallback(render_404);

    let mut hasher = DefaultHasher::new();
    env::current_dir().unwrap().hash(&mut hasher);
    let offset = (hasher.finish() % 1000) as u16;

    let builder = (|| {
        for iteration in 0..100 {
            let port = 8000 + offset + iteration;
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

    // open::that(format!("http://localhost:{}", server.local_addr().port())).unwrap();

    server.await.unwrap();

    Ok(())
}
