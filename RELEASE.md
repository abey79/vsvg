# Release process

- Create branch `release/0.x.y` and related PR, use "release" label
- Update `CHANGELOG.md`, use "Unreleased" heading unless final release
- Bump version to `-rc.Z`:
  ```
  cargo ws version -a --exact --force "*" --no-git-commit
  ```
- Check, commit, push, and ensure CI ✅
- Publish to <https://crates.io>
  ```
  cargo ws publish --from-git
  ```
- Check docs.rs ✅
  - <https://docs.rs/whiskers/>
  - <https://docs.rs/vsvg/>
  - <https://docs.rs/vsvg-viewer/>
- Tag `v0.X.Y` and push tag
- Check GH Release
