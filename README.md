# itch

## Goals

1. Compatible with git + LFS
2. Fast
3. Easy to use


## Things you need to be able to do

### Start a new repo

```
itch init
```

### Start a new branch

```
itch new
```

### Save the branch

```
itch save
```

### Merge the branch to main

```
itch merge
```

### Delete a branch

```
itch delete <name>
```

### Pull in new changes

```
itch sync
```

### List all branches

```
itch list
```

### Reset specific files

```
itch reset my-file.txt
```

## Internals

- Git user metadata - uses as normal, supports code signing
- Base branch - defaults to `main`, but can also be set per branch or specified on the cli
- Remote prefix - defaults to `user-`, but can be set per branch
