# itch

## Goals

1. Compatible with git + LFS
2. Fast
3. Easy to use

## Command Breakdown

| command | branches | current | files | description
| :- | :- | :- | :- | :- |
| init   | | | | Start a new repo
| new    | ✍️ | ✍️ | ✍️ | Start a new branch
| load   | ✍️ | ✍️ | ✍️ | Switch to an existing branch
| delete | ✍️ |   |   | Delete a branch
| save   | ✍️ | 📍 |   | Checkpoint the current work
| merge  | ✍️ | 📍 | ✍️ | Integrate the current changes into the main branch
| sync   | ✍️ | 📍 | ✍️ | Pull new changes from the main branch into the current branch
| list   | 👀 | 👀 | |  List branches
| log    | 👀 | 👀 | | Show the checkpoints along a branch
| squash | ✍️ | 📍 | | Combine all the save commits into one commit
| diff | 👀 | | | Compare branches and historical changes
| cleanup | ✍️ | | | Delete branches with no changes
| undo | | | ✍️ | Undo a change

## TODO

### Implement features

- [ ] Add log
- [ ] Add merge
- [ ] Add rename
- [ ] Add copy
- [ ] Add squash
- [ ] Add cleanup
- [ ] Add undo

### Tweaks

- [ ] Make printing from main look more right
- [ ] Make the index reset at times that make sense
- [ ] Make diff take more options
- [ ] Make sync handle conflicts
- [ ] Trim whitespace when truncating commit messages
- [ ] Add changed files to status


### Mapping from git

- **git clone** -> `itch init https://something.git`
-  **git init** -> `itch init`
-  **git add** -> Not needed - there is no staging area
-  **git commit** -> `itch save`
-  **git status** -> todo
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
