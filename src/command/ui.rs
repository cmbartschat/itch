use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    env,
    hash::{Hash, Hasher},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    rc::Rc,
};

use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use git2::{Commit, Delta, DiffDelta, DiffHunk, DiffLine, Patch};
use rand::rngs::OsRng;
use serde::Deserialize;

use crate::{
    branch::get_current_branch,
    cli::{DeleteArgs, LoadArgs, NewArgs, SaveArgs},
    command::new::new_command,
    ctx::{init_ctx, Ctx},
    diff::{collapse_renames, good_diff_options, split_diff_line},
    error::{fail, Attempt, Fail, Maybe},
    reset::pop_and_reset,
    save::save_temp,
    sync::{Conflict, ResolutionChoice, ResolutionMap, SyncDetails},
};

use axum::{
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use maud::{html, Markup, PreEscaped, DOCTYPE};

use super::{
    delete::delete_command,
    load::load_command,
    merge::merge_command,
    prune::prune_command,
    save::save_command,
    squash::squash_command,
    status::{resolve_fork_info, FileStatus, ForkInfo, SegmentedStatus},
    sync::try_sync_branch,
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

fn radio(label: &str, name: &str, value: &str, checked: bool) -> Markup {
    html! {
        label {
            (if checked {
                html! {input type="radio" name=(name) value=(value) checked;}
            } else {
                html! {input type="radio" name=(name) value=(value);}
            })
            (label)
        }
    }
}

fn status_class(delta: &Delta) -> &'static str {
    match delta {
        Delta::Added | Delta::Untracked => "status-added",
        Delta::Deleted => "status-deleted",
        Delta::Modified => "status-modified",
        Delta::Renamed => "status-renamed",
        _ => "status-other",
    }
}

fn render_file_status(status: &SegmentedStatus, work_status: &FileStatus) -> Markup {
    let display_name = status.get_work_rename_chain().join(" → ");
    let diff_link = work_status.to.as_ref().map(|f| format!("/diff/{}", f));
    html! {
        li class=(status_class(&work_status.status)) {
            @if let Some(link) = diff_link {
                a href=(link) {(display_name)}
            } @else {
                (display_name)
            }
        }
    }
}

fn render_file_statuses(info: &DashboardInfo) -> Markup {
    html! {
        ul.status-files {
            @for status in &info.fork_info.file_statuses {
                @if let Some(work_status) = &status.work {
                    (render_file_status(status, work_status))
                }
            }
        }
    }
}

type Args = Option<HashMap<String, String>>;

fn hidden_args(args: &Args) -> Option<Markup> {
    args.as_ref().map(|map| {
        html! {
           @for field in map.iter() {
              input type="hidden" name=(field.0) value=(field.1);
           }
        }
    })
}

fn action_btn(method: &str, action: &str, content: &str, args: &Args, disabled: bool) -> Markup {
    html! {
      form method=(method) action=(action) .inline-form {
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
            (action_btn("POST", "/api/load", "Load", &Some(named(name)), info.current_branch == name))
            (action_btn("POST", "/api/delete", "Delete", &Some(named(name)), name == "main" || info.current_branch == name))
        }
    }
}

struct DashboardInfo {
    current_branch: String,
    unsaved_changes: usize,
    commits_ahead: usize,
    commits_behind: usize,
    fork_info: ForkInfo,
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
        html {
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
}

async fn render_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, render_message("Not found", None))
}

fn count_commits_since(_ctx: &Ctx, older: &Commit, newer: &Commit) -> Maybe<usize> {
    let mut count: usize = 0;
    let mut current = Rc::from(newer.clone());
    while current.id() != older.id() {
        let next = current.parents().next();
        match next {
            Some(c) => {
                count += 1;
                current = Rc::from(c);
            }
            None => return fail("Unable to navigate to fork point."),
        }
    }

    Ok(count)
}

