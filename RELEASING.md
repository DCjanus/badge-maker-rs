# Releasing

Releases are published from version tags by GitHub Actions. Pushing a matching
`v<semver>` tag publishes the crate to crates.io and creates a GitHub release.
Because a crates.io publication cannot be overwritten, complete all preparation
and verification before creating the tag.

## 1. Choose the version

Follow Semantic Versioning. Before 1.0, increment the minor version for breaking
changes and the patch version for backward-compatible fixes.

If the release includes a dependency refresh, apply the rolling MSRV policy in
`CONTRIBUTING.md` and update the declared and CI-tested Rust versions together.

## 2. Prepare a release pull request

Start from the latest successful `master` build and create a dedicated release
branch:

```console
git fetch origin
git switch master
git merge --ff-only origin/master
git switch -c chore/release-vX.Y.Z
cargo set-version X.Y.Z
```

Update `CHANGELOG.md`:

- Move the release's user-visible changes under a dated `X.Y.Z` heading.
- Identify breaking changes and required migrations explicitly.
- Update the comparison links at the bottom of the file.

Commit the version and changelog changes and verify the release candidate:

```console
just check
cargo package --locked
```

Open a pull request named `chore(release): prepare vX.Y.Z`, and wait for all
required checks to pass.

## 3. Publish the merged release

After the release pull request is merged and the resulting `master` build is
successful, update the local branch and confirm the version before tagging:

```console
git fetch origin
git switch master
git merge --ff-only origin/master
cargo pkgid
git status --short
```

Create an annotated tag on that exact commit and push only the tag:

```console
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin vX.Y.Z
```

The release workflow then:

1. Runs the normal and MSRV checks.
2. Verifies that the tag is valid SemVer, points to a commit on `master`,
   matches the crate version, and has a dated `CHANGELOG.md` entry.
3. Builds and verifies the package with `cargo package --locked`.
4. Authenticates through crates.io trusted publishing and runs
   `cargo publish --locked`.
5. Creates a GitHub release with generated release notes.

Monitor the workflow through completion, then verify the new version on both
crates.io and the GitHub Releases page. If publication succeeds but a later step
fails, do not reuse or move the version tag; repair the remaining release step
without attempting to republish the same crate version.
