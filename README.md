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

No agent setup, API key or configuration. vis-oss never invokes an agent; it writes
markdown files. Any coding agent that can read and write files can fill them in, and so
can you.

## Use

From inside your clone of the project:

```sh
vis-oss 804
```

```
~/vis-oss/lance-format/lance/804/
  CONTEXT.md    the study — headings in place, issue body pasted in
  AGENT.md      what a good study contains
  examples/     today's behaviour, and the behaviour you are aiming for
```

Then point an agent at it:

> Read `~/vis-oss/lance-format/lance/804/AGENT.md` and fill in `CONTEXT.md` in that
> directory. I'm working in `~/Coding/lance`.

Read the markdown when it's done. [Here is a finished
one.](examples/lance-format/lance/804/CONTEXT.md)

The repository, root and commit come from git — `upstream` wins over `origin`, since on
a fork that's where the issue lives. If your clone is behind upstream, vis-oss says so
and asks before writing, because a study describes code at one commit.

| | |
|---|---|
| `vis-oss 804 ~/notes` | File under a different base. `<owner>/<name>/<issue>/` is appended either way |
| `--repo owner/name` | Override the inferred repository |
| `--source <path>` | Study a checkout other than the enclosing one |
| `--solution` | Finished code in examples, instead of exercises for you to complete |
| `--yes` | Don't prompt when the checkout is behind upstream |

## License

MIT
