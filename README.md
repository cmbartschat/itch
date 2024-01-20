# itch

## Goals

1. Compatible with git + LFS
2. Fast
3. Easy to use

## Command Breakdown

| command | branches | current | files | description
| :- | :- | :- | :- | :- |
| init    |   |   |   | Start a new repo
| new     | ✍️ | ✍️ | ✍️ | Start a new branch
| load    | ✍️ | ✍️ | ✍️ | Switch to an existing branch
| delete  | ✍️ |   |   | Delete a branch
| save    | ✍️ | 📍 |   | Checkpoint the current work
| merge   | ✍️ | 📍 | ✍️ | Integrate the current changes into the main branch
| sync    | ✍️ | 📍 | ✍️ | Pull new changes from the main branch into the current branch
| list    | 👀 | 👀 |   |  List branches
| log     | 👀 | 👀 |   | Show the checkpoints along a branch
| squash  | ✍️ | 📍 |   | Combine all the save commits into one commit
| diff    | 👀 |   |   | Compare branches and historical changes
| prune   | ✍️ |   |   | Delete branches with no changes
| undo    |   |   | ✍️ | Undo a change
| ui      | ✍️ | ✍️ | ✍️ | Interactive UI for making commands

## Natural language commands

```
itch new branch from otherbranch
itch squash to 7c5ac8
itch squash as commit message
itch diff between main and branch
itch diff from 37f937
itch sync to c001e5
```

## TODO

### Implement features

- [x] Add log
- [x] Add merge
- [x] Add squash
- [x] Add clean
- [ ] Add init
- [ ] Add rename
- [ ] Add revert
- [ ] Add copy
- [ ] Add undo
- [ ] Add remote pull/push
- [ ] Add unsave
- [ ] Add fork

### Tweaks

- [x] Make printing from main look more right
- [x] Make diff take more options
- [x] Make sync handle conflicts
- [x] Don't show main twice in status
- [x] Fix diff not showing new files
- [x] Make sync actually work
- [x] Remove extraneous print/debug statements
- [x] Line up diff lines with context lines
- [x] Trim whitespace when truncating commit messages
- [x] Add changed files to status
- [x] Make the index reset at times that make sense
    - [x] Save
    - [x] Load, if the previous save was a "switch"
    - [x] Sync
- [ ] Make long output print with `less`
- [ ] More advanced merge resolution during sync
- [x] Fix diff not diffing from fork point
- [x] Fix status showing renamed files as modified
- [x] Prevent CSRF for ui
- [x] Make ui not use unwrap
- [ ] Add status/diff to ui
- [ ] Make diff look better when piped to code
- [ ] Show trailing whitespace in diff
- [x] Diff unsaved
- [x] Handle plural/disabled states in ui buttons
- [x] Hash something to generate port for ui
- [x] Compare trees for pruning
- [x] Refresh ui on focus
- [x] Favicon for ui
- [ ] Open ui process as daemon
- [ ] Conflict resolution in ui
- [ ] Prevent autosave on main
- [x] Fix sync throwing away unsaved changes
- [ ] Fix the .into() on windows/get compiler working
- [ ] Make sure LFS works
- [ ] Detect and propagate info around interactivity/verbose/escape characters
- [ ] Check if commands can be made more atomic
- [ ] Specify squash message?

### Mapping from git

-  **git clone** -> `itch init https://something.git`
-  **git init** -> `itch init`
-  **git add** -> Not needed - there is no staging area
-  **git commit** -> `itch save`
-  **git status** -> `itch status`
-  **git pull** -> `itch sync`
-  **git push** -> `itch merge` ?? 
-  **git branch** `itch new` / `itch delete` / `itch list`
-  **git checkout** `itch load`
-  **git merge** `itch merge`
-  **git log** `itch log`
-  **git diff** `itch diff`
-  **git remote** -> todo
-  **git fetch** -> todo
-  **git stash** -> `itch save`

## Internals

- Git user metadata - uses as normal, supports code signing
- Base branch - defaults to `main`, but can also be set per branch or specified on the cli
- Remote prefix - defaults to `<user>-`, but can be set per branch

### Possible state of the world

```
{
  files: Record<string, string>
  branches: Record<string, string>
  current_branch: string
}
```
