# Releasing

## Finalize

1. `open Cargo.toml`
2. Update `version`
3. Merge to main

## Create a release

1. Go to https://github.com/cmbartschat/itch/releases/new
2. Create a new tag of the format `vX.X.X`
3. Submit

## Update the brew formula

1. Switch to a clone of [homebrew-itch](https://github.com/cmbartschat/homebrew-itch/)
2. `open Formula/itch.rb` 
3. Grab in the new release .zip URL and paste it into `url`
4. Run `curl -L <url> | openssl dgst -sha256` and paste it into `sha256`
5. Bump the version number on the `assert_match` statement
6. Merge to main
7. Test by running `brew upgrade cmbartschat/itch/itch`
