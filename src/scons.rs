use std::collections::BTreeMap;

use superconsole::{Component, Dimensions, DrawMode, Line, Lines, SuperConsole};
use superconsole::{Span, components::DrawVertical, style::Color};

pub struct Scons {
    console: Option<SuperConsole>,
    names: BTreeMap<usize, String>,
}

impl Scons {
    pub fn new() -> Self {
        Self {
            console: SuperConsole::new(),
            names: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, id: usize, name: String) {
        self.names.insert(id, name);

        if let Some(console) = &mut self.console {
            console.render(&Root { names: &self.names }).unwrap();
        }
    }

    pub fn remove(&mut self, id: usize, res: &anyhow::Result<crate::Success>) -> String {
        let name = self.names.remove(&id).expect("id must be present");

        let line = res_line(&name, res);

        if let Some(console) = &mut self.console {
            if let Some(line) = line {
                console.emit(Lines(vec![line]));
            }

            console.render(&Root { names: &self.names }).unwrap();
        } else if let Some(line) = line {
            println!("{}", line.render());
        }

        name
    }

    pub fn finalize(self) {
        if let Some(console) = self.console {
            console.finalize(&Root { names: &self.names }).unwrap();
        }
    }
}

fn res_line(name: &str, res: &anyhow::Result<crate::Success>) -> Option<Line> {
    let (color, text) = match res {
        Ok(success) => {
            if let Some(warn) = success.warning() {
                (Color::Yellow, format!("{name} WARN: {warn}"))
            } else if success.branches().is_empty() {
                (Color::Green, format!("{name} OK"))
            } else {
                return None;
            }
        }
        Err(err) => (Color::Red, format!("{name} ERROR: {err:#}")),
    };

    Some(Line::from_iter([Span::new_colored_lossy(&text, color)]))
}

struct Root<'a> {
    names: &'a BTreeMap<usize, String>,
}

impl Component for Root<'_> {
    fn draw_unchecked(&self, dimensions: Dimensions, mode: DrawMode) -> anyhow::Result<Lines> {
        let mut vert = DrawVertical::new(dimensions);
        vert.draw(&InProgressList { names: self.names }, mode)?;
        Ok(vert.finish())
    }
}

struct InProgressList<'a> {
    names: &'a BTreeMap<usize, String>,
}

impl Component for InProgressList<'_> {
    fn draw_unchecked(&self, _dimensions: Dimensions, _mode: DrawMode) -> anyhow::Result<Lines> {
        let mut lines = Lines::new();

        for name in self.names.values().take(10) {
            lines.push(Line::sanitized(name));
        }

        Ok(lines)
    }
}
