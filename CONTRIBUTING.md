# Contributing to vexide

Thanks for taking the time to help this project improve! Your contributions are helpful and welcome.

Before you start contributing, please take a moment to review the guidelines outlined here. This ensures a smooth collaboration and helps maintain the project's quality.

## I have a question!

If you simply have a question about vexide or need help using it, the best way you can
get support is by asking in our active [Discord Server][discord-server]. Several core maintainers are active in there and happy to help!

## Using the Issue Tracker

The [issues page][issues-page] is used to report bugs and request new features. Before opening an issue, please use the search bar to make sure that the problem or feature hasn't already been opened.

### Reporting a Problem

If something is not working as expected, you can use the **Bug report** template.

For some general guidelines on issue reporting:
- Give the issue a clear and concise title.
- Fill out as many of the template's fields as possible.
- Provide a code sample to help readers reproduce the issue.
- Provide your Rust version, vexide version, and operating system.
- If you have screenshots, photos, or videos, attach them to the GitHub issue.
- Explain when the problem started happening. Was it after a recent update? Or has it always been a problem?

If you're reporting a typo or a simple mistake, submit an issue using the **Small issue** template, which requires less details than a full bug report.

### Suggesting a Feature

If you have an idea on a new feature or improvement to vexide, you can use the **Feature request** template.

