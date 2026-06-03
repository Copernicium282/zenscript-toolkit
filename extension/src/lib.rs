//! `zenscript-toolkit` — Zed extension that bundles ZenScript and
//! Minecraft `.lang` language support with a ZSBC bracket-completion
//! language server.
//!
//! The ZSBC language server is **not** shipped as part of the extension
//! per the Zed extension publishing guidelines
//! (<https://zed.dev/docs/extensions/developing-extensions#extension-publishing-prerequisites>).
//! On first activation we download the appropriate prebuilt binary from
//! the GitHub releases of this repository and cache it in Zed's extension
//! working directory.

use std::path::PathBuf;

use serde::Deserialize;
use zed_extension_api::{
    self as zed, Architecture, DownloadedFileType, GithubReleaseOptions, Os, Result,
};

const RELEASE_REPO: &str = "Copernicium282/zenscript-toolkit";
const LSP_BINARY: &str = "zsbc-lsp";

/// Settings block exposed in `extension.toml` under
/// `[language_servers.zsbc.settings]`. Mirrors the fields of the original
/// VSCode extension, renamed to snake_case for Rust.
#[derive(Debug, Default, Clone, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ZsbcSettings {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    additional_path: Option<String>,
    #[serde(default)]
    always_reload: bool,
    #[serde(default)]
    only_complete_brackets: bool,
    #[serde(default)]
    completion_suggest_all_items: bool,
    #[serde(default)]
    completion_suggest_with_start: bool,
}

struct ZenscriptToolkitExtension {
    cached_binary: Option<PathBuf>,
}

impl ZenscriptToolkitExtension {
    fn new() -> Self {
        Self { cached_binary: None }
    }

    /// Resolve the Rust target triple for the current platform.
    fn target_triple() -> &'static str {
        let (os, arch) = zed::current_platform();
        match (os, arch) {
            (Os::Mac, Architecture::Aarch64) => "aarch64-apple-darwin",
            (Os::Mac, Architecture::X8664) => "x86_64-apple-darwin",
            (Os::Linux, Architecture::Aarch64) => "aarch64-unknown-linux-gnu",
            (Os::Linux, Architecture::X8664) => "x86_64-unknown-linux-gnu",
            (Os::Windows, Architecture::Aarch64) => "aarch64-pc-windows-msvc",
            (Os::Windows, Architecture::X8664) => "x86_64-pc-windows-msvc",
            _ => "unknown",
        }
    }

    /// Download the appropriate `zsbc-lsp` binary for the current platform
    /// into Zed's extension working directory, mark it executable, and cache
    /// its path for subsequent calls.
    fn download_lsp_binary(&mut self) -> Result<PathBuf> {
        if let Some(p) = &self.cached_binary {
            return Ok(p.clone());
        }

        let release = zed::latest_github_release(
            RELEASE_REPO,
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let target = Self::target_triple();
        let asset_name = format!("{LSP_BINARY}-{target}");
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "no release asset named `{asset_name}` in {RELEASE_REPO} {} (have: {})",
                    release.version,
                    release
                        .assets
                        .iter()
                        .map(|a| a.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;

        let file_type = if cfg!(windows) {
            DownloadedFileType::Uncompressed
        } else {
            DownloadedFileType::GzipTar
        };
        let destination = format!("{LSP_BINARY}-{target}");
        zed::download_file(&asset.download_url, &destination, file_type)
            .map_err(|e| format!("failed to download {asset_name}: {e}"))?;

        let binary_path = PathBuf::from(destination);
        if !cfg!(windows) {
            zed::make_file_executable(binary_path.to_string_lossy().as_ref())
                .map_err(|e| format!("failed to chmod zsbc-lsp: {e}"))?;
        }
        self.cached_binary = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for ZenscriptToolkitExtension {
    fn new() -> Self {
        Self::new()
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary = self.download_lsp_binary()?;
        Ok(zed::Command {
            command: binary.to_string_lossy().to_string(),
            args: Vec::new(),
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = read_settings_from_worktree(worktree);
        Ok(Some(serde_json::to_value(settings).unwrap_or(serde_json::Value::Null)))
    }
}

/// Read `zsbc.*` settings from the worktree's zed settings. The VSCode
/// extension stored its settings under the `zsbc.*` namespace; we look for
/// the same keys in the user's Zed settings file.
fn read_settings_from_worktree(worktree: &zed::Worktree) -> ZsbcSettings {
    let raw = match worktree.read_text_file(".zed/settings.json") {
        Ok(s) => s,
        Err(_) => return ZsbcSettings::default(),
    };
    let value: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return ZsbcSettings::default(),
    };
    serde_json::from_value(value.get("zsbc").cloned().unwrap_or(serde_json::Value::Null))
        .unwrap_or_default()
}

zed::register_extension!(ZenscriptToolkitExtension);
