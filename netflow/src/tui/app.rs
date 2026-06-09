use crate::flow_table::FlowEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Tick,
    Up,
    Down,
    PageUp,
    PageDown,
    Top,
    Bottom,
    ToggleModal,
    ModalUp,
    ModalDown,
    ModalPageUp,
    ModalPageDown,
    ModalTop,
    ModalBottom,
    None,
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub flows: Vec<FlowEntry>,
    pub selected: usize,
    pub offset: usize,
    pub modal_open: bool,
    pub modal_scroll: usize,
    modal_max_scroll: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            flows: Vec::new(),
            selected: 0,
            offset: 0,
            modal_open: false,
            modal_scroll: 0,
            modal_max_scroll: 0,
        }
    }

    pub fn update_flows(&mut self, flows: Vec<FlowEntry>) {
        self.flows = flows;
        if self.selected >= self.flows.len() && !self.flows.is_empty() {
            self.selected = self.flows.len().saturating_sub(1);
        }
        if self.flows.is_empty() {
            self.selected = 0;
            self.offset = 0;
            self.modal_open = false;
            self.modal_scroll = 0;
        }
    }

    pub fn selected_flow(&self) -> Option<&FlowEntry> {
        self.flows.get(self.selected)
    }

    pub fn cursor_down(&mut self, view_height: usize) {
        if self.flows.is_empty() {
            return;
        }
        let max = self.flows.len().saturating_sub(1);
        if self.selected < max {
            self.selected += 1;
        }
        if self.selected >= self.offset + view_height {
            self.offset = self.selected.saturating_sub(view_height).saturating_add(1);
        }
    }

    pub fn cursor_up(&mut self, _view_height: usize) {
        if self.flows.is_empty() {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
        }
        if self.selected < self.offset {
            self.offset = self.selected;
        }
    }

    pub fn page_down(&mut self, view_height: usize) {
        if self.flows.is_empty() {
            return;
        }
        let max = self.flows.len().saturating_sub(1);
        let jump = view_height.saturating_sub(2).max(1);
        self.selected = (self.selected + jump).min(max);
        if self.selected >= self.offset + view_height {
            self.offset = self.selected.saturating_sub(view_height).saturating_add(1);
        }
    }

    pub fn page_up(&mut self, view_height: usize) {
        if self.flows.is_empty() {
            return;
        }
        let jump = view_height.saturating_sub(2).max(1);
        self.selected = self.selected.saturating_sub(jump);
        if self.selected < self.offset {
            self.offset = self.selected;
        }
    }

    pub fn cursor_top(&mut self) {
        self.selected = 0;
        self.offset = 0;
    }

    pub fn cursor_bottom(&mut self, view_height: usize) {
        if self.flows.is_empty() {
            return;
        }
        self.selected = self.flows.len().saturating_sub(1);
        self.offset = self.selected.saturating_sub(view_height).saturating_add(1);
    }

    pub fn modal_down(&mut self) {
        self.modal_scroll = (self.modal_scroll + 1).min(self.modal_max_scroll);
    }

    pub fn modal_up(&mut self) {
        self.modal_scroll = self.modal_scroll.saturating_sub(1);
    }

    pub fn modal_page_down(&mut self, page: usize) {
        self.modal_scroll = (self.modal_scroll + page).min(self.modal_max_scroll);
    }

    pub fn modal_page_up(&mut self, page: usize) {
        self.modal_scroll = self.modal_scroll.saturating_sub(page);
    }

    pub fn modal_top(&mut self) {
        self.modal_scroll = 0;
    }

    pub fn modal_bottom(&mut self) {
        self.modal_scroll = self.modal_max_scroll;
    }

    pub fn set_modal_max_scroll(&mut self, content_lines: usize) {
        // max_scroll = content_lines that can't fit in modal inner area
        // render sets this based on actual content
        self.modal_max_scroll = content_lines.saturating_sub(1);
        self.modal_scroll = self.modal_scroll.min(self.modal_max_scroll);
    }

    pub fn open_modal(&mut self) {
        if !self.flows.is_empty() {
            self.modal_open = true;
            self.modal_scroll = 0;
        }
    }

    pub fn close_modal(&mut self) {
        self.modal_open = false;
        self.modal_scroll = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow_table::FlowState;
    use netflow_common::{FlowKey, FlowStats};

    fn make_flows(n: usize) -> Vec<FlowEntry> {
        (0..n)
            .map(|i| FlowEntry {
                key: FlowKey {
                    src_ip: i as u32,
                    dst_ip: 0,
                    src_port: i as u16,
                    dst_port: 0,
                    protocol: 6,
                },
                stats: FlowStats::default(),
                state: FlowState::Active,
                created_at: std::time::Instant::now(),
                last_seen: std::time::Instant::now(),
            })
            .collect()
    }

    #[test]
    fn test_cursor_down() {
        let mut s = AppState::new();
        s.update_flows(make_flows(5));
        s.cursor_down(3);
        assert_eq!(s.selected, 1);
        s.cursor_down(3);
        assert_eq!(s.selected, 2);
        s.cursor_down(3);
        assert_eq!(s.selected, 3);
        assert_eq!(s.offset, 1); // scrolls to keep selected visible
    }

    #[test]
    fn test_cursor_up() {
        let mut s = AppState::new();
        s.update_flows(make_flows(5));
        s.selected = 3;
        s.offset = 1;
        s.cursor_up(3);
        assert_eq!(s.selected, 2);
        s.cursor_up(3);
        s.cursor_up(3);
        assert_eq!(s.selected, 0);
        assert_eq!(s.offset, 0);
    }

    #[test]
    fn test_page_down() {
        let mut s = AppState::new();
        s.update_flows(make_flows(10));
        s.page_down(4);
        assert_eq!(s.selected, 2);
        s.page_down(4);
        assert_eq!(s.selected, 4);
    }

    #[test]
    fn test_page_up() {
        let mut s = AppState::new();
        s.update_flows(make_flows(10));
        s.selected = 5;
        s.offset = 3;
        s.page_up(4);
        assert_eq!(s.selected, 3);
        assert_eq!(s.offset, 3);
    }

    #[test]
    fn test_cursor_top_bottom() {
        let mut s = AppState::new();
        s.update_flows(make_flows(10));
        s.selected = 5;
        s.offset = 3;
        s.cursor_top();
        assert_eq!(s.selected, 0);
        assert_eq!(s.offset, 0);
        s.cursor_bottom(3);
        assert_eq!(s.selected, 9);
    }

    #[test]
    fn test_empty_flows_resets_state() {
        let mut s = AppState::new();
        s.update_flows(make_flows(5));
        s.selected = 3;
        s.modal_open = true;
        s.update_flows(vec![]);
        assert_eq!(s.selected, 0);
        assert_eq!(s.offset, 0);
        assert!(!s.modal_open);
    }

    #[test]
    fn test_update_flows_clamps_selection() {
        let mut s = AppState::new();
        s.update_flows(make_flows(5));
        s.selected = 4;
        s.update_flows(make_flows(3));
        assert_eq!(s.selected, 2);
    }

    #[test]
    fn test_modal_scroll_bounded() {
        let mut s = AppState::new();
        s.update_flows(make_flows(3));
        s.open_modal();
        s.set_modal_max_scroll(5);
        s.modal_down();
        s.modal_down();
        s.modal_down();
        s.modal_down();
        assert_eq!(s.modal_scroll, 4); // clamped to max
        s.modal_bottom();
        assert_eq!(s.modal_scroll, 4);
        s.modal_top();
        assert_eq!(s.modal_scroll, 0);
    }

    #[test]
    fn test_modal_page_scroll() {
        let mut s = AppState::new();
        s.update_flows(make_flows(3));
        s.open_modal();
        s.set_modal_max_scroll(10); // max_scroll = 9
        s.modal_page_down(3);
        assert_eq!(s.modal_scroll, 3);
        s.modal_page_down(10);
        assert_eq!(s.modal_scroll, 9); // clamped to max_scroll
        s.modal_page_up(2);
        assert_eq!(s.modal_scroll, 7);
        s.modal_page_up(20);
        assert_eq!(s.modal_scroll, 0); // clamped
    }

    #[test]
    fn test_selected_flow() {
        let mut s = AppState::new();
        s.update_flows(make_flows(3));
        assert!(s.selected_flow().is_some());
        s.selected = 1;
        assert_eq!(s.selected_flow().unwrap().key.src_ip, 1);
        s.update_flows(vec![]);
        assert!(s.selected_flow().is_none());
    }
}
