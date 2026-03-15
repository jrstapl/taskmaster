use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph},
};

#[derive(Debug, Clone)]
pub struct InputForm {
    /// Current value of the input box
    pub input: String,
    pub label: String,
    /// Position of cursor in the editor area.
    pub character_index: usize,
    /// Current input mode
    pub focused: bool,
}

impl InputForm {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            label: "Input".into(),
            focused: false,
            character_index: 0,
        }
    }

    pub fn new_with_focus() -> Self {
        Self {
            input: String::new(),
            label: "Input".into(),
            focused: false,
            character_index: 0,
        }
    }
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }
    // const fn reset_cursor(&mut self) {
    //     self.character_index = 0;
    // }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]);
        let [label_area, input_area] = area.layout(&layout);

        frame.render_widget(Paragraph::new(self.label.clone()), label_area);

        let input = Paragraph::new(self.input.as_str())
            .style(if self.focused {
                Style::default()
            } else {
                Style::default().fg(Color::Yellow)
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        if self.focused {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[expect(clippy::cast_possible_truncation)]
            frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            ))
        }
    }
}
