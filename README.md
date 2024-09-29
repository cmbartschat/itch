# itch

Itch is a source control tool that lets you get work done faster. It is:

- compatible with git
- usable on the command line or as a graphical interface
- easy to use either way

## Installation

TODO

## Getting started

- `itch init` start a new repository in the current folder

## Using the GUI

`itch ui` - Open the graphical user interface. While the command is running in the terminal, the interface will remain available.

## Working with branches

In itch, like git, branches are used to keep track of what you're working on. If you're stuck on one thing, you can start a fresh branch and get something done in that branch while the other branch is on the backburner.

`itch list` - Display branches

`itch new` - Create a new branch with a placeholder name

`itch new mybranch` - Create a new branch called "mybranch"

`itch load mybranch` - Switch to the specified branch.

If there are unsaved changes in the current branch, they will be saved and brought back when you return.

`itch prune` - Delete branches that have no pending changes

`itch split` - Duplicate the current branch with a placeholder name

`itch split mybranch` - Duplicate the current branch as "mybranch"

`itch delete mybranch` - Delete a branch

## Making changes

After you make changes to your files, you'll want to save them. Initially, saves are only visible to the branch they were saved to. To finalize your saves, you'll use `merge` to merge them into the main branch.

`itch save` - Save with placeholder message

`itch save this is the message` - Save changes as "this is the message"

`itch squash` - Squash all unmerged saves into one, preserving the most recent save message

`itch unsave` - Clear out all unmerged saves without reverting changes

`itch merge` - Merge saved changes into the main branch

## Pulling in changes
    
`itch sync` - Bring the latest changes from main into this branch

If there are conflicts, you may be asked to keep, reset, or edit the conflicted file. If you "keep", you will keep your branch's version of the file, ignoring any changes made on the main branch. If you "reset", you will undo all your unmerged changes. If you "edit", you'll get a popup window allowing you to select which portions of the files you want to keep.

## Inspecting

`itch log` - Show history of the current branch

`itch status` - Show the status of the current branch

This will show where your branch is compared to main, what files have changed, and what still needs to be saved.

`itch diff` - Show unmerged changes

`itch diff of mybranch` - Show unmerged changes of a different branch

`itch diff from mybranch` - Compare current changes since some other point in time

`itch diff from mybranch to otherbranch` - Compare two points in time

## Synchronizing with a remote

- `itch connect <url>` - connect a repo to a remote git service
- `itch disconnect` - disconnect from the current remote

If you have remote, it will be used by itch to backup pending changes, and synchronize shared changes. Any time you save, the branch will be saved to the remote as `<username>-<branchname>`. Use the `ITCH_REMOTE_PREFIX` if you want a different prefix before the branch name.

itch will also push and pull changes from the remote main branch, allowing you to collaborate with others in the repository.

## Recovering from bad states

Itch is made to reduce the chances of mistakes during normal operation. If something does happen, it might be time to drop down into git. git has powerful tools for manipulating state and using the reflog to recover "lost" work is usually possible.