fn get_workspace_name(ctx: &Ctx) -> String {
    ctx.repo
        .workdir()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn load_dashboard_info() -> Maybe<DashboardInfo> {
    let mut ctx = init_ctx()?;
    ctx.set_mode(crate::ctx::Mode::Background);

    let repo_head = ctx.repo.head()?;

    let head_name_str = repo_head.name().unwrap();

    let head_name = head_name_str[head_name_str.rfind("/").map_or(0, |e| e + 1)..].to_owned();

    let base = "main";

    let base_commit = ctx
        .repo
        .find_branch(base, git2::BranchType::Local)?
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

    let mut unsaved_diff = ctx
        .repo
        .diff_tree_to_workdir(Some(&head_commit.tree()?), Some(&mut good_diff_options()))?;

    collapse_renames(&mut unsaved_diff)?;

    Ok(DashboardInfo {
        commits_ahead: head_past_fork,
        commits_behind: base_past_fork,
        current_branch: head_name.to_owned(),
        unsaved_changes: unsaved_diff.deltas().count(),
        branches,
        fork_info: resolve_fork_info(&ctx, None)?,
        workspace: get_workspace_name(&ctx),
    })
}

fn render_dashboard(info: &DashboardInfo) -> Markup {
    html! {
        (DOCTYPE)
        html {
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

                        form method="POST" action="/api/save" {
                            div.spaced-across.end {
                                label {
                                    "Save message"
                                    br;
                                    input .in name="message" placeholder="(optional)" disabled[info.unsaved_changes == 0];
                                }
                                (btn("submit", "Save", info.unsaved_changes == 0))
                            }
                        }

                        (render_file_statuses(info))
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
                                (branch_entry(info, b))
                            }
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
        .map_err(map_error_to_response)
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

            form.spaced-down method="POST" action="/api/sync"  {
                @for conflict in conflicts {
                    @match conflict {
                      Conflict::MainDeletion(path) => {
                        fieldset {
                            (path) " has changes, but was deleted on main." br;
                            (radio("Keep", path, "incoming", true))
                            (radio("Delete", path, "base", false))
                        }
                      },
                      Conflict::BranchDeletion(path) => {
                        fieldset {
                            (path)
                             " was deleted on your branch, but was modified on main."
                             br;

                            (radio("Keep", path, "base", true))
                            (radio("Delete", path, "incoming", false))
                        }
                        },
                        Conflict::Merge(info) => {
                            fieldset {
                                (info.path) " has conflicts." br;
                                (radio("Keep your version", &info.path, "incoming", true))
                                (radio("Reset to main version", &info.path, "base", false))
                                (radio("Resolve conflicts later", &info.path, "later", false))

                                details {
                                    summary {"Your content"}
                                    pre {
                                        code {
                                            (info.branch_content)
                                        }
                                    }
                                }
                                details {
                                    summary {"Main content"}
                                    pre {
                                        code {
                                            (info.main_content)
                                        }
                                    }
                                }
                                details {
                                    summary {"With conflicts"}
                                    pre {
                                        code {
                                            (info.merge_content)
                                        }
                                    }
                                }
                            }
                        }
                        Conflict::OpaqueMerge(path) => {
                            fieldset {
                                (path) "has conflicts." br;
                                label {
                                    "Your version"
                                    input type="radio" name=(path) value="incoming" checked;
                                }
                                label {
                                    "Their version"
                                    input type="radio" name=(path) value="base";
                                }
                            }
                        }

                      }
                }

                (btn("submit", "Submit", false))
            }

            a href="/" {"Cancel"}
        }
    }
}

async fn sync() -> impl IntoResponse {
    render_sync(&vec![])
}

