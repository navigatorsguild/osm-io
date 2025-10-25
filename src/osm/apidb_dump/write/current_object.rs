pub(crate) struct CurrentObjectLine {
    last_id: i64,
    last_line: Option<String>,
}

impl CurrentObjectLine {
    pub(crate) fn new() -> CurrentObjectLine {
        CurrentObjectLine {
            last_id: 0,
            last_line: None,
        }
    }

    pub(crate) fn set_last_id(&mut self, id: i64) {
        self.last_id = id;
    }

    pub(crate) fn set_last_line(&mut self, line: String, id: i64, visible: bool) -> Option<String> {
        let line_opt =
            if visible {
                self.last_line.replace(line)
            } else {
                std::mem::take(&mut self.last_line)
            };

        if id > self.last_id {
            line_opt
        } else {
            None
        }
    }

    pub(crate) fn take(&mut self) -> Option<String> {
        std::mem::take(&mut self.last_line)
    }
}

pub(crate) struct CurrentObjectLines {
    last_id: i64,
    last_lines: Option<Vec<String>>,
}

impl CurrentObjectLines {
    pub(crate) fn new() -> CurrentObjectLines {
        CurrentObjectLines {
            last_id: 0,
            last_lines: None,
        }
    }

    pub(crate) fn set_last_id(&mut self, id: i64) {
        self.last_id = id;
    }

    pub(crate) fn set_last_lines(&mut self, lines: Vec<String>, id: i64, visible: bool) -> Option<Vec<String>> {
        let lines_opt =
            if visible {
                self.last_lines.replace(lines)
            } else {
                std::mem::take(&mut self.last_lines)
            };

        if id > self.last_id {
            lines_opt
        } else {
            None
        }
    }

    pub(crate) fn take(&mut self) -> Option<Vec<String>> {
        std::mem::take(&mut self.last_lines)
    }
}
