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
  CONTEXT.md          the study — a step-by-step lesson: what the issue is, a walkthrough
                      into the code, exercises, what to decide
  AGENTS.md           the contract the agent followed
  00_*.py, 01_*.py    probes of today, each ending in what changes
  01_*.rs             the same probe in the language the fix lands in, where the API allows
```

Studies are filed as `<root>/<repo-name>/<owner>/<issue>/`, so the root stays browsable by
project.

[Here is a finished one](examples/lance/lance-format/804/CONTEXT.md), for
[an open issue in lance](https://github.com/lance-format/lance/issues/804).

A study reads like a textbook chapter about one issue. `CONTEXT.md` walks you step by
step from the behaviour — run this, you should see that — into the code that produces
it, so by the end you know enough to hold your own opinion about the fix. Its exercises
deliberately reserve the judgment calls — the decisions, the tradeoffs, the domain
concepts — for you: the study exists to grow a contributor, not to replace one. The probes are
where the walkthrough touches ground: runnable files with comments naming the function
or struct each call reaches and where it is declared, and an `AFTER` block saying what
would differ once the issue is fixed. They never patch the project.

Each probe comes twice: in the language the behaviour is *observed* in — what a user
actually hits — and in the language it is *implemented* in, which runs against your own
working tree and so doubles as an acceptance check once you make the change.

**They are drafts.** The agent writes them but never runs or compiles them: both cost
whatever the project charges to build — for a Rust workspace, its entire dependency graph —
and that is work you repeat anyway on your own tree. So expect the Rust one to need a fix or
two on first `cargo check`; that is the trade for a study that arrives in minutes. Every
file records which paths were read, at which commit, and says plainly that it is
unexecuted. They also arrive one or two stubs short of running — the study's exercises,
see `--tutorial` — and if
executing something would settle a question, the agent offers rather than spending your
afternoon on it.

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

### The flags

Two groups, and they refuse to mix — a setup flag next to an issue number is an error,
not a guess about which you meant.

**Setup** — run yourself, in a terminal, rarely. Each does its one job and exits:

| | |
|---|---|
| `--install-command` | (Re)install the `/vis-oss` command and exit |
| `--set-root <path>` | Remember a root for every later run, and exit |
| `--update` | Reinstall the latest vis-oss, refresh the agent command, and exit |

**Creating a study** — everything else rides along with the issue number, and means the
same whether you type it after `/vis-oss` in your agent or after `vis-oss` in a terminal
(the slash command passes your arguments straight through):

| | |
|---|---|
| `vis-oss 804 ~/notes` | Use a different root for one run. The `<repo-name>/<owner>/<issue>/` layout still applies, so this writes `~/notes/lance/lance-format/804/` |
| `--repo owner/name` | Override the inferred repository |
| `--source <path>` | Study a checkout other than the enclosing one |
| `--tutorial full\|partial\|none` | How much is written for you. `full` is complete code, `partial` (default) leaves the parts worth thinking about as exercises, `none` is `TODO`s |
| `--note <text>` | A lead for the agent to verify. Repeatable |
| `--redo` | Delete an existing study and start fresh. The old one is not kept |
| `--sync-upstream` | Fast-forward the checkout onto the canonical remote before writing. On its own, `vis-oss --sync-upstream` just syncs and exits |

## License

MIT
