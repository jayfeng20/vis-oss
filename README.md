# vis-oss

Understand an open-source issue before you try to fix it.

Working on an unfamiliar codebase means answering the same questions every time: what
does this do today, where do I start reading, has this project already solved something
like it, what did the maintainers leave undecided. vis-oss creates a directory for one
issue, an agent fills it in, and you get a written answer you can keep.

**IMPORTANT**: An AI agent can absolutely make mistakes especially on complex issues. The study it 
generates helps contributors familiarize themselves with issues faster, serves as a helpful starting point,
and is not meant to "generate the final solution for the issue".

## Install

```sh
cargo install --git https://github.com/jayfeng20/vis-oss
```

That builds the binary and puts it in `~/.cargo/bin`, which rustup already added to your
`PATH`, so `vis-oss` works from any directory. Nothing is installed into the projects you
study.

To upgrade later:

```sh
vis-oss --update
```

That reinstalls from source and refreshes the `/vis-oss` agent command in the same step,
because the command is a copy of what the binary knows and a stale one quietly tells your
agent the wrong thing. (By hand it is
`cargo install --git ... --force && vis-oss --install-command`; the `--force` matters,
since the version does not change between commits and cargo otherwise does nothing.)

You need `git`, and `gh` authenticated (`gh auth login`) to read the issue — without it
you still get a directory, just with a stub header instead of the issue text.

No API key or configuration. vis-oss never invokes an agent — `--install-command` writes
a markdown prompt file into your agent CLI's own directory, and that agent runs vis-oss.
Claude Code and Codex are recognised; only directories that already exist are written
to.

## Use

### Once, in a terminal

```sh
vis-oss --install-command                 # adds a /vis-oss command to your agent CLIs
vis-oss --set-root ~/Coding/OSS-study     # where studies live; defaults to ~/vis-oss
```

`--install-command` writes a prompt file into `~/.claude/commands/` and `~/.codex/prompts/`,
whichever already exist. `--set-root` saves one path to `~/.config/vis-oss/root`. Neither
touches the project you are contributing to.

### Per issue, in your agent

Start the agent **from inside your clone of the project** — vis-oss reads the repository,
remotes and commit from wherever it is run, so the directory you are in is what decides
which project gets studied.

```sh
cd ~/Coding/lance      # your clone of the project
claude                 # or whichever agent you installed the command into
```

Then at the agent's prompt, not the shell's:

```
/vis-oss 804
```

`804` is the issue number. The agent runs `vis-oss 804` for you, reads the contract that
lands next to the study, investigates the codebase, and writes the study. You get:

```
~/Coding/OSS-study/lance/lance-format/804/
  CONTEXT.md          the study — what the issue is, where the code is, what to decide
  AGENTS.md           the contract the agent followed
  00_*.py, 01_*.py    runnable probes of today, each ending in what changes
```

Studies are filed as `<root>/<repo-name>/<owner>/<issue>/`, so the root stays browsable by
project.

[Here is a finished one](examples/lance/lance-format/804/CONTEXT.md), for
[an open issue in lance](https://github.com/lance-format/lance/issues/804).

The examples are the point: runnable probes of what the code does today, with comments
naming the function or struct each call reaches and where it is declared, and an `AFTER`
block saying what would differ once the issue is fixed. They never patch the project.

The agent writes them but never runs them: a first execution costs whatever the project
charges to build, which is unbounded and is work you repeat anyway on your own tree. The
one mechanical check is `cargo check` for Rust, where the compiler catches a moved
signature cheaply; for Python there is no equivalent worth running, so the check is
reading the source, and each file says which paths it read. They arrive one or two stubs
short of running — see `--tutorial` — and if executing something would settle a question,
the agent offers rather than spending your afternoon on it.

### Or without an agent, in a terminal

```sh
cd ~/Coding/lance
vis-oss 804
```

Same directory, same contract, but it stops after creating them — `CONTEXT.md` arrives as
a markdown file with empty headings and the issue body, and you fill it in yourself.

If you already know something useful, say so up front — `--note` puts it at the top of
the study as a lead for the agent to verify:

```sh
vis-oss 804 --note "flat FTS already warns like this: inverted/index/flat_search.rs:449"
```

Running it again for an issue you have already studied never overwrites anything. It
prints where the study is, and whether the checkout has moved since it was written:

```
already studied: ~/Coding/OSS-study/lance/lance-format/804
  written against 4a54e5dde, checkout is now at 324cedd9d
  its file references may have moved; re-read before relying on them
```

### What it reads from where you run it

The repository, root and commit come from git. `upstream` wins over `origin`, since on a
fork that's where the issue lives.

A study describes code at one commit, so if your clone is behind, vis-oss says so and asks
before writing. Since you are about to start work anyway, `--sync-upstream` fast-forwards
the checkout first:

```sh
vis-oss 804 --sync-upstream
# synced: fast-forwarded 387 commit(s) to upstream/main
```

It only ever fast-forwards, and it refuses — telling you why, then falling back to the
prompt — if you are on another branch, the working tree is dirty, or your branch has
commits the remote does not. It will not merge, rebase, stash, or touch a branch other
than the one you have checked out.

| | |
|---|---|
| `vis-oss 804 ~/notes` | Use a different root for one run. The `<repo-name>/<owner>/<issue>/` layout still applies, so this writes `~/notes/lance/lance-format/804/` |
| `--set-root <path>` | Remember a root for every later run, and exit |
| `--repo owner/name` | Override the inferred repository |
| `--source <path>` | Study a checkout other than the enclosing one |
| `--tutorial full\|partial\|none` | How much of the examples you write yourself. `partial` (default) leaves the parts worth thinking about as stubs, `none` is complete code, `full` is `TODO`s |
| `--note <text>` | A lead for the agent to verify. Repeatable |
| `--redo` | Archive an existing study and start fresh |
| `--sync-upstream` | Fast-forward the checkout onto the canonical remote. With an issue number it runs first; on its own, `vis-oss --sync-upstream` just syncs |
| `--update` | Reinstall the latest vis-oss, refresh the agent command, and exit |
| `--install-command` | (Re)install the `/vis-oss` command and exit |

## License

MIT
