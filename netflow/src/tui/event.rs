use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use super::app::Action;

pub fn handle_event(event: Event) -> Action {
    match event {
        Event::Key(key) => handle_key(key),
        Event::Resize(_, _) => Action::Tick, // trigger immediate redraw
        _ => Action::None,
    }
}

fn handle_key(key: KeyEvent) -> Action {
    // Ctrl+C / Cmd+C (macOS) always quits
    if key.code == KeyCode::Char('c')
        && key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::SUPER)
    {
        return Action::Quit;
    }

    // Ctrl+D (EOF) also quits
    if key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
    }

    // Cmd+Q (macOS) quits
    if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::SUPER) {
        return Action::Quit;
    }

    // Ignore keys with Ctrl/Alt/Super modifiers (except the combos handled above)
    if key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER) {
        return Action::None;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Action::Quit,
        KeyCode::Esc => Action::ToggleModal,
        KeyCode::Up | KeyCode::Char('k') => Action::Up,
        KeyCode::Down | KeyCode::Char('j') => Action::Down,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Char('g') => Action::Top,
        KeyCode::Char('G') => Action::Bottom,
        KeyCode::Enter => Action::ToggleModal,
        _ => Action::None,
    }
}

/// Convert an Action to its Modal-scoped equivalent.
/// Called when the modal is open to redirect navigation into the modal.
pub fn modal_action(action: Action) -> Action {
    match action {
        Action::Up => Action::ModalUp,
        Action::Down => Action::ModalDown,
        Action::PageUp => Action::ModalPageUp,
        Action::PageDown => Action::ModalPageDown,
        Action::Top => Action::ModalTop,
        Action::Bottom => Action::ModalBottom,
        Action::ToggleModal => Action::ToggleModal, // Enter/Esc toggles modal
        Action::Quit => Action::ToggleModal,         // q/Ctrl+C closes modal
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::from(code)
    }

    fn key_ctrl(code: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(code), KeyModifiers::CONTROL)
    }

    fn key_super(code: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(code), KeyModifiers::SUPER)
    }

    fn key_alt(code: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(code), KeyModifiers::ALT)
    }

    #[test]
    fn test_q_quits() {
        assert_eq!(handle_key(key(KeyCode::Char('q'))), Action::Quit);
        assert_eq!(handle_key(key(KeyCode::Char('Q'))), Action::Quit);
    }

    #[test]
    fn test_ctrl_c_quits() {
        assert_eq!(handle_key(key_ctrl('c')), Action::Quit);
    }

    #[test]
    fn test_ctrl_q_ignored() {
        assert_eq!(handle_key(key_ctrl('q')), Action::None);
    }

    #[test]
    fn test_alt_q_ignored() {
        assert_eq!(handle_key(key_alt('q')), Action::None);
    }

    #[test]
    fn test_navigation_keys() {
        assert_eq!(handle_key(key(KeyCode::Down)), Action::Down);
        assert_eq!(handle_key(key(KeyCode::Up)), Action::Up);
        assert_eq!(handle_key(key(KeyCode::PageDown)), Action::PageDown);
        assert_eq!(handle_key(key(KeyCode::PageUp)), Action::PageUp);
        assert_eq!(handle_key(key(KeyCode::Enter)), Action::ToggleModal);
        assert_eq!(handle_key(key(KeyCode::Esc)), Action::ToggleModal);
    }

    #[test]
    fn test_vim_keys() {
        assert_eq!(handle_key(key(KeyCode::Char('j'))), Action::Down);
        assert_eq!(handle_key(key(KeyCode::Char('k'))), Action::Up);
        assert_eq!(handle_key(key(KeyCode::Char('g'))), Action::Top);
        assert_eq!(handle_key(key(KeyCode::Char('G'))), Action::Bottom);
    }

    #[test]
    fn test_resize_triggers_tick() {
        assert_eq!(handle_event(Event::Resize(80, 24)), Action::Tick);
    }

    #[test]
    fn test_super_c_quits() {
        assert_eq!(handle_key(key_super('c')), Action::Quit);
    }

    #[test]
    fn test_super_q_quits() {
        assert_eq!(handle_key(key_super('q')), Action::Quit);
    }

    #[test]
    fn test_ctrl_d_quits() {
        assert_eq!(handle_key(key_ctrl('d')), Action::Quit);
    }

    #[test]
    fn test_modal_action_redirect() {
        assert_eq!(modal_action(Action::Down), Action::ModalDown);
        assert_eq!(modal_action(Action::Up), Action::ModalUp);
        assert_eq!(modal_action(Action::PageDown), Action::ModalPageDown);
        assert_eq!(modal_action(Action::PageUp), Action::ModalPageUp);
        assert_eq!(modal_action(Action::Top), Action::ModalTop);
        assert_eq!(modal_action(Action::Bottom), Action::ModalBottom);
        assert_eq!(modal_action(Action::ToggleModal), Action::ToggleModal);
        assert_eq!(modal_action(Action::Quit), Action::ToggleModal);
    }
}
