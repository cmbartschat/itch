use std::{
    ops::DerefMut,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, atomic::AtomicBool},
};

use gix::{
    bstr::{self, BStr},
    fs::FileSnapshot,
    objs::Find,
    progress::Discard,
    refs::{
        FullName, FullNameRef,
        transaction::{Change, LogChange, PreviousValue},
    },
    worktree::stack::state::attributes::Source,
};

use crate::{
    branch::find_branch, cli::LoadArgs, ctx::Ctx, error::Attempt, reset::pop_and_reset,
    save::save_temp,
};

struct Wildcard {}

impl gix::objs::Find for Wildcard {
    fn try_find<'a>(
        &self,
        id: &gix::hash::oid,
        buffer: &'a mut Vec<u8>,
    ) -> Result<Option<gix::objs::Data<'a>>, gix::objs::find::Error> {
        todo!()
    }
}

fn load_command_inner(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    let mut target_ref = find_branch(ctx, &args.name)?;

    let mut options = ctx.repo.checkout_options(Source::IdMappingThenWorktree)?;
    options.overwrite_existing = true;

    // let owned_index: gix::index::File = owned_snapshot;
    let mut index_state: gix::index::State = ctx
        .repo
        .index_from_tree(&target_ref.peel_to_tree()?.id)?
        .into_parts()
        .0;
    let dir: PathBuf = ctx.repo.path().to_path_buf();
    let objects = ctx.repo.clone().objects.into_arc()?;
    let files = Discard; // &dyn gix::features::progress::Count;
    let bytes = Discard; // &dyn gix::features::progress::Count;
    let should_interrupt: AtomicBool = Default::default();

    let outcome = gix::worktree::state::checkout(
        &mut index_state,
        dir,
        objects,
        &files,
        &bytes,
        &should_interrupt,
        options,
    )?;

    eprintln!("Checkout finished: {:?}", outcome);

    let message = format!("Loading {}", args.name);

    ctx.repo.edit_reference(gix::refs::transaction::RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: gix::refs::transaction::RefLog::AndReference,
                force_create_reflog: false,
                message: message.into(),
            },
            expected: PreviousValue::Any,
            new: gix::refs::Target::Symbolic(target_ref.name().to_owned()),
        },
        name: ctx.repo.head()?.name().into(),
        deref: false,
    })?;

    // pop_and_reset(ctx)
    Ok(())
}

pub fn load_command(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    save_temp(ctx, format!("Save before switching to {}", args.name))?;

    load_command_inner(ctx, args)
}
