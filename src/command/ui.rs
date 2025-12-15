use base64::Engine;
use fork::{Fork, close_fd, fork};
use rand::RngCore;
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use git2::{BranchType, Delta, DiffDelta, DiffHunk, DiffLine, Patch};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::{
    branch::get_current_branch,
    cli::{DeleteArgs, LoadArgs, NewArgs, SaveArgs, SquashArgs},
    command::new::new_command,
    commit::count_commits_since,
    ctx::{Ctx, init_ctx},
    diff::{collapse_renames, good_diff_options, split_diff_line},
    error::{Attempt, Fail, Maybe, fail, inner_fail},
    reset::pop_and_reset,
    save::save_temp,
    sync::{Conflict, ResolutionChoice, ResolutionMap, SyncDetails},
};

use axum::{
    Form, Router,
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use maud::{DOCTYPE, Markup, PreEscaped, html};

use super::{
    delete::delete_command,
    load::load_command,
    merge::merge_command,
    prune::prune_command,
    save::save_command,
    squash::squash_command,
    status::{FileStatus, ForkInfo, SegmentedStatus, resolve_fork_info},
    sync::try_sync_branch,
};

#[derive(Clone)]
struct CsrfState {
    token: String,
    exit_signal: tokio::sync::mpsc::Sender<()>,
}

#[derive(Debug)]
struct UiHost {
    port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct UiInfo {
    directory: String,
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

fn status_class(delta: Delta) -> &'static str {
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
    let diff_link = work_status.to.as_ref().map(|f| format!("/diff/{f}"));
    html! {
        li class=(status_class(work_status.status)) {
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
           @for field in map {
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

fn branch_entry(info: &DashboardInfo, branch: &BranchInfo) -> Markup {
    let name: &str = &branch.name;
    html! {
        li.spaced-across {
            span.grow .selected[info.current_branch == name] { (name) }
            @if branch.commits_behind > 0 {
                (action_btn("POST", "/api/sync", "Sync", &Some(named(name)), false))
            }
            (action_btn("POST", "/api/load", "Load", &Some(named(name)), info.current_branch == name))
            (action_btn("POST", "/api/delete", "Delete", &Some(named(name)), name == "main"))
        }
    }
}

struct BranchInfo {
    name: String,
    commits_behind: usize,
}

struct DashboardInfo {
    current_branch: String,
    unsaved_changes: usize,
    commits_ahead: usize,
    commits_behind: usize,
    fork_info: ForkInfo,
    branches: Vec<BranchInfo>,
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

fn render_message(title: &str, text: Option<&str>) -> Markup {
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

    let head_name = head_name_str[head_name_str.rfind('/').map_or(0, |e| e + 1)..].to_owned();

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

    let mut branches: Vec<BranchInfo> = vec![];

    for branch in ctx.repo.branches(Some(git2::BranchType::Local))? {
        let (branch, _type) = branch?;
        let branch_name = branch.name()?.unwrap().to_string();
        let head_commit = branch.into_reference().peel_to_commit()?;
        let fork_point = ctx
            .repo
            .find_commit(ctx.repo.merge_base(base_commit.id(), head_commit.id())?)?;

        let base_past_fork = count_commits_since(&ctx, &fork_point, &base_commit)?;

        branches.push(BranchInfo {
            name: branch_name,
            commits_behind: base_past_fork,
        });
    }

    let mut unsaved_diff = ctx
        .repo
        .diff_tree_to_workdir(Some(&head_commit.tree()?), Some(&mut good_diff_options()))?;

    collapse_renames(&mut unsaved_diff)?;

    Ok(DashboardInfo {
        commits_ahead: head_past_fork,
        commits_behind: base_past_fork,
        current_branch: head_name.clone(),
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
                    div.spaced-across.grow {
                        h1 { (info.workspace) }
                        a href="/" { "Refresh" }
                    }
                    div.right {
                       (action_btn("POST", "/api/quit", "Quit", &None, false))
                    }
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
                            (action_btn("POST", "/api/sync", "Sync", &None, false))
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
                            (action_btn("POST", "/api/sync_all", "Sync all", &None, false))
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

fn render_sync(conflicts: &Vec<Conflict>, branch_name: Option<&str>) -> Markup {
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
                (branch_name.map(|b| hidden_args(&Some(named(b)))).unwrap_or_default().unwrap_or_default())

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
    render_sync(&vec![], None)
}

fn render_diff(file_path: &str) -> Maybe<Option<Markup>> {
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
    match render_diff(&file_path) {
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
    let res = (
        StatusCode::INTERNAL_SERVER_ERROR,
        render_message("Error", Some(err.message())),
    );
    std::mem::drop(err);
    res
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
    api_handler(|ctx| squash_command(ctx, &SquashArgs { message: vec![] }))
}

async fn handle_quit(State(state): State<CsrfState>) -> impl IntoResponse {
    match state.exit_signal.send(()).await {
        Ok(()) => (
            StatusCode::OK,
            render_message(
                "Exited",
                Some("The UI server has exited, you can close the tab."),
            ),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            render_message("Error", Some("Failed to exit UI.")),
        ),
    }
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

    for (key, value) in body {
        let value = if value == "incoming" {
            ResolutionChoice::Incoming
        } else if value == "base" {
            ResolutionChoice::Base
        } else if value == "later" {
            ResolutionChoice::Later
        } else if let Some(("manual", value)) = value.split_once(':') {
            ResolutionChoice::Manual(value.into())
        } else {
            return fail!("Unexpected selection");
        };
        resolutions.insert(key.clone(), value);
    }

    Ok(resolutions)
}

async fn handle_sync(Form(mut body): Form<SyncForm>) -> impl IntoResponse {
    let sync_result = with_ctx(|ctx| {
        let name = body.remove("name");
        let args = convert_sync_form(&body)?;
        let current_branch = get_current_branch(ctx)?;
        let target_branch = name.unwrap_or(current_branch);
        save_temp(ctx, "Save before sync".to_string())?;
        let details = try_sync_branch(ctx, &target_branch, Some(&args))?;
        pop_and_reset(ctx)?;
        Ok((target_branch, details))
    });
    match sync_result {
        Ok((name, details)) => {
            if let SyncDetails::Conflicted(d) = details {
                render_sync(&d, Some(&name)).into_response()
            } else {
                Redirect::to("/").into_response()
            }
        }
        Err(e) => map_error_to_response(e).into_response(),
    }
}

async fn handle_sync_all() -> impl IntoResponse {
    let sync_result = with_ctx(|ctx| {
        save_temp(ctx, "Save before sync".to_string())?;

        let branches = ctx.repo.branches(Some(BranchType::Local))?;
        for branch in branches {
            let branch = branch?.0;
            if let Some(branch_name) = branch.name()?
                && branch_name != "main"
            {
                try_sync_branch(ctx, branch_name, None)?;
            }
        }
        pop_and_reset(ctx)?;
        Ok(())
    });
    match sync_result {
        Ok(()) => Redirect::to("/").into_response(),
        Err(e) => map_error_to_response(e).into_response(),
    }
}

async fn handle_new(Form(body): Form<NewArgs>) -> impl IntoResponse {
    api_handler(move |ctx| new_command(ctx, &body))
}

async fn handle_load(Form(body): Form<LoadArgs>) -> impl IntoResponse {
    api_handler(move |ctx| load_command(ctx, &body))
}

async fn handle_info() -> impl IntoResponse {
    with_ctx(|ctx| {
        let data = UiInfo {
            directory: ctx.repo.path().to_string_lossy().to_string(),
        };
        let json = serde_json::to_string(&data).map_err(|e| {
            eprintln!("{e:?}");
            inner_fail!("Failed to Serialize")
        })?;
        Ok((StatusCode::OK, json))
    })
    .map_err(map_error_to_response)
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

fn iterate_ports(dir: &str) -> std::ops::Range<u16> {
    let mut hasher = DefaultHasher::new();
    dir.hash(&mut hasher);
    let bytes = hasher.finish().to_be_bytes();
    let offset = 8000 + (u16::from_be_bytes([bytes[0], bytes[1]]) % 1000);
    offset..(offset + 100)
}

async fn locate_background(dir: &str) -> Maybe<Option<UiHost>> {
    for port in iterate_ports(dir) {
        match reqwest::get(format!("http:/0.0.0.0:{port}/_info")).await {
            Ok(res) => {
                if res.status().is_success() {
                    let text = res
                        .text()
                        .await
                        .map_err(|_| inner_fail!("Failed to load"))?;
                    let parsed: UiInfo =
                        serde_json::from_str(&text).map_err(|_| inner_fail!("Parse error"))?;
                    if parsed.directory == dir {
                        return Ok(Some(UiHost { port }));
                    }
                }
            }
            Err(e) => {
                if e.is_connect() {
                    continue;
                }
                return fail!("Unexpected error in request");
            }
        }
    }

    Ok(None)
}

async fn run_ui_server(ctx: &Ctx) -> Attempt {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let state = CsrfState {
        token: base64::engine::general_purpose::STANDARD_NO_PAD.encode(key),
        exit_signal: tx,
    };

    let api_router = Router::new()
        .route("/merge", post(handle_merge))
        .route("/squash", post(handle_squash))
        .route("/sync", post(handle_sync))
        .route("/sync_all", post(handle_sync_all))
        .route("/save", post(handle_save))
        .route("/quit", post(handle_quit))
        .route("/load", post(handle_load))
        .route("/delete", post(handle_delete))
        .route("/prune", post(handle_prune))
        .route("/new", post(handle_new))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            csrf_check,
        ));

    let root_router = Router::new()
        .route("/", get(dashboard))
        .route("/sync", get(sync))
        .route("/diff/*file_path", get(diff))
        .route("/_info", get(handle_info))
        .nest("/api", api_router)
        .with_state(state)
        .fallback(render_404);

    let builder = (|| {
        for port in iterate_ports(&ctx.repo.path().to_string_lossy()) {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
            if let Ok(l) = axum::Server::try_bind(&addr) {
                return l;
            }
        }
        panic!("Unable to find unused port");
    })();

    let server = builder.serve(root_router.into_make_service());

    let address = format!("http://localhost:{}", server.local_addr().port());

    let shutdown = server.with_graceful_shutdown(async {
        rx.recv().await;
    });

    if ctx.can_prompt() {
        println!("Listening on {address}");
    }

    open::that(address).unwrap();

    close_fd().map_err(|_| inner_fail!("Failed to fork"))?;

    shutdown.await.map_err(|_| inner_fail!("Exited with error"))
}

pub fn ui_command(ctx: &Ctx) -> Attempt {
    match fork() {
        Ok(Fork::Child) => {}
        Ok(Fork::Parent(_)) => {
            std::thread::sleep(std::time::Duration::from_millis(500));
            return Ok(());
        }
        Err(_) => return fail!("Failed to start in background"),
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| inner_fail!("Async runtime error"))?;

    runtime.block_on(async {
        match locate_background(ctx.repo.path().to_string_lossy().as_ref()).await {
            Ok(Some(e)) => {
                let address = format!("http://localhost:{}", e.port);
                open::that(&address).unwrap();
                println!("Listening on {address}");
                Ok(())
            }
            Ok(None) => run_ui_server(ctx).await,
            Err(e) => Err(e),
        }
    })
}
