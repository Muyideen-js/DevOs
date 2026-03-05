use std::path::PathBuf;

// ── File Explorer ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub expanded: bool,
}

// ── Editor ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EditorTab {
    pub path: PathBuf,
    pub label: String,
    pub content: String,
    pub original: String,
    pub scroll_offset: f32,
}

impl EditorTab {
    pub fn is_modified(&self) -> bool {
        self.content != self.original
    }

    pub fn title(&self) -> String {
        let name = self.label.clone();
        if self.is_modified() {
            format!("*{}", name)
        } else {
            name
        }
    }
}

// ── Terminal ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TerminalEntry {
    pub command: String,
    pub output: String,
    pub is_error: bool,
    pub running: bool,
}

// ── Git ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GitFileStatus {
    pub path: String,
    pub status: GitStatus,
    pub staged: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GitStatus {
    Modified,
    New,
    Deleted,
    Renamed,
    Untracked,
}

impl std::fmt::Display for GitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitStatus::Modified => write!(f, "M"),
            GitStatus::New => write!(f, "A"),
            GitStatus::Deleted => write!(f, "D"),
            GitStatus::Renamed => write!(f, "R"),
            GitStatus::Untracked => write!(f, "?"),
        }
    }
}

// ── Chat ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

// ── Patch ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FilePatch {
    pub file_path: String,
    pub original: String,
    pub patched: String,
    pub diff_text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatchAction {
    Pending,
    Applied,
    Rejected,
}

// ── Right Panel Tab ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RightTab {
    Git,
    Chat,
    Patches,
}

impl Default for RightTab {
    fn default() -> Self {
        RightTab::Git
    }
}

