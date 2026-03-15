use chrono::{Datelike, Local, NaiveDate};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};
use rusqlite::Connection;

use crate::ToDo;
use crate::app::App;
use crate::db;
use crate::input::InputForm;

impl Widget for &App<'_> {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("to-dui")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let text = format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                ",
        );

        let paragraph = Paragraph::new(text)
            .block(block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered();

        paragraph.render(area, buf);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum InputFocus {
    Name,
    Day,
    Month,
    Year,
    Report,
}

#[derive(Debug, Clone)]
pub struct ToDoInput {
    pub name: InputForm,
    pub due_day: InputForm,
    pub due_month: InputForm,
    pub due_year: InputForm,
    pub report_to: InputForm,
    pub focus: InputFocus,
}

impl Default for ToDoInput {
    fn default() -> Self {
        let mut name_section = InputForm::new_with_focus();
        name_section.label = "Name".into();

        let mut day_section = InputForm::new();
        day_section.label = "Due Day [day of month]".into();

        let mut month_section = InputForm::new();
        month_section.label = "Due Month [numeric]".into();

        let mut year_section = InputForm::new();
        year_section.label = "Due Year".into();

        let mut report_to_section = InputForm::new();
        report_to_section.label = "Report To".into();

        Self {
            name: name_section,
            due_day: day_section,
            due_month: month_section,
            due_year: year_section,
            report_to: report_to_section,
            focus: InputFocus::Name,
        }
    }
}

impl ToDoInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_todo(todo: &ToDo) -> Self {
        let mut tmp = Self::default();

        tmp.name.input = todo.name.clone();
        tmp.due_day.input = todo.due.day().to_string();
        tmp.due_month.input = todo.due.month().to_string();
        tmp.due_year.input = todo.due.year().to_string();
        tmp.report_to.input = todo.name.clone();
        tmp
    }
    pub fn cycle_focus(&mut self) {
        match self.focus {
            InputFocus::Name => {
                self.focus = InputFocus::Day;
                self.name.focused = false;
                self.due_day.focused = true;
            }
            InputFocus::Day => {
                self.focus = InputFocus::Month;
                self.due_day.focused = false;
                self.due_month.focused = true;
            }
            InputFocus::Month => {
                self.focus = InputFocus::Year;
                self.due_month.focused = false;
                self.due_year.focused = true;
            }
            InputFocus::Year => {
                self.focus = InputFocus::Report;
                self.due_year.focused = false;
                self.report_to.focused = true;
            }
            InputFocus::Report => {
                self.focus = InputFocus::Name;
                self.report_to.focused = false;
                self.name.focused = true;
            }
        }
    }

    pub fn cycle_focus_reverse(&mut self) {
        match self.focus {
            InputFocus::Name => {
                self.focus = InputFocus::Report;
                self.name.focused = false;
                self.report_to.focused = true;
            }
            InputFocus::Day => {
                self.focus = InputFocus::Name;
                self.due_day.focused = false;
                self.name.focused = true;
            }
            InputFocus::Month => {
                self.focus = InputFocus::Day;
                self.due_month.focused = false;
                self.due_day.focused = true;
            }
            InputFocus::Year => {
                self.focus = InputFocus::Month;
                self.due_year.focused = false;
                self.due_month.focused = true;
            }
            InputFocus::Report => {
                self.focus = InputFocus::Year;
                self.report_to.focused = false;
                self.due_year.focused = true;
            }
        }
    }
}

#[derive(Debug)]
pub struct AddTodoWidget {
    pub todo: ToDoInput,
    db_conn: Connection,
}

impl Default for AddTodoWidget {
    fn default() -> Self {
        Self {
            todo: ToDoInput::default(),
            db_conn: db::get_todo_db_conn(),
        }
    }
}

impl AddTodoWidget {
    pub fn new() -> Self {
        Self {
            todo: ToDoInput::new(),
            db_conn: db::get_todo_db_conn(),
        }
    }

    pub fn from_todo(todo: &ToDo) -> Self {
        Self {
            todo: ToDoInput::from_todo(todo),
            db_conn: db::get_todo_db_conn(),
        }
    }

    pub fn tick(&mut self) {}

    pub fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]);
        let [name, due_day, due_month, due_year, report_to] = frame.area().layout(&layout);
        self.todo.name.render(frame, name);
        self.todo.due_day.render(frame, due_day);
        self.todo.due_month.render(frame, due_month);
        self.todo.due_year.render(frame, due_year);
        self.todo.report_to.render(frame, report_to);
    }

    pub fn submit_form(&mut self) {
        let mut todo = ToDo::new();
        todo.name = self.todo.name.input.clone();
        let year = self
            .todo
            .due_year
            .input
            .parse()
            .unwrap_or(Local::now().naive_local().year());
        let month = self
            .todo
            .due_month
            .input
            .parse()
            .unwrap_or(Local::now().naive_local().month());
        let day = self
            .todo
            .due_day
            .input
            .parse()
            .unwrap_or(Local::now().naive_local().month());
        todo.due = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        todo.report_to = self.todo.report_to.input.clone();
        db::add_todo_to_db(&todo, &self.db_conn);
        *self = Self::new();
    }
}

pub struct TodoWidget {
    pub todo: ToDo,
}

impl TodoWidget {
    pub fn new() -> Self {
        Self { todo: ToDo::new() }
    }
}

impl Widget for &TodoWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_style(Style::new().gray())
            .title("TaskView")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let text = format!(
            "\
            Name: {name}\n\n\
            Created: {created}\n\n\
            Due: {due}\n\n\
            Report To: {report}\n\n

            ",
            name = self.todo.name.to_string(),
            created = self.todo.created.to_string(),
            due = self.todo.due.to_string(),
            report = self.todo.report_to.to_string(),
        );

        let paragraph = Paragraph::new(text)
            .block(block)
            .fg(Color::Cyan)
            .bg(Color::Black)
            .centered();
        paragraph.render(area, buf);
    }
}
