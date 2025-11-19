# Contributing to vexide

Thanks for taking the time to help this project improve! Your contributions are
helpful and welcome.

## I have a question!

If you simply have a question about vexide or need help using it, the best way you can
get support is by asking in our active [Discord Server][discord-server].

## Ways to contribute

### Reporting a problem

If something is not working as expected, you can use the repository's
[Issues][issues-page] page to report it. Before
creating a bug report, use the search bar to make sure that what you're
experiencing isn't already a known issue.

#### If you find an issue that describes the same problem

If the issue you found is closed, feel free to make a new one, but it helps
to link the one you found under the **Additional information** header.

If the issue you found is open, the best way to help is by leaving a
comment on it describing your experience, or by joining our
[Discord server][discord-server] and telling us about it.

#### Reporting a small problem

If you're reporting a typo or a simple mistake, submit an issue using the
**Small issue** template, which requires less details than a full bug report.

#### Writing and submitting your report

When creating your report, you should use the **Bug report** issue template to
be provided with a list of questions that will help describe the problem you are
having.

Additionally, try to do the following:

- Give the issue a **clear and concise** title.
- Fill out **as many of the template's headers as possible**.
- Provide a **code sample** to help readers reproduce the issue.
- Provide your Rust version, vexide version, and operating system.
- If you have **screenshots, photos, or videos**, attach them to the GitHub issue.
- Explain **when the problem started happening**. Was it after a recent update?
  Or has it always been an issue?

### Suggesting features

Thanks for sharing your idea! Before submitting your suggestion, please:

- Check if your idea is already being discussed by using the [Issues][issues-page]
  search bar to search for similar suggestions.
- Ensure your idea is within the project's scope: to provide an opinionated Rust
  framework for developing VEX V5 robots.
  * If your idea is about motion control (PID, motion profiles, etc), then you might
    be interested in [`evian`][evian].
  * If you're interested in robot simulation support, check out vexide's [simulator].
  * If you want to contribute to our low-level bindings to the VEX SDK, check out\
    [`vex-sdk`][vex-sdk].

#### Writing and submitting your suggestion

When creating your report, you should use the **Feature request** issue template
to be provided with a list of questions that will help describe the suggestion
you are submitting.

Additionally, try to do the following:

- Give the issue a **clear and concise** title.
- Fill out **as many of the template's headers as possible**.
- Provide **code samples, photos, or videos** to help readers understand what
  you're saying.
- Explain **how the suggestion would be implemented**.


### Contributing code

The simplest ways to start contributing code to vexide are by finding an unresolved [Issue][issues-page]
or by asking on our [Discord server][discord-server]. Issues with the [good first issue][first-issue-search]
label are good candidates for your first contribution.

#### Code styleguide

All Rust source code should be formatted with Rustfmt, by running `cargo fmt` after making changes.

Use Clippy to lint your changes: `cargo clippy`.

In files not formatted by Rustfmt, there should be no trailing whitespace, the end of line
sequence should be LF (line feed), and the file should end with one trailing newline.

#### Committing & commit messages

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

From this commit, you can easily see that the commit altered **docs** in the
**contributing** guidelines file by **add**ing an **Acknowledgements section**.
When writing the commit description, make sure to use the present imperative
tense ("add ABC" instead of "added ABC" or "adds ABC"). It might help to imagine
you're telling someone to do something ("go add ABC").

Here is a list of common commit types:

| Type | Description |
|------|------------|
| chore | Changes to workspace & configuration files |
| feat | New features |
| fix | Bug fixes |
| refactor | Changes to internal features but not the external interface |
| revert | Reversion of a previous change |
| style | Changes to code style and formatting |
| test | Changes or additions to unit tests |
| types | Changes to type definitions |
| docs | Changes to documentation files |

<!--
#### Unit tests

TODO
-->

#### Changelog

After making changes to your code, update the Unreleased section of the changelog with what you changed. Breaking changes should be [painfully clear][ignoring-deprecations], so list all deprecations, removals, and generic breaking changes. Include your pull request's number. See the example below for the recommended format.

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

#### Pull requests

When you're ready for your changes to be merged, head over to the [Pull
Requests][pr-page] page and create a new pull request. Include a description of
what changed, and [link to an Issue][link-to-issue-guide] if applicable. Pull request names
should follow the same conventions as [commit messages](#committing--commit-messages).

If you're not quite done with the changes but are ready to start sharing them, you can
[mark it as a draft][about-draft-prs] to prevent it from being merged.

Once your pull request has been merged, congrats! Your changes will be mentioned
in the next release's changelog.

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
