use crate::ToDo;
use crate::db;
use crate::event::{AppEvent, Event, EventHandler};
use crate::ui::{AddTodoWidget, InputFocus, TodoWidget};
use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::widgets::{Block, BorderType, Row, Table, TableState};
use rusqlite::Connection;

#[derive(Debug)]
pub struct TodoList<'a> {
    title: String,
    items: Vec<ToDo>,
    table: Table<'a>,
    num_items: usize,
    tablestate: TableState,
    conn: Connection,
    active: bool,
}

impl TodoList<'_> {
    fn new(title: String, conn: Connection) -> Self {
        let items = db::get_todo_list_from_db(&conn, &"%".to_string());
        let num_items = items.len();
        let table = TodoList::create_table(title.clone(), items.clone(), false);
        Self {
            title: title,
            items: items,
            table: table,
            num_items: num_items,
            tablestate: TableState::default(),
            conn: conn,
            active: false,
        }
    }
    fn update(&mut self) {
        let items = db::get_todo_list_from_db(&self.conn, &"%".to_string());
        self.num_items = items.len();
        self.items = items.clone();
        self.table = TodoList::create_table(self.title.clone(), items, self.active);
    }

    fn create_table<'a>(title: String, items: Vec<ToDo>, active: bool) -> Table<'a> {
        let block = if active {
            Block::bordered()
                .border_style(Style::new().light_cyan())
                .fg(Color::LightCyan)
        } else {
            Block::bordered().border_style(Style::new().gray())
        }
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
        let rows: Vec<Row> = items
            .iter()
            .map(|i| {
                Row::new(vec![
                    i.id.to_string(),
                    i.name.to_string(),
                    i.created.to_string(),
                    i.due.to_string(),
                    i.report_to.to_string(),
                ])
            })
            .collect();

        let widths = [
            Constraint::Percentage(5),  // id
            Constraint::Percentage(25), // name
            Constraint::Percentage(25), // created
            Constraint::Percentage(25), // due
            Constraint::Percentage(15), // report_to
        ];

        Table::new(rows, widths)
            .block(block)
            .header(
                Row::new(vec!["id", "name", "created", "due", "report"]).style(Style::new().bold()),
            )
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol('>'.to_string())
    }

    // Select the next item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn next(&mut self) {
        let i = match self.tablestate.selected() {
            Some(i) => {
                if i >= self.num_items - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.tablestate.select(Some(i));
    }

    // Select the previous item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn previous(&mut self) {
        let i = match self.tablestate.selected() {
            Some(i) => {
                if i == 0 {
                    self.num_items - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.tablestate.select(Some(i));
    }

    // Unselect the currently selected item if any. The implementation of `ListState` makes
    // sure that the stored offset is also reset.
    // pub fn unselect(&mut self) {
    //     self.tablestate.select(None);
    // }
}

/// Handle what screen is being displayed
#[derive(Debug, PartialEq, Eq)]
pub enum AppScreenState {
    /// Display Screen
    Display,
    /// Add Screen
    Add,
}

#[derive(Debug)]
struct DisplayPanel<'a> {
    todo_list: TodoList<'a>,
    completed_list: TodoList<'a>,
}

impl<'a> Default for DisplayPanel<'a> {
    fn default() -> Self {
        let mut tmp = Self {
            todo_list: TodoList::new("ToDo".to_string(), db::get_todo_db_conn()),
            completed_list: TodoList::new("Completed".to_string(), db::get_completed_db_conn()),
        };
        tmp.todo_list.active = true;
        tmp
    }
}

impl DisplayPanel<'_> {
    fn new() -> Self {
        Self::default()
    }

    fn toggle_active_panel(&mut self) {
        self.todo_list.active = !self.todo_list.active;
        self.completed_list.active = !self.completed_list.active;
    }

    fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ]);
        let [completed, todo, view] = frame.area().layout(&layout);

        frame.render_stateful_widget(
            &self.completed_list.table,
            completed,
            &mut self.completed_list.tablestate,
        );

        frame.render_stateful_widget(&self.todo_list.table, todo, &mut self.todo_list.tablestate);

        let mut todo_view_widget = TodoWidget::new();
        todo_view_widget.todo = if self.todo_list.active && self.todo_list.num_items > 0 {
            self.todo_list.items[self.todo_list.tablestate.selected().unwrap_or_default()].clone()
        } else if self.completed_list.active && self.completed_list.num_items > 0 {
            self.completed_list.items[self
                .completed_list
                .tablestate
                .selected()
                .unwrap_or_default()]
            .clone()
        } else {
            ToDo {
                id: -1,
                name: "Your todo here!".into(),
                created: Local::now().date_naive().into(),
                due: Local::now().date_naive().into(),
                report_to: "Greg Davies".into(),
            }
        };
        frame.render_widget(&todo_view_widget, view);
    }

    pub fn tick(&mut self) {
        self.todo_list.update();
        self.completed_list.update();
    }
}

