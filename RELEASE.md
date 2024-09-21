# Release process


## Minor release

- Create branch `release/0.x.y` and related PR, use "release" label
- Update `CHANGELOG.md`, use "Unreleased" heading unless final release, copy-paste-edit from:
  ```
  python scripts/changelog.py --commit-range v0.${X-1}.0..HEAD
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
- Bump version to `0.X.0`, tag `v0.X.0` and push tag
- Publish to <https://crates.io>
- Check GH Release
- Squash-merge the release PR, undelete the branch
- Bump version to next alpha on `master`


## Patch release

(WIP)

- Cherry-pick commits on the release branch
- Update CHANGELOG.md
- (optional) RC
- Bump version to `0.${X}.${Y}`, tag `v0.X.Y` and push tag
- Check GH Release
