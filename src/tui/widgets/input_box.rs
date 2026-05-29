//! Multi-mode input box widget (vim/emacs/natural)

pub struct InputBox {
    pub buffer: String,
    pub cursor_pos: usize,
    pub mode: InputBoxMode,
}

pub enum InputBoxMode {
    Insert,
    Normal,
    Command,
    Search,
}

impl InputBox {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor_pos: 0,
            mode: InputBoxMode::Insert,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.buffer.insert(self.cursor_pos, ch);
        self.cursor_pos += ch.len_utf8();
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.buffer.remove(self.cursor_pos);
        }
    }

    pub fn submit(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }
}
