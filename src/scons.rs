use indexmap::IndexSet;
use superconsole::{Component, Dimensions, DrawMode, Line, Lines, SuperConsole};
use superconsole::{Span, components::DrawVertical, style::Color};

pub struct Scons {
    console: Option<SuperConsole>,
    names: IndexSet<String>,
}

impl Scons {
    pub fn new() -> Self {
        Self {
            console: SuperConsole::new(),
            names: IndexSet::new(),
        }
    }

    pub fn insert(&mut self, name: String) {
        self.names.insert(name);

        if let Some(console) = &mut self.console {
            console.render(&Root { names: &self.names }).unwrap();
        }
    }

    pub fn remove(&mut self, name: &str, res: &crate::Result) {
        self.names.shift_remove(name);

        let line = [span_res(name, res)].into_iter().collect::<Line>();

        if let Some(console) = &mut self.console {
            if !matches!(res, crate::Result::Branch(_)) {
                console.emit([line].into_iter().collect())
            }

            console.render(&Root { names: &self.names }).unwrap();
        } else {
            println!("{}", line.render());
        }
    }

    pub fn finalize(self) {
        if let Some(console) = self.console {
            console.finalize(&Root { names: &self.names }).unwrap();
        }
    }
}

fn span_res(name: &str, res: &crate::Result) -> Span {
    let color = match res {
        crate::Result::Ok => Color::Green,
        crate::Result::Warn(_) => Color::Yellow,
        crate::Result::Err(_) => Color::Red,
        crate::Result::Branch(_) => Color::Grey,
    };

    let text = match res {
        crate::Result::Ok => format!("{name} OK"),
        crate::Result::Warn(warn) => format!("{name} WARN: {warn}"),
        crate::Result::Err(err) => format!("{name} ERROR: {err:#}"),
        crate::Result::Branch(_) => String::new(),
    };

    Span::new_colored_lossy(&text, color)
}

struct Root<'a> {
    names: &'a IndexSet<String>,
}

impl Component for Root<'_> {
    fn draw_unchecked(&self, dimensions: Dimensions, mode: DrawMode) -> anyhow::Result<Lines> {
        let mut vert = DrawVertical::new(dimensions);
        vert.draw(&InProgressList { names: self.names }, mode)?;
        Ok(vert.finish())
    }
}

struct InProgressList<'a> {
    names: &'a IndexSet<String>,
}

impl Component for InProgressList<'_> {
    fn draw_unchecked(&self, _dimensions: Dimensions, _mode: DrawMode) -> anyhow::Result<Lines> {
        let mut lines = Lines::new();

        for name in self.names.iter().take(10) {
            lines.push(Line::sanitized(name));
        }

        Ok(lines)
    }
}
