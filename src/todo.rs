use chrono::{Local, NaiveDate};

#[derive(Debug, Clone)]
pub struct ToDo {
    pub id: i32,
    pub name: String,
    pub created: NaiveDate,
    pub due: NaiveDate,
    pub report_to: String,
}

impl ToDo {
    pub fn new() -> Self {
        Self {
            id: -1,
            name: String::new(),
            created: Local::now().date_naive(),
            due: Local::now().date_naive(),
            report_to: String::new(),
        }
    }

    pub fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_report_to(&mut self, report_to: String) {
        self.report_to = report_to;
    }

    pub fn set_due(&mut self, due: NaiveDate) {
        self.due = due;
    }
}
