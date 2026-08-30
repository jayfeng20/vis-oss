# vis-oss

Understand an open-source issue before you try to fix it.

Working on an unfamiliar codebase means answering the same questions every time: what
does this do today, where do I start reading, has this project already solved something
like it, what did the maintainers leave undecided. vis-oss creates a directory for one
issue, an agent fills it in, and you get a written answer you can keep.

## Install

```sh
cargo install --git https://github.com/jayfeng20/vis-oss vis-oss
```

You need `git`, and `gh` authenticated (`gh auth login`) to read the issue — without it
you still get a directory, just with a stub header instead of the issue text.

No API key or configuration. vis-oss never invokes an agent — `--install-command` writes
a markdown prompt file into your agent CLI's own directory, and that agent runs vis-oss.
Claude Code and Codex are recognised; only directories that already exist are written
to.

## Use

Once, to install the `/vis-oss` command into whichever agent CLIs you have:

```sh
vis-oss --install-command
```

Then, from inside your clone of the project, with the number of the issue you want to
study:

```
/vis-oss 804
```

Your agent creates the study and fills it in. You get:

```
~/vis-oss/lance-format/lance/804/
  CONTEXT.md    the study — what the issue is, where the code is, what to decide
  AGENTS.md     the contract the agent followed
  examples/     today's behaviour, and the behaviour you are aiming for
```

[Here is a finished one](examples/lance-format/lance/804/CONTEXT.md), for
[an open issue in lance](https://github.com/lance-format/lance/issues/804).

Without an agent, `vis-oss 804` creates the same directory and you fill in `CONTEXT.md`
yourself — it is a markdown file with headings.

The repository, root and commit come from git. `upstream` wins over `origin`, since on a
fork that's where the issue lives. If your clone is behind upstream, vis-oss says so and
asks before writing, because a study describes code at one commit.

| | |
|---|---|
| `vis-oss 804 ~/notes` | File under a different base. `<owner>/<name>/<issue>/` is appended either way |
| `--repo owner/name` | Override the inferred repository |
| `--source <path>` | Study a checkout other than the enclosing one |
| `--solution` | Finished code in examples, instead of exercises for you to complete |
| `--yes` | Don't prompt when the checkout is behind upstream |
| `--install-command` | (Re)install the `/vis-oss` command and exit |

## License

MIT
