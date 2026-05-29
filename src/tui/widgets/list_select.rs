//! Generic scrollable list selector widget

pub struct ListSelect<T> {
    pub items: Vec<T>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub visible_height: usize,
}

impl<T> ListSelect<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            selected: 0,
            scroll_offset: 0,
            visible_height: 10,
        }
    }

    pub fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
            self.auto_scroll();
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.auto_scroll();
    }

    fn auto_scroll(&mut self) {
        if self.selected >= self.scroll_offset + self.visible_height {
            self.scroll_offset = self.selected - self.visible_height + 1;
        } else if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.items.get(self.selected)
    }
}
