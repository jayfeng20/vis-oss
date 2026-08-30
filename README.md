# vis-oss

Understand an open-source issue before you try to fix it.

Picking up an issue in a project you did not write means answering the same questions
every time. What does this code actually do today? Where do I start reading? Has this
codebase already solved something like it somewhere else? What did the maintainers
leave undecided? Answering those well takes hours, they are the difference between a
PR that merges and one that stalls, and the answers evaporate the moment you close the
tab.

vis-oss makes that work into an artifact. You fork a project, clone it, and run one
command inside it; you get a directory holding a structured explanation of the issue —
where the relevant code lives, what already exists nearby, what you will have to
decide — plus runnable examples of the behaviour today and the behaviour you are aiming
for.

```
                    you, in your clone of the project
                                  │
                    vis-oss init 804
                                  │
   ~/vis-oss/lance-format/lance/804/ ── study.json   the structured explanation
                                     ├── examples/    today's behaviour, and the target
                                     └── README.md
                                  │
                    an agent investigates and fills it in
                                  │
                    vis-oss render ──► you understand the issue
```

Studies are filed **outside** the project, under `~/vis-oss/<owner>/<name>/<issue>/`.
Keeping them out of the tree means a study can never turn up in `git status` or get
swept into the PR you are preparing — the risk is removed rather than defended against.

## Install

```sh
cargo install --git https://github.com/jayfeng20/vis-oss vis-oss
```

Or from a clone: `cargo build --release` → `target/release/vis-oss`.

Needs `git`, and `gh` (authenticated) for `init`.

## Use

From inside your clone of the project you want to contribute to:

```sh
vis-oss init 804                              # -> ~/vis-oss/<owner>/<name>/804/
vis-oss render ~/vis-oss/lance-format/lance/804
```

`init` reads the repository, root and commit from git. It **prefers the `upstream`
remote over `origin`**, because on a fork `origin` is your copy and the issue lives
upstream.

Pass a base directory to file studies somewhere else — the
`<owner>/<name>/<issue>/` layout is appended either way, so one base can hold every
issue you have ever looked at:

```sh
vis-oss init 804 ~/notes        # -> ~/notes/lance-format/lance/804/
```

| Command | |
|---|---|
| `init <number> [base]` | Create the study. `--repo`, `--source`, `--solution`, `--yes` |
| `render [path]` | Read it. `--format term\|markdown`, `--section`, `--width`, `--color` |
| `validate [path]` | Is it complete? Do its code references still resolve? |
| `repair [path]` | Fix references the code has moved out from under. `--dry-run` |
| `schema` | JSON Schema, for agents to emit against |

## Tutorial mode

Examples are exercises by default — comments and `TODO`s telling you precisely what to
write, without writing it for you. The point of studying an issue is that you come away
able to work in the codebase, which does not happen if you are handed a script.
`init --solution` emits finished code instead.

## Staleness

A study describes code at a particular commit. If your clone is behind upstream, it
describes code that has already changed — and every file and line reference in it will
be quietly wrong. So `init` checks before it writes anything:

```
$ vis-oss init 804
warning: this checkout is 387 commit(s) behind upstream/main.
         A study written now will describe code that has already moved.

  to sync first:
    git -C ~/Coding/lance fetch upstream
    git -C ~/Coding/lance merge --ff-only upstream/main

Continue with the stale checkout? [y/N]
```

Syncing stays your decision — vis-oss tells you and gets out of the way. `--yes` skips
the prompt.

For a study you keep working from over days, `validate` re-resolves its code references
and `repair` corrects the ones that merely moved:

```
$ vis-oss repair ~/vis-oss/lance-format/lance/804
entry_points rust/lance/src/dataset/scanner.rs: 4226 -> 5305
```

## Writing a study

See [docs/agent-contract.md](docs/agent-contract.md) for what each field means and what
separates a study that helps from one that merely validates. `examples/study-804/` is a
real one, of an issue in [lance](https://github.com/lance-format/lance).

## Relationship to caliper

[caliper](https://github.com/jayfeng20/caliper) is for reviewing someone else's PR: you
triage findings, post, and move on. vis-oss is for understanding a codebase well enough
to contribute to it, and its output is something you keep. Same idea underneath — agents
emit structure, Rust owns presentation — different job.

## Status

**v0.1 — init, render, validate, repair.** Usable today.

## License

MIT
