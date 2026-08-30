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
cargo install --git https://github.com/jayfeng20/vis-oss vis-oss
```

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

### Or without an agent, in a terminal

```sh
cd ~/Coding/lance
vis-oss 804
```

Same directory, same contract, but it stops after creating them — `CONTEXT.md` arrives as
a markdown file with empty headings and the issue body, and you fill it in yourself.

### What it reads from where you run it

The repository, root and commit come from git. `upstream` wins over `origin`, since on a
fork that's where the issue lives. If your clone is behind upstream, vis-oss says so and
asks before writing, because a study describes code at one commit.

| | |
|---|---|
| `vis-oss 804 ~/notes` | Use a different root for one run. The `<repo-name>/<owner>/<issue>/` layout still applies, so this writes `~/notes/lance/lance-format/804/` |
| `--set-root <path>` | Remember a root for every later run, and exit |
| `--repo owner/name` | Override the inferred repository |
| `--source <path>` | Study a checkout other than the enclosing one |
| `--tutorial full\|partial\|none` | How much of the examples you write yourself. `none` (default) is complete code, `partial` leaves stubs, `full` is `TODO`s |
| `--install-command` | (Re)install the `/vis-oss` command and exit |

## License

MIT
