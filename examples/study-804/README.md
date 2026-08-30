# lance-format/lance #804

Display warning if the index is not built for a vector column during query

https://github.com/lance-format/lance/issues/804

A **vis-oss study**: a structured investigation of one open-source issue, so you can
understand it before you change anything.

## Layout

| Path | What it is |
|---|---|
| `study.json` | The study itself. Source of truth — everything else is derived. |
| `examples/` | Runnable files: what the code does today, and what it should do after. |
| `README.md` | This file. |

## Reading it

```sh
vis-oss render .        # the study, formatted, with drift warnings
vis-oss validate .      # is it complete, and has the code moved under it?
```

`render` re-resolves every code reference against the checkout, so a study that has
gone stale says so instead of quietly pointing at the wrong lines. To keep a copy in
version control: `vis-oss render . > context.md`.

## Filling it in

`study.json` starts mostly empty; an investigating agent fills it. See the
[agent contract](https://github.com/jayfeng20/vis-oss/blob/main/docs/agent-contract.md)
for what each field means and what makes a study good rather than merely valid.
