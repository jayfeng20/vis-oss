# vis-oss

Understand an open-source issue before you try to fix it.

Picking up an issue in a project you did not write means answering the same questions
every time. What does this code actually do today? Where do I start reading? Has this
codebase already solved something like it somewhere else? What did the maintainers
leave undecided? Answering those well takes hours, they are the difference between a PR
that merges and one that stalls, and the answers evaporate the moment you close the tab.

vis-oss turns that work into an artifact. From inside your clone of a project:

```sh
vis-oss 804
```

You get a directory, filed outside the project so it can never end up in the PR you are
preparing:

```
~/vis-oss/lance-format/lance/804/
  CONTEXT.md    the study — headings in place, issue body pasted in
  AGENT.md      what a good study contains
  examples/     today's behaviour, and the behaviour you are aiming for
```

Point an agent at it, tell it to follow `AGENT.md`, and read the markdown.

## What this program does, and does not

It reads the issue and the checkout, creates the directory, and writes a `CONTEXT.md`
skeleton with the agent contract beside it. That is all of it.

It does not invoke an agent, and it does not parse a study back in. Both were tried and
removed. Orchestration makes the binary non-deterministic and hard to test. A schema in
the middle was worse: the agent wrote markdown prose into JSON string fields so that a
renderer could turn it back into markdown — a round trip that produced what the agent
already had.

So the study is markdown, because a study is prose. Everything that makes one *good*
lives in [the contract](docs/agent-contract.md), which is also prose.

## Install

```sh
cargo install --git https://github.com/jayfeng20/vis-oss vis-oss
```

Or from a clone: `cargo build --release` → `target/release/vis-oss`.

Needs `git`, and `gh` (authenticated).

## Use

Run from inside your clone of the project you want to contribute to:

```sh
vis-oss 804                    # -> ~/vis-oss/<owner>/<name>/804/
vis-oss 804 ~/notes            # -> ~/notes/<owner>/<name>/804/
```

The positional path sets the *base*; the `<owner>/<name>/<issue>/` layout is appended
either way, so one base holds every issue you have looked at.

The repository, root and commit come from git. `upstream` wins over `origin`, because on
a fork `origin` is your copy and the issue lives upstream.

| Flag | |
|---|---|
| `--repo owner/name` | Override the inferred repository |
| `--source <path>` | Study a checkout other than the enclosing one |
| `--solution` | Finished code in examples instead of exercises |
| `--yes`, `-y` | Do not prompt when the checkout is behind upstream |

## Tutorial mode

Examples are exercises by default — comments and `TODO`s telling you precisely what to
write, without writing it for you. The point of studying an issue is to come away able
to work in the codebase, which does not happen if you are handed a script. `--solution`
writes finished code.

## Staleness

A study describes code at one commit. If your clone is behind upstream it describes code
that has already changed, and every file reference in it will be quietly wrong. So
vis-oss checks before writing anything:

```
$ vis-oss 804
warning: this checkout is 387 commit(s) behind upstream/main.
         A study written now will describe code that has already moved.

  to sync first:
    git -C ~/Coding/lance fetch upstream
    git -C ~/Coding/lance merge --ff-only upstream/main

Continue with the stale checkout? [y/N]
```

Syncing stays your decision — vis-oss tells you and gets out of the way.

## A worked example

[`examples/lance-format/lance/804/`](examples/lance-format/lance/804/) is a real, filled-in
study of [an open issue in lance](https://github.com/lance-format/lance/issues/804),
including the three example files it refers to.

## Relationship to caliper

[caliper](https://github.com/jayfeng20/caliper) is for reviewing someone else's PR: you
triage findings, post, and move on. vis-oss is for understanding a codebase well enough
to contribute to it, and its output is something you keep.

## Status

**v0.1.** One command.

## License

MIT
