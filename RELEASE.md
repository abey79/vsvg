# Release process

- Create branch `release/0.x.y` and related PR, use "release" label
- Update `CHANGELOG.md`, use "Unreleased" heading unless final release, copy-paste-edit from:
  ```
  python scripts/changelog --commit-range v0.X-1.Y-1..HEAD
  ```
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
- Finalise `CHANGELOG.md` with proper heading
- Tag `v0.X.Y` and push tag
- Check GH Release
- Bump version to next alpha
- Squash-merge the release PR, undelete the branch
