use std::path::Path;

use git2::{build::TreeUpdateBuilder, Commit, FileMode, ResetType, Tree, TreeEntry};

use crate::{
    cli::UnsaveArgs,
    ctx::Ctx,
    error::{fail, Attempt},
};

const GIT_FILEMODE_UNREADABLE: i32 = 0o000000;
const GIT_FILEMODE_TREE: i32 = 0o040000;
const GIT_FILEMODE_BLOB: i32 = 0o100644;
const GIT_FILEMODE_BLOB_GROUP_WRITABLE: i32 = 0o100664;
const GIT_FILEMODE_BLOB_EXECUTABLE: i32 = 0o100755;
const GIT_FILEMODE_LINK: i32 = 0o120000;
const GIT_FILEMODE_COMMIT: i32 = 0o160000;

fn get_entry_mode(entry: &TreeEntry) -> FileMode {
    let mode = entry.filemode();
    if mode == GIT_FILEMODE_UNREADABLE {
        return FileMode::Unreadable;
    }
    if mode == GIT_FILEMODE_TREE {
        return FileMode::Tree;
    }
    if mode == GIT_FILEMODE_BLOB {
        return FileMode::Blob;
    }
    if mode == GIT_FILEMODE_BLOB_GROUP_WRITABLE {
        return FileMode::BlobGroupWritable;
    }
    if mode == GIT_FILEMODE_BLOB_EXECUTABLE {
        return FileMode::BlobExecutable;
    }
    if mode == GIT_FILEMODE_LINK {
        return FileMode::Link;
    }
    if mode == GIT_FILEMODE_COMMIT {
        return FileMode::Commit;
    }
    FileMode::Unreadable
}

fn unsave_files(
    ctx: &Ctx,
    files: &[String],
    head_commit: &Commit,
    prev_commit: &Commit,
) -> Attempt {
    let current_tree = head_commit.tree()?;
    let prev_tree = prev_commit.tree()?;

    let mut new_tree_builder = TreeUpdateBuilder::new();

    for file_path in files.iter().map(Path::new) {
        match prev_tree.get_path(file_path) {
            Ok(entry) => {
                new_tree_builder.upsert(file_path, entry.id(), get_entry_mode(&entry));
            }
            Err(e) => match e.code() {
                git2::ErrorCode::NotFound => {
                    let should_delete = match current_tree.get_path(file_path) {
                        Ok(_) => true,
                        Err(e) => {
                            if e.code() == git2::ErrorCode::NotFound {
                                false
                            } else {
                                return Err(e);
                            }
                        }
                    };
                    if should_delete {
                        new_tree_builder.remove(file_path);
                    }
                }
                _ => return Err(e),
            },
        }
    }

    let new_tree: Tree = ctx
        .repo
        .find_tree(new_tree_builder.create_updated(&ctx.repo, &current_tree)?)?;
    let parents: Vec<Commit> = head_commit.parents().collect();
    let parent_refs: Vec<&Commit> = parents.iter().collect();

    let committed = ctx.repo.commit(
        None,
        &head_commit.author(),
        &head_commit.committer(),
        head_commit.message().unwrap_or(""),
        &new_tree,
        &parent_refs,
    )?;

    ctx.repo.reset(
        &ctx.repo.find_object(committed, None)?,
        ResetType::Mixed,
        None,
    )?;

    Ok(())
}

fn unsave_single(ctx: &Ctx, prev_commit: Commit) -> Attempt {
    ctx.repo
        .reset(&prev_commit.into_object(), ResetType::Mixed, None)?;

    Ok(())
}

pub fn unsave_command(ctx: &Ctx, args: &UnsaveArgs) -> Attempt {
    let head_commit = ctx.repo.head()?.peel_to_commit()?;
    let base_commit = ctx
        .repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference()
        .peel_to_commit()?;

    let fork_commit = ctx
        .repo
        .find_commit(ctx.repo.merge_base(base_commit.id(), head_commit.id())?)?;

    if head_commit.id() == fork_commit.id() {
        return fail("Cannot unsave past the fork point");
    }

    let mut parent_commits = head_commit.parents();
    match (parent_commits.next(), parent_commits.next()) {
        (None, _) => fail("Latest commit has no parent."),
        (Some(prev_commit), None) => {
            if !args.args.is_empty() {
                unsave_files(ctx, &args.args, &head_commit, &prev_commit)
            } else {
                unsave_single(ctx, prev_commit)
            }
        }
        (Some(_), Some(_)) => fail("Expected single parent commit."),
    }
}