/// Application.
#[derive(Debug)]
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,

    /// Which screen to be displayed
    pub screen: AppScreenState,

    /// hold display panel
    display: DisplayPanel<'a>,

    /// hold add screen,
    add_screen: AddTodoWidget,
    /// Event handler.
    pub events: EventHandler,
}

impl Default for App<'_> {
    fn default() -> Self {
        let events = EventHandler::new();
        let display = DisplayPanel::new();
        let add_screen = AddTodoWidget::new();

        Self {
            running: true,
            screen: AppScreenState::Display,
            display: display,
            add_screen: add_screen,
            events: events,
        }
    }
}

impl App<'_> {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            match self.screen {
                AppScreenState::Display => {
                    self.display.todo_list.update();
                    terminal.draw(|frame| self.display.render(frame))?;
                }
                AppScreenState::Add => {
                    terminal.draw(|frame| self.add_screen.render(frame))?;
                }
            }
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == crossterm::event::KeyEventKind::Press =>
                    {
                        if self.screen == AppScreenState::Add {
                            match key_event.code {
                                KeyCode::Enter => {
                                    self.add_screen.submit_form();
                                    self.screen = AppScreenState::Display;
                                }
                                KeyCode::Char(to_insert) => match self.add_screen.todo.focus {
                                    InputFocus::Name => {
                                        self.add_screen.todo.name.enter_char(to_insert)
                                    }
                                    InputFocus::Day => {
                                        self.add_screen.todo.due_day.enter_char(to_insert)
                                    }
                                    InputFocus::Month => {
                                        self.add_screen.todo.due_month.enter_char(to_insert)
                                    }
                                    InputFocus::Year => {
                                        self.add_screen.todo.due_year.enter_char(to_insert)
                                    }
                                    InputFocus::Report => {
                                        self.add_screen.todo.report_to.enter_char(to_insert)
                                    }
                                },
                                KeyCode::Backspace => match self.add_screen.todo.focus {
                                    InputFocus::Name => self.add_screen.todo.name.delete_char(),
                                    InputFocus::Day => self.add_screen.todo.due_day.delete_char(),
                                    InputFocus::Month => {
                                        self.add_screen.todo.due_month.delete_char()
                                    }
                                    InputFocus::Year => self.add_screen.todo.due_year.delete_char(),
                                    InputFocus::Report => {
                                        self.add_screen.todo.report_to.delete_char()
                                    }
                                },
                                KeyCode::Left => match self.add_screen.todo.focus {
                                    InputFocus::Name => {
                                        self.add_screen.todo.name.move_cursor_left()
                                    }
                                    InputFocus::Day => {
                                        self.add_screen.todo.due_day.move_cursor_left()
                                    }
                                    InputFocus::Month => {
                                        self.add_screen.todo.due_month.move_cursor_left()
                                    }
                                    InputFocus::Year => {
                                        self.add_screen.todo.due_year.move_cursor_left()
                                    }
                                    InputFocus::Report => {
                                        self.add_screen.todo.report_to.move_cursor_left()
                                    }
                                },
                                KeyCode::Right => match self.add_screen.todo.focus {
                                    InputFocus::Name => {
                                        self.add_screen.todo.name.move_cursor_right()
                                    }
                                    InputFocus::Day => {
                                        self.add_screen.todo.due_day.move_cursor_right()
                                    }
                                    InputFocus::Month => {
                                        self.add_screen.todo.due_month.move_cursor_right()
                                    }
                                    InputFocus::Year => {
                                        self.add_screen.todo.due_year.move_cursor_right()
                                    }
                                    InputFocus::Report => {
                                        self.add_screen.todo.report_to.move_cursor_right()
                                    }
                                },
                                KeyCode::Tab if key_event.modifiers == KeyModifiers::SHIFT => {
                                    self.add_screen.todo.cycle_focus_reverse();
                                }
                                KeyCode::Tab => {
                                    self.add_screen.todo.cycle_focus();
                                }
                                KeyCode::Esc => self.screen = AppScreenState::Display,
                                _ => {}
                            }
                        } else {
                            self.handle_key_events(key_event)?
                        }
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::TableUp => {
                        if self.display.todo_list.active {
                            self.display.todo_list.previous()
                        } else {
                            self.display.completed_list.previous()
                        }
                    }
                    AppEvent::TableDown => {
                        if self.display.todo_list.active {
                            self.display.todo_list.next()
                        } else {
                            self.display.completed_list.next()
                        }
                    }
                    AppEvent::NextWidget => self.display.toggle_active_panel(),
                    AppEvent::PreviousWidget => self.display.toggle_active_panel(),
                    AppEvent::AddTodo => {
                        self.screen = AppScreenState::Add;
                    }
                    AppEvent::CompleteTodo => {
                        if self.display.todo_list.active && self.display.todo_list.num_items > 0 {
                            db::mark_todo_complete(
                                self.display.todo_list.items
                                    [self.display.todo_list.tablestate.selected().unwrap()]
                                .id
                                .clone(),
                                &self.display.todo_list.conn,
                                &self.display.completed_list.conn,
                            );
                            self.display.todo_list.tablestate = TableState::default()
                        }
                    }
                    AppEvent::RemoveTodo => {
                        if self.display.todo_list.active {
                            match db::remove_todo_from_db(
                                self.display.todo_list.items
                                    [self.display.todo_list.tablestate.selected().unwrap()]
                                .id
                                .clone(),
                                &self.display.todo_list.conn,
                            ) {
                                _ => (),
                            }
                            self.display.todo_list.tablestate = TableState::default()
                        }
                    }
                    AppEvent::EditTodo => {
                        self.screen = AppScreenState::Add;
                        let todo_to_edit = if self.display.todo_list.active
                            && self.display.todo_list.num_items > 0
                        {
                            self.display.todo_list.items
                                [self.display.todo_list.tablestate.selected().unwrap()]
                            .clone()
                        } else if self.display.completed_list.active
                            && self.display.completed_list.num_items > 0
                        {
                            self.display.completed_list.items
                                [self.display.completed_list.tablestate.selected().unwrap()]
                            .clone()
                        } else {
                            ToDo {
                                id: -1,
                                name: "Your todo".into(),
                                created: Local::now().date_naive().into(),
                                due: Local::now().date_naive().into(),
                                report_to: "YOU!".into(),
                            }
                        };
                        self.add_screen = AddTodoWidget::from_todo(&todo_to_edit);
                    }
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Char('c' | 'C') => self.events.send(AppEvent::CompleteTodo),
            KeyCode::Char('a' | 'A') => self.events.send(AppEvent::AddTodo),
            KeyCode::Char('r' | 'R') => self.events.send(AppEvent::RemoveTodo),
            KeyCode::Char('e' | 'E') => self.events.send(AppEvent::EditTodo),
            KeyCode::Down | KeyCode::Char('j' | 'J') => self.events.send(AppEvent::TableDown),
            KeyCode::Up | KeyCode::Char('k' | 'K') => self.events.send(AppEvent::TableUp),
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.events.send(AppEvent::NextWidget),
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.events.send(AppEvent::PreviousWidget),
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {
        match self.screen {
            AppScreenState::Display => self.display.tick(),
            AppScreenState::Add => self.add_screen.tick(),
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
