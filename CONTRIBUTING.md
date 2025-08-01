# Contributing to Miden client

#### First off, thanks for taking the time to contribute!

We want to make contributing to this project as easy and transparent as possible, whether it's:

- Reporting a [bug](https://github.com/0xMiden/miden-client/issues/new)
- Taking part in [discussions](https://github.com/0xMiden/miden-client/discussions)
- Submitting a [fix](https://github.com/0xMiden/miden-client/pulls)
- Proposing new [features](https://github.com/0xMiden/miden-client/issues/new)

## Flow
We are using [Github Flow](https://docs.github.com/en/get-started/quickstart/github-flow), so all code changes happen through pull requests from a [forked repo](https://docs.github.com/en/get-started/quickstart/fork-a-repo).

### Branching
- The current active branch is `next`. Every branch with a fix/feature must be forked from `next`.

- The branch name should contain a short issue/feature description separated with hyphens [(kebab-case)](https://en.wikipedia.org/wiki/Letter_case#Kebab_case).

    For example, if the issue title is `Fix functionality X in component Y` then the branch name will be something like: `fix-x-in-y`.

- New branch should be rebased from `next` before submitting a PR in case there have been changes to avoid merge commits.
i.e. this branches state:
  ```
          A---B---C fix-x-in-y
         /
    D---E---F---G next
            |   |
         (F, G) changes happened after `fix-x-in-y` forked
  ```

  should become this after rebase:


  ```
                  A'--B'--C' fix-x-in-y
                 /
    D---E---F---G next
  ```


  More about rebase [here](https://git-scm.com/docs/git-rebase) and [here](https://www.atlassian.com/git/tutorials/rewriting-history/git-rebase#:~:text=What%20is%20git%20rebase%3F,of%20a%20feature%20branching%20workflow.)


### Commit messages
- Commit messages should be written in a short, descriptive manner and be prefixed with tags for the change type and scope (if possible) according to the [semantic commit](https://gist.github.com/joshbuchea/6f47e86d2510bce28f8e7f42ae84c716) scheme.
For example, a new change to the `miden-node-store` crate might have the following message: `feat(miden-node-store): fix block-headers database schema`

- Also squash commits to logically separated, distinguishable stages to keep git log clean:
    ```
    7hgf8978g9... Added A to X \
                                \  (squash)
    gh354354gh... oops, typo --- * ---------> 9fh1f51gh7... feat(X): add A && B
                                /
    85493g2458... Added B to X /


    789fdfffdf... Fixed D in Y \
                                \  (squash)
    787g8fgf78... blah  blah --- * ---------> 4070df6f00... fix(Y): fixed D && C
                                /
    9080gf6567... Fixed C in Y /
    ```

### Code Style and Documentation
- For documentation in the codebase, we follow the [rustdoc](https://doc.rust-lang.org/rust-by-example/meta/doc.html) convention with no more than 100 characters per line.
- We also have technical and user documentation built with [mkdocs](https://github.com/mkdocs/mkdocs). You should update it whenever architectural changes or public interface (cli, client lib, etc.) changes are being made.
- For code sections, we use code separators like the following to a width of 100 characters::
    ```
    // CODE SECTION HEADER
    // ================================================================================
    ```

- [Rustfmt](https://github.com/rust-lang/rustfmt), [Clippy](https://github.com/rust-lang/rust-clippy), [Rustdoc](https://doc.rust-lang.org/rustdoc/index.html), [Typos](https://github.com/crate-ci/typos) and [Taplo](https://github.com/tamasfe/taplo) linting is included in the CI pipeline. 

You can run all checks locally before opening a PR. To simplify running all checks in a reproducible manner we use `make` commands:

```
make lint
```

You can find more information about other `make` commands in the [Makefile](Makefile).

### Versioning
We use [semver](https://semver.org/) naming convention.

## Pre-PR checklist

> [!TIP]
> You can check if you have the tools that we use by running `make check-tools`.

- Repo forked and branch created from `next` according to the naming convention.
- Commit messages and code style follow conventions.
- Tests added for new functionality, with all previously existing tests passing.
  - Read more on how to run integration tests [here](./tests/README.md).
- Documentation/comments updated for all changes according to our documentation convention.
- Lints: Rustfmt, Clippy, Rustdoc, Typos and TOML-formatting linting passing.
  - As mentioned above, `make lint` can be used for checking there.
- If the PR includes changes to the web client, the documentation needs to be up to date. You can automatically generate them by running `make typedoc`.

### Changelog

If a PR introduces anything a downstream user might notice - a new feature, a behaviour change, a bug-fix, a deprecation, or a breaking API change - add a bullet to [CHANGELOG.md](./CHANGELOG.md) under the last existing heading (corresponds to the unreleased version when working on the `next` branch). The convention is to write a past-tense summary (e.g. "* [BREAKING] Renamed foo() to bar() in `Client` {#PR number link}"). Internal refactors or smaller tweaks that don’t affect public behaviour can be left out.

## Write bug reports with detail, background, and sample code

**Great Bug Reports** tend to have:

- A quick summary and/or background
- Steps to reproduce
- What you expected would happen
- What actually happens
- Notes (possibly including why you think this might be happening, or stuff you tried that didn't work)

## Any contributions you make will be under the MIT Software License
In short, when you submit code changes, your submissions are understood to be under the same [MIT License](http://choosealicense.com/licenses/mit/) that covers the project. Feel free to contact the maintainers if that's a concern.
