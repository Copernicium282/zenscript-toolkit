//! `zsbc-lsp` — LSP shim for ZenScript bracket-handler completions.
//! Sibling binary to the `zenscript-toolkit` Zed extension; the extension
//! downloads this binary from GitHub releases at activation time.

mod lsp;
mod parse;

fn main() -> anyhow::Result<()> {
    lsp::run()
}