fn render_diff(file_path: String) -> Maybe<Option<Markup>> {
    let mut ctx = init_ctx()?;
    ctx.set_mode(crate::ctx::Mode::Background);

    let head_commit = ctx.repo.head()?.peel_to_commit()?;

    let mut unsaved_diff = ctx
        .repo
        .diff_tree_to_workdir(Some(&head_commit.tree()?), Some(&mut good_diff_options()))?;

    collapse_renames(&mut unsaved_diff)?;

    let target_path = PathBuf::from(&file_path);

    let change_index = unsaved_diff.deltas().enumerate().find_map(|(i, e)| {
        if e.new_file().path() == Some(&target_path) {
            Some(i)
        } else {
            None
        }
    });

    match change_index {
        None => Ok(None),
        Some(change_index) => {
            let mut patch = Patch::from_diff(&unsaved_diff, change_index)?
                .expect("Change index should be valid");

            let mut lines = Vec::<(&'static str, String, String)>::new();

            patch.print(&mut |_: DiffDelta, __: Option<DiffHunk>, line: DiffLine| {
                let origin = line.origin();
                let (main_line, trailing_whitespace_line) = split_diff_line(&line);

                lines.push((
                    match origin {
                        '+' => "new",
                        '-' => "deleted",
                        _ => "",
                    },
                    main_line,
                    trailing_whitespace_line,
                ));

                true
            })?;
            Ok(Some(html! {
                (DOCTYPE)
                html {
                    head {
                        title {
                            (&file_path) " | " (get_workspace_name(&ctx)) " | itch ui"
                        }
                        (common_head_contents())
                    }
                    body.spaced-down {
                        h1 {
                            (&file_path)
                        }
                        a href="/" {"Back"}

                        pre { code { @for line in lines {
                            div class={"diff-line "(line.0)} {
                                (line.1)span.trailing_whitespace{(line.2)}
                            }
                        }}}
                    }
                }
            }))
        }
    }
}

async fn diff(Path(file_path): Path<String>) -> impl IntoResponse {
    match render_diff(file_path) {
        Ok(e) => match e {
            Some(e) => e.into_response(),
            None => (StatusCode::NOT_FOUND, render_message("Not found", None)).into_response(),
        },
        Err(e) => map_error_to_response(e).into_response(),
    }
}

fn with_ctx<R, T>(callback: T) -> Maybe<R>
where
    T: FnOnce(&Ctx) -> Maybe<R>,
{
    let mut ctx = init_ctx()?;
    ctx.set_mode(crate::ctx::Mode::Background);
    callback(&ctx)
}

fn api_handler<R, T>(callback: T) -> impl IntoResponse
where
    T: FnOnce(&Ctx) -> Maybe<R>,
{
    map_result_to_response(with_ctx(callback))
}

fn map_error_to_response(err: Fail) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        render_message("Error", Some(err.message())),
    )
}

fn map_result_to_response<T>(res: Maybe<T>) -> impl IntoResponse {
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

type SyncForm = HashMap<String, String>;

fn convert_sync_form(body: &SyncForm) -> Maybe<ResolutionMap> {
    let mut resolutions: ResolutionMap = HashMap::new();

    for (key, value) in body.iter() {
        let value = if value == "incoming" {
            ResolutionChoice::Incoming
        } else if value == "base" {
            ResolutionChoice::Base
        } else if value == "later" {
            ResolutionChoice::Later
        } else if let Some(("manual", value)) = value.split_once(":") {
            ResolutionChoice::Manual(value.into())
        } else {
            return fail("Unexpected selection");
        };
        resolutions.insert(key.clone(), value);
    }

    Ok(resolutions)
}

async fn handle_sync(Form(body): Form<SyncForm>) -> impl IntoResponse {
    let sync_result = with_ctx(|ctx| {
        let args = convert_sync_form(&body)?;
        save_temp(ctx, "Save before sync".to_string())?;
        let details = try_sync_branch(ctx, &get_current_branch(ctx)?, Some(&args))?;
        pop_and_reset(ctx)?;
        Ok(details)
    });
    match sync_result {
        Ok(details) => {
            if let SyncDetails::Conflicted(d) = details {
                render_sync(&d).into_response()
            } else {
                Redirect::to("/").into_response()
            }
        }
        Err(e) => map_error_to_response(e).into_response(),
    }
}

async fn handle_new(Form(body): Form<NewArgs>) -> impl IntoResponse {
    api_handler(move |ctx| new_command(ctx, &body))
}

async fn handle_load(Form(body): Form<LoadArgs>) -> impl IntoResponse {
    api_handler(move |ctx| load_command(ctx, &body))
}

#[derive(Deserialize, Debug)]
struct DeleteForm {
    name: String,
}

async fn handle_delete(Form(body): Form<DeleteForm>) -> impl IntoResponse {
    api_handler(|ctx| {
        delete_command(
            ctx,
            &DeleteArgs {
                names: vec![body.name],
            },
        )
    })
}

async fn handle_prune() -> impl IntoResponse {
    api_handler(prune_command)
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

pub async fn ui_command(ctx: &Ctx) -> Attempt {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);

    let state = CsrfState {
        token: base64::engine::general_purpose::STANDARD_NO_PAD.encode(key),
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
        .route("/diff/*file_path", get(diff))
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
            if let Ok(l) = axum::Server::try_bind(&addr) {
                return l;
            }
        }
        panic!("Unable to find unused port");
    })();

    let server = builder.serve(app.into_make_service());

    let address = format!("http://localhost:{}", server.local_addr().port());

    if ctx.can_prompt() {
        println!("Listening on {address}");
    }

    open::that(address).unwrap();

    server.await.unwrap();

    Ok(())
}
