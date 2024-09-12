# TODO

## Features

- [x] Add log
- [x] Add merge
- [x] Add squash
- [x] Add clean
- [x] Add remote pull/push
- [x] Add unsave
- [x] Add split
- [ ] Add init
- [ ] Add rename
- [ ] Add revert
- [ ] Add undo

## Tweaks

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
- [x] Make long output print with `less`
- [x] More advanced merge resolution during sync
- [x] Fix diff not diffing from fork point
- [x] Fix status showing renamed files as modified
- [x] Prevent CSRF for ui
- [x] Make ui not use unwrap
- [x] Make diff look better when piped to code
- [x] Diff unsaved
- [x] Handle plural/disabled states in ui buttons
- [x] Hash something to generate port for ui
- [x] Compare trees for pruning
- [x] Refresh ui on focus
- [x] Favicon for ui
- [x] Fix sync throwing away unsaved changes
- [x] Detect and propagate info around interactivity/verbose/escape characters
- [x] Specify remote prefix
- [x] Include files in untracked folders in status
- [x] Make ui refresh faster
- [x] Make sure to use eprintln
- [ ] Add status/diff to ui
- [ ] Check if commands can be made more atomic
- [ ] Specify squash message?
- [ ] Show trailing whitespace in diff
- [ ] Unsave specific files
- [ ] Delete branch when deleting client
- [ ] Prevent saving "prefix-main"
- [ ] Fix the .into() on windows/get compiler working
- [ ] Make sure LFS works
- [ ] Open ui process as daemon
- [ ] Conflict resolution in ui
- [ ] Prevent autosave on main
