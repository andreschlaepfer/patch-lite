/// Undo/redo stack with edit debouncing.
///
/// Groups consecutive edits of the same kind so that undoing
/// rolls back a full "word" or "deletion run" rather than a single character.

/// The kind of edit that was just performed (used for debouncing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    Insert,
    InsertBreak, // word boundary (space/punct) — forces a snapshot
    Delete,
    Paste,
    Enter,
}

pub struct UndoStack {
    undo: Vec<String>,
    redo: Vec<String>,
    last_edit: Option<EditKind>,
    /// Whether the current text has been snapshotted already.
    dirty: bool,
    max_history: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            last_edit: None,
            dirty: false,
            max_history: 200,
        }
    }

    /// Call this **before** applying an edit action.
    /// `current_text` is the editor content before the edit.
    /// `kind` is the category of the edit about to happen.
    pub fn on_before_edit(&mut self, current_text: &str, kind: EditKind) {
        let should_snapshot = match (self.last_edit, kind) {
            // First edit ever or after an undo/redo → always snapshot
            (None, _) => true,
            // Edit kind changed (e.g. typing → deleting) → snapshot
            (Some(prev), cur) if prev != cur => true,
            // Word boundary while inserting → snapshot
            (Some(EditKind::Insert), EditKind::InsertBreak) => true,
            // Paste always snapshots
            (_, EditKind::Paste) => true,
            // Enter always snapshots
            (_, EditKind::Enter) => true,
            // Same edit kind continues → no snapshot (group them)
            _ => false,
        };

        if should_snapshot {
            self.push_undo(current_text.to_owned());
        }

        // Normalize InsertBreak back to Insert for next comparison
        self.last_edit = Some(match kind {
            EditKind::InsertBreak => EditKind::Insert,
            other => other,
        });
        self.dirty = true;

        // Any new edit clears the redo stack
        self.redo.clear();
    }

    /// Undo: returns the text to restore, if any.
    /// `current_text` is the editor content right now.
    pub fn undo(&mut self, current_text: &str) -> Option<&str> {
        if self.undo.is_empty() {
            return None;
        }
        // Save current state to redo
        self.redo.push(current_text.to_owned());
        self.last_edit = None;
        self.dirty = false;
        self.undo.last().map(|s| s.as_str())
    }

    /// Actually pop the undo entry (call after restoring).
    pub fn pop_undo(&mut self) {
        self.undo.pop();
    }

    /// Redo: returns the text to restore, if any.
    /// `current_text` is the editor content right now.
    pub fn redo(&mut self, current_text: &str) -> Option<&str> {
        if self.redo.is_empty() {
            return None;
        }
        // Save current state to undo
        self.push_undo(current_text.to_owned());
        self.last_edit = None;
        self.dirty = false;
        self.redo.last().map(|s| s.as_str())
    }

    /// Actually pop the redo entry (call after restoring).
    pub fn pop_redo(&mut self) {
        self.redo.pop();
    }

    /// Reset the stack (e.g. when loading a saved request).
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.last_edit = None;
        self.dirty = false;
    }

    fn push_undo(&mut self, text: String) {
        if self.undo.len() >= self.max_history {
            self.undo.remove(0);
        }
        self.undo.push(text);
    }
}
