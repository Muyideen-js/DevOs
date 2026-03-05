use std::path::PathBuf;
use std::process::Child;
use std::sync::{Arc, Mutex};

use crate::models::*;

// ── Top-Level App State ────────────────────────────────────────

pub struct AppState {
    pub project: ProjectState,
    pub explorer: ExplorerState,
    pub editor: EditorState,
    pub terminal: TerminalState,
    pub git: GitState,
    pub chat: ChatState,
    pub patch: PatchState,
    pub settings: SettingsState,
    pub right_tab: RightTab,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            project: ProjectState::default(),
            explorer: ExplorerState::default(),
            editor: EditorState::default(),
            terminal: TerminalState::default(),
            git: GitState::default(),
            chat: ChatState::default(),
            patch: PatchState::default(),
            settings: SettingsState::default(),
            right_tab: RightTab::default(),
        }
    }
}

// ── Project ────────────────────────────────────────────────────

#[derive(Default)]
pub struct ProjectState {
    pub root: Option<PathBuf>,
    pub recent: Vec<PathBuf>,
}

// ── Explorer ───────────────────────────────────────────────────

#[derive(Default)]
pub struct ExplorerState {
    pub tree: Vec<FileNode>,
}

// ── Editor ─────────────────────────────────────────────────────

#[derive(Default)]
pub struct EditorState {
    pub tabs: Vec<EditorTab>,
    pub active_tab: Option<usize>,
    pub show_search: bool,
    pub search_query: String,
}

impl EditorState {
    pub fn open_file(&mut self, path: PathBuf, content: String) {
        // Check if already open
        if let Some(idx) = self.tabs.iter().position(|t| t.path == path) {
            self.active_tab = Some(idx);
            return;
        }
        let label = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string());
        self.tabs.push(EditorTab {
            path,
            label,
            original: content.clone(),
            content,
            scroll_offset: 0.0,
        });
        self.active_tab = Some(self.tabs.len() - 1);
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.tabs.remove(idx);
            if self.tabs.is_empty() {
                self.active_tab = None;
            } else if let Some(active) = self.active_tab {
                if active >= self.tabs.len() {
                    self.active_tab = Some(self.tabs.len() - 1);
                }
            }
        }
    }
}

// ── Terminal ───────────────────────────────────────────────────

pub struct TerminalState {
    pub input: String,
    pub entries: Vec<TerminalEntry>,
    pub command_history: Vec<String>,
    pub history_index: Option<usize>,
    pub running_child: Option<Arc<Mutex<Child>>>,
    pub show: bool,
    pub scroll_to_bottom: bool,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            input: String::new(),
            entries: Vec::new(),
            command_history: Vec::new(),
            history_index: None,
            running_child: None,
            show: true,
            scroll_to_bottom: false,
        }
    }
}

// ── Git ────────────────────────────────────────────────────────

#[derive(Default)]
pub struct GitState {
    pub is_repo: bool,
    pub branch: String,
    pub files: Vec<GitFileStatus>,
    pub commit_message: String,
    pub last_error: Option<String>,
}

// ── Chat ───────────────────────────────────────────────────────

#[derive(Default)]
pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub context_files: Vec<(PathBuf, bool)>, // (path, selected)
    pub include_terminal_output: bool,
    pub waiting_for_response: bool,
    pub last_error: Option<String>,
}

// ── Patch ──────────────────────────────────────────────────────

#[derive(Default)]
pub struct PatchState {
    pub patches: Vec<FilePatch>,
    pub actions: Vec<PatchAction>,
    pub show_preview: bool,
}

// ── Settings ───────────────────────────────────────────────────

pub struct SettingsState {
    pub ollama_endpoint: String,
    pub ollama_model: String,
    pub show_settings: bool,
    pub connection_ok: Option<bool>,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            ollama_endpoint: "http://localhost:11434".to_string(),
            ollama_model: "llama3.1:8b".to_string(),
            show_settings: false,
            connection_ok: None,
        }
    }
}
