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
    branch::find_branch, checkout::checkout, cli::LoadArgs, ctx::Ctx, error::Attempt,
    reference::set_head_direct, reset::pop_and_reset, save::save_temp,
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

pub fn load_command(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    save_temp(ctx, format!("Save before switching to {}", args.name))?;
    let mut target_ref = find_branch(ctx, &args.name)?;
    checkout(ctx, target_ref.peel_to_commit()?)?;
    set_head_direct(ctx, target_ref.name().to_owned())?;
    pop_and_reset(ctx)?;
    Ok(())
}
