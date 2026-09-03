# Contributing to Caby

## Commit messages: Conventional Commits

Caby's releases are automated with [release-please](https://github.com/googleapis/release-please),
which derives the next version and the changelog from commit messages. Commits (and PR titles, since
we squash-merge — the PR title becomes the commit subject) must follow
[Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(optional scope): <description>
```

Common types:

| Type                                                        | Use for                   | Release effect                     |
| ----------------------------------------------------------- | ------------------------- | ---------------------------------- |
| `feat`                                                      | a new feature             | minor bump                         |
| `fix`                                                       | a bug fix                 | patch bump                         |
| `perf`                                                      | a performance improvement | patch bump                         |
| `docs`, `refactor`, `test`, `build`, `ci`, `style`, `chore` | everything else           | no release (hidden from changelog) |

### Overriding the version

To override the version of the next release, add a `Release-As:` footer to any commit (an empty
commit works):

```sh
git commit --allow-empty -m "chore: release 0.1.4" -m "Release-As: 0.1.4"
```

## Releasing

release-please keeps a standing **Release PR** on `main` that bumps every version file
(`caby-service/Cargo.toml`, `caby-web/package.json`, `docker/compose.yaml`) and `CHANGELOG.md` to
one shared version. Merging that PR tags `vX.Y.Z`, creates the GitHub Release, and triggers the
image build. Caby's backend and frontend are always released together as the same version.
