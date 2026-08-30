# vis-oss

Visualize open-source software — so an unfamiliar issue is legible before you change
anything.

Picking up an issue in a repository you did not write means answering the same
questions every time: what does this actually do today, where does that live, has
someone already solved this nearby, what did the maintainers leave undecided. Agents
can answer all of that faster than you can. What they cannot do is keep the answer
*true* — a line number is correct for exactly as long as nobody rebases.

vis-oss is the layer in between. An agent investigates and emits a **study**; vis-oss
owns the skeleton, the validation, and the presentation — and re-resolves every code
reference on demand, so a study that has gone stale says so instead of quietly pointing
you at the wrong code.

```
gh + git ──► vis-oss init ──► study.json ──► agent fills it ──► vis-oss render ──► you
                                   ▲                                  │
                                   └────── vis-oss repair ◄───────────┘
                                          (the code moved; fix the anchors)
```

## Why this exists

A study is a set of claims about where things are in a codebase, and those claims decay
continuously. The failure is silent and expensive: a line number that once pointed at
the function you cared about now points at unrelated code, and you believe it. Three
things follow, and vis-oss addresses all three:

- **Anchors carry symbols, not just lines.** A moved symbol is reported as *moved to
  line 5305*, not as a failure — so a stale study is repairable instead of merely wrong.
- **Drift is fixable by machine.** `repair` already knows where the symbol went; making
  a human hand-edit JSON with that information is waste.
- **A study outlives one sitting.** It is a directory you return to over days of
  implementation, so it renders to markdown as readily as to a terminal.

## Install

```sh
cargo install --git https://github.com/jayfeng20/vis-oss vis-oss
```

Or from a clone: `cargo build --release` → `target/release/vis-oss`.

Needs `git`, and `gh` (authenticated) for `init`.

## Use

Run from inside a checkout of the project you are studying:

```sh
vis-oss init 804                 # repo, root and commit come from git
vis-oss validate study-804       # complete? anchors still resolve?
vis-oss render   study-804       # read it
vis-oss repair   study-804       # the code moved; fix the line numbers
```

`init` infers the repository from your git remotes and **prefers `upstream` over
`origin`** — on a fork, `origin` is your copy and the issues live upstream.

| Command | |
|---|---|
| `init <number>` | Create the skeleton. `--repo`, `--out`, `--source`, `--solution` |
| `validate [path]` | Completeness and drift. Non-zero exit on error |
| `render [path]` | `--format term\|markdown`, `--section`, `--no-check`, `--width`, `--color` |
| `repair [path]` | Rewrite moved anchors, re-pin. `--dry-run` |
| `schema` | JSON Schema, for agents to emit against |

`render --format markdown > context.md` if you want the study in version control.

## Tutorial mode

Examples are written as exercises by default — comments and `TODO`s directing you to
write the code — because the goal is that you learn the codebase, not that you obtain a
script. `init --solution` emits finished code instead.

## What drift detection looks like

```
$ vis-oss validate study-804
warn: checkout has moved: study pinned to 4a54e5dde but /Users/me/lance is at f603c5516
warn: entry_points rust/lance/src/dataset/scanner.rs:4226: moved to line 5305
error: entry_points rust/lance/src/dataset/scanner.rs:100: symbol no longer in file

$ vis-oss repair study-804
entry_points rust/lance/src/dataset/scanner.rs: 4226 -> 5305
needs a human: entry_points rust/lance/src/dataset/scanner.rs — symbol no longer in file
```

A symbol that merely moved is fixed. One that vanished may have been renamed, deleted,
or moved to another file — that is a judgement call, so it is left for you.

## Writing a study

See [docs/agent-contract.md](docs/agent-contract.md) for what each field means and what
makes a study good rather than merely valid. `examples/study-804/` is a real one.

## Relationship to caliper

[caliper](https://github.com/jayfeng20/caliper) renders agent findings for a PR you are
reviewing. Its artifact is ephemeral and decision-shaped: triage, post, discard. A
vis-oss study is durable and reference-shaped: you return to it while implementing, and
it has to survive the code moving underneath it. Same philosophy — agents emit
structure, Rust owns presentation — different lifetime, which is why only one of them
needs drift detection.

## Status

**v0.1 — init, validate, render, repair.** Usable today.

## License

MIT
