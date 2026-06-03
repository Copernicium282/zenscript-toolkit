//! `zenscript-toolkit` — Zed extension entry point.
//!
//! The LSP server is implemented in the `zsbc-lsp` sibling crate. It
//! will be wired up in a follow-up commit.

use zed_extension_api as zed;

struct ZenscriptToolkitExtension;

impl zed::Extension for ZenscriptToolkitExtension {
    fn new() -> Self {
        Self
    }
}

zed::register_extension!(ZenscriptToolkitExtension);
