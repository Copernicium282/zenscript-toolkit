# Changelog

All notable changes to this extension are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-06-05

### Added

- Initial release.
- ZenScript language support: syntax highlighting, indent rules, and
  code-folding driven by the vendored `tree-sitter-zenscript` grammar.
- Minecraft `.lang` language support via `tree-sitter-properties`.
- ZSBC bracket-handler completions and hover, served by a Rust LSP
  server (`zsbc-lsp`) downloaded from this repo's GitHub releases on
  first activation.
- Vendored ZenScript grammar completed against the official
  `CraftTweaker/ZenScript` Java parser: full expression precedence,
  classes, constructors, lambdas, `version N;`, `has` operator.
- Release workflow: `git tag vX.Y.Z` triggers a GitHub Actions matrix
  that builds and uploads the LSP server for six target triples.
