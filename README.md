# ZenScript Toolkit

A [Zed](https://zed.dev) extension that bundles language support for
[CraftTweaker](https://crafttweaker.readthedocs.io)'s ZenScript plus
Minecraft `.lang` localization files, with an optional bracket-handler
completion server that reads the dumper output from
[`zenscript-bracket-completion`](https://github.com/Blue-Beaker/zenscript-bracket-completion).

Three VSCode extensions are unified into a single Zed extension:

- **ZenScript** — syntax highlighting, indent rules, and code-folding
  driven by a completed tree-sitter grammar.
- **vscode-minecraft-lang** — `.lang` file highlighting (keys, values,
  comments, escape sequences, and `${...}` substitutions).
- **zenscript-bracket-completion (ZSBC)** — LSP completions and hover
  for `<item:minecraft:diamond>`-style bracket handlers, populated from
  your `crafttweaker.log`.

The original **ZsLint** linter is not included — its WebSocket service
is archived and unmaintained (last touched ~8 years ago) and wiring it
into Zed would require shipping a sidecar LSP-to-WS bridge, which is
not appropriate for a modern editor extension.

## Repository layout

```
zenscript-toolkit/                         # this repo (Zed extension)
├── extension.toml                         # Zed extension manifest
├── Cargo.toml                             # workspace + the cdylib crate
├── src/lib.rs                             # downloads zsbc-lsp at activation
├── lsp-server/                            # the ZSBC language server
│   ├── Cargo.toml
│   └── src/{main,lsp,parse}.rs
├── languages/
│   ├── zenscript/{config.toml,highlights.scm}
│   └── minecraft-lang/{config.toml,highlights.scm}
├── dumpers/dumper_112.zs                  # 1.12 dumper (from ZSBC)
├── dumpers/dumper_116.zs                  # 1.16+ dumper (from ZSBC)
└── README.md, LICENSE, CHANGELOG.md
```

The repository is a Cargo workspace. The root `Cargo.toml` is both the
workspace manifest and the cdylib crate that Zed compiles to `cdylib`
(at install time, `cargo build --target wasm32-wasip2`); the
`lsp-server` subdirectory is a workspace member producing a standalone
binary that is **not** shipped as part of the extension per the
[Zed extension publishing guidelines](https://zed.dev/docs/extensions/developing-extensions#extension-publishing-prerequisites)
("Extensions that intend to provide a language … must not ship the
language server as part of the extension").

## Languages

| Language   | File extensions | Grammar source                                                                   | Language server                |
| ---------- | --------------- | -------------------------------------------------------------------------------- | ------------------------------ |
| ZenScript  | `.zs`, `.zsrc`  | [`Copernicium282/tree-sitter-zenscript`](https://github.com/Copernicium282/tree-sitter-zenscript) (fork of `ikexing-cn`) | `zsbc` (downloaded, see below) |
| Minecraft  | `.lang`         | [`tree-sitter-grammars/tree-sitter-properties`](https://github.com/tree-sitter-grammars/tree-sitter-properties) | —                              |

## ZenScript grammar

The upstream [`ikexing-cn/tree-sitter-zenscript`](https://github.com/ikexing-cn/tree-sitter-zenscript)
repository is a work in progress. As of the commit it was forked from,
the `class_declaration` rule was `choice('todo1')` and `_expression` was
`choice('todo4')`, which means complex expressions fall to `ERROR`
nodes. The fork at
[`Copernicium282/tree-sitter-zenscript`](https://github.com/Copernicium282/tree-sitter-zenscript)
completes the missing pieces, grounded in the official
[`CraftTweaker/ZenScript`](https://github.com/CraftTweaker/ZenScript)
Java parser (see `ZenTokener.java`, `parser/expression/ParsedExpression.java`,
and `statements/Statement.java` for token sets, operator precedence,
and statement grammar), which we treat as the source of truth.

The fork adds:

- A full expression grammar with explicit precedence levels: `=`, `?:`, `||`,
  `&&`, `|`, `^`, `&`, `==`/`!=`/`<`/`<=`/`>`/`>=`/`in`/`has`, `+`/`-`/`~`
  (string concat), `*`/`/`/`%`, unary `-`/`!`, postfix `.member`/`[index]`/
  `(call)`, `as Type`, `instanceof Type`, and the `..`/`to` range operators.
- `class_declaration` (zenClass / frigginClass) with a body of members.
- `constructor_declaration` (zenConstructor / frigginConstructor).
- `lambda_expression` (`function (params) as Type { ... }`).
- `version N;` as a real statement.
- A defined `formal_parameter_list` rule (the upstream grammar referenced it
  but never defined it, so the parser would not compile).
- The `has` operator (NuclearCraft's `loadedMods has "modid"` form).
- Bug fixes to the upstream's `return_statement` and similar rules, which
  referenced `$._expression` without taking `$` as a parameter.

## ZSBC bracket completions

The ZSBC language server is downloaded from this repo's GitHub
releases on first activation. The `extension/src/lib.rs` file calls
`zed::latest_github_release`, finds the asset named
`zsbc-lsp-{rust-target-triple}` for the current platform, downloads
it to Zed's extension working directory, marks it executable, and
spawns it for LSP.

To release a new version:

1. Tag the commit with `git tag vX.Y.Z && git push --tags`.
2. The `.github/workflows/release.yml` workflow builds the `zsbc-lsp`
   binary for `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
   `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`,
   and `aarch64-pc-windows-msvc`, gzips the non-Windows ones, and
   uploads them as release assets named `zsbc-lsp-{target}`.
3. Bump `version` in `extension.toml` (and in `Cargo.toml` /
   `extension/Cargo.toml` / `lsp-server/Cargo.toml` workspace members)
   on the next commit.

### ZSBC user setup

1. **Install CraftTweaker** in your modded Minecraft instance. The
   server has no other external dependencies.
2. **Drop a dumper script** in `.minecraft/scripts/`. The `dumpers/`
   directory contains two ready-to-use scripts:
   - `dumpers/dumper_112.zs` for CraftTweaker 1.12 (Minecraft 1.12.x).
   - `dumpers/dumper_116.zs` for CraftTweaker 4 (Minecraft 1.16.x).
3. **Launch Minecraft once** with CraftTweaker and the script loaded.
   The dumper writes a `[ZSBC DUMPER START] … [ZSBC DUMPER END]` block
   to `crafttweaker.log` in your `.minecraft` folder (1.12) or
   `.minecraft/logs/` folder (1.16+).
4. **Point the extension at the log** by adding a `zsbc` block to your
   `.zed/settings.json`:

   ```json
   {
     "zsbc": {
       "path": "/home/you/.minecraft/logs/crafttweaker.log",
       "alwaysReload": true,
       "onlyCompleteBrackets": true
     }
   }
   ```

5. **Edit a `.zs` file** in Zed. Type `<item:` and the editor will
   suggest every item the dumper discovered; hovering on a complete
   bracket such as `<item:minecraft:diamond>` shows the item's display
   name.

### ZSBC settings

All settings are read from the `zsbc` key in `.zed/settings.json`:

| Key                            | Type      | Default | Description                                                                                          |
| ------------------------------ | --------- | ------- | ---------------------------------------------------------------------------------------------------- |
| `path`                         | `string`  | `null`  | Absolute path to your `crafttweaker.log`. If unset, the extension searches the worktree.             |
| `additionalPath`               | `string`  | `null`  | Path to a file of extra entries. Lines starting with `<` and containing ` = ` are merged in.         |
| `alwaysReload`                 | `boolean` | `false` | Re-read the log on every completion/hover request. Cheap; recommended while the log is changing.     |
| `onlyCompleteBrackets`         | `boolean` | `true`  | Only trigger completions after `<…:`. Set to `false` to also complete unbracketed identifiers.       |
| `completionSuggestAllItems`    | `boolean` | `false` | Show every known item regardless of the typed prefix. Off by default because large modpacks lag.     |
| `completionSuggestWithStart`   | `boolean` | `false` | Only show items whose key starts with the typed prefix. Stricter than the default substring match.   |

## Limitations

### ZsLint integration

The original ZsLint VSCode extension communicated with a long-archived,
unmaintained WebSocket service (`ws://127.0.0.1:24532`, subprotocol
`zslint`) that has been dead for years. This extension does not ship a
linter.

### Minecraft format codes

Minecraft `.lang` files embed colour and style codes such as `§a`
(green), `§l` (bold), and `§r` (reset) directly inside string values.
The underlying `tree-sitter-properties` grammar tokenises the value as
a flat stream of per-character nodes, so individual format codes do not
receive dedicated captures. They inherit the value's `@string` styling
and can still be spotted visually by looking for the `§` character.

### ZSBC requires a one-off Minecraft run

The completion feature is only as good as the data the dumper produces.
The first time you set up a modpack, you must launch Minecraft once
with CraftTweaker and the bundled dumper script installed; subsequent
edits in Zed are then fully offline.

### Preprocessor directives

CraftTweaker supports preprocessor directives like `#priority`,
`#loader`, `#modloaded`, `#sideonly`, and `#ikwid` at the top of files.
The tree-sitter grammar tokenises these as comments due to a regex
precedence limitation. The semantics (which scripts load first, which
client vs. server side) are unaffected; the only impact is cosmetic
highlighting.

## License

MIT — see [`LICENSE`](LICENSE).

Third-party:

- `Copernicium282/tree-sitter-zenscript` — MIT (fork of upstream
  `ikexing-cn`), with our additions under the same MIT license as
  this extension.
- `dumpers/dumper_*.zs` — MIT, from
  `Blue-Beaker/zenscript-bracket-completion`.
- `tree-sitter-properties` — MIT.
- The `lsp-server` and `lsp-types` crates used by `zsbc-lsp` are MIT.

## Tested on

The grammar and LSP have been validated against:

- The full
  [Nuclear Tech: Reborn modpack](https://www.curseforge.com/minecraft/modpacks/nuclear-tech-reborn/files/7442534)
  — 32 `.zs` files, ~70 KB, mix of CraftTweaker and NuclearCraft
  recipes, JEI integration, and complex expressions — all parse
  with **zero ERROR nodes** against the fork.
- Four additional sanitized real-world fixture scripts covering
  recipe remapping, waste processing, fluid mechanics, and alloy
  crafting — all parse with **zero ERROR nodes**.
- A 30-entry ZSBC dumper block, exercising every `parse_ct_log`
  branch (last-block wins, missing block, additional list merges).

Result: grammar is fit for real CraftTweaker code; ZSBC server reads
real dumper output.