> In some cases, your request may be denied if it is deemed *out of scope*, meaning the feature belongs elsewhere. The `vexide` organization maintains several sister projects that exist outside of the main repository for this (like [evian](https://github.com/vexide/evian/) for motion control).

## Contributing code

The simplest ways to start contributing code to vexide are by finding an unresolved [Issue][issues-page]
or by asking on our [Discord server][discord-server]. Issues with the [good first issue][first-issue-search]
label are good candidates for your first contribution.

Some specific aspects of vexide's internals are documented on our [internal docs page][https://internals.vexide.dev/].

### Use of AI/LLMs

Contributions are expected to be *written*, *understood*, *reviewed*, and *maintained* by humans. vexide does not explicitly forbid the use of LLMs for assisting development, but contributors that use these tools will be held to high standards:

1. Do not submit changes that you do not understand and/or could not write yourself.

    > **Rationale:** Contributors are expected to both fully understand the code that they write **and** have the necessary skills to *maintain it*. Opening PRs containing code that you did not write more often than not fails to meet either of these expectations and wastes the time of reviewers.
3. Code should not be recognizably "vibe-coded" or AI-written. This includes excessive overuse of comments, throwaway utility functions, and documentation clearly written in "LLM-prose".

    > **Rationale:** LLM-assisted contributions are held to the same quality as any other PR. If we can tell that a human didn't write it, then it has failed to meet our expectation of code quality.
4. Don't add "Assisted-by: [some LLM]" tags to your commits. Pull requests that do this will be closed.

    > **Rationale:** Refer to points 1 and 2. Doing this is just free advertising for the LLM's provider.

### Code Style & Formatting

All Rust source code should be formatted with Rustfmt, by running `cargo fmt` after making changes. vexide loosely follows the [Rust styleguide](https://doc.rust-lang.org/style-guide/) as a standard of code quality.

Use Clippy to lint your changes: `cargo clippy`.

In files not formatted by Rustfmt, there should be no trailing whitespace, the end of line
sequence should be LF (line feed), and the file should end with one trailing newline.

ARM assembly language files should prefer at-sign (`@`) comment syntax over double slash (`//`) comment syntax.

### Keep scope to a minimum.

Pull requests should ideally do one thing in one place. Avoiding opening massive pull requests that change multiple unrelated modules. These types of pull requests are often not reviewable and result in unmanageable conflicts with other active PRs.

### Test your changes.

Please run and test your changes on real hardware or in an [emulator](https://github.com/vexide/vex-v5-qemu) if possible. If you are unable to do so, please mention it in your pull request's description so that a reviewer can test your changes.

vexide is tested through a series of both unit tests (which run on the host in a mock environment) and integration tests (which run on a real brain in bulk). Details on how to run these can be found in the project's [README](https://github.com/vexide/vexide/#testing).

### Try to fix the cause, not the effect.

If you are fixing a bug, avoid submitting "hacks" that attempt to patch the effects of the bug rather than the root cause.

### Committing & commit messages

All vexide projects use [Conventional Commits][conventional-commits-website]
to ensure commit messages are useful. Conventional commits have the following form:

```
type(OptionalScope): description

[optional body]

[optional footers]
```

Here is an example of a conforming commit message:

```
docs(contributing): add Acknowledgements section
```

<!--
#### Unit tests

TODO
-->

### Changelog

After making changes to your code, update the Unreleased section of the [changelog](./CHANGELOG.md) with what you changed. Breaking changes should be [painfully clear][ignoring-deprecations], so list all deprecations, removals, and generic breaking changes. Include your pull request's number. See the example below for the recommended format.

```diff
  ## [Unreleased]

  ### Added

  ### Fixed

  ### Changed

+ * All functions in the `foo` module now
+   must be passed a Bar struct. (**Breaking change**) (#30)

  ### Removed

+ * Removed the deprecated `bar` module. Use the
+   `foo` module instead. (**Breaking change**) (#28)

  ### Deprecated

+ * The `Baz` struct is now deprecated. (#28)
```

### Pull requests

When you're ready for your changes to be merged, head over to the [Pull
Requests][pr-page] page and create a new pull request. Include a description of
what changed, and [link to an Issue][link-to-issue-guide] if applicable. Pull request names
should follow the same conventions as [commit messages](#committing--commit-messages).

If you're not quite done with the changes but are ready to start sharing them, you can
[mark it as a draft][about-draft-prs] to prevent it from being merged.

Once your pull request has been merged, congrats! Your changes will be mentioned
in the next release's changelog.

## Versioning

This project's alphas, betas, and release candidates are kept on separate branches which diverge from `main` as needed. The project versions on the `main` branch are always kept on the **next** version of the library.

For example, if `v0.8.0-alpha.1` is ready to be released, then the `alpha` branch will be rebased off `main`. Then, a commit will be made changing all the `0.8.0`s to `0.8.0-alpha.1`s. Finally, all crates will be published off that branch.

This means that we can release many alphas or betas without clogging up the commit history of main.

### Updating versions

While crates like `vexide-startup` have their own version number, there is a concept of the current "flagship" vexide version which is stored in various places throughout the project.

- The `vexide` crate's version is the canonical project flagship version, and must be updated if there is a breaking change to any vexide sub-crate.
- In the changelog, the most recent version must be the flagship version.
- The [VEXIDE_VERSION] constant in `vexide-startup` must be the flagship version.

These must be updated before a pre-release is published or when `main` is updated to point to the next version after a full release.

[VEXIDE_VERSION]: packages/vexide-startup/src/banner/mod.rs

## Acknowledgements

This CONTRIBUTING.md file contains excerpts from and was inspired in part by the
Atom editor's CONTRIBUTING.md. [Click here to go check it
out.][atom-contributing]

[discord-server]: https://discord.gg/DhfnWNX7ah
[issues-page]: https://github.com/vexide/vexide/issues
[pr-page]: https://github.com/vexide/vexide/pulls
[first-issue-search]:
    https://github.com/vexide/vexide/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22
[conventional-commits-website]: https://conventionalcommits.org
[ignoring-deprecations]: https://keepachangelog.com/en/1.1.0/#ignoring-deprecations
[link-to-issue-guide]:
    https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue
[about-draft-prs]:
    https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/about-pull-requests#draft-pull-requests
[atom-contributing]: https://github.com/atom/atom/blob/master/CONTRIBUTING.md
[internal-docs]: (https://internals.vexide.dev/)
