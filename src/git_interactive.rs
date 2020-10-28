use crate::list::action::Action;
use crate::list::line::Line;
use anyhow::{anyhow, Result};
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;

pub struct GitInteractive {
	filepath: String,
	lines: Vec<Line>,
	selected_line_index: usize,
	visual_index_start: Option<usize>,
	comment_char: String,
	is_noop: bool,
}

impl GitInteractive {
	pub(crate) fn new(path: &str, comment_char: &str) -> Self {
		Self {
			filepath: path.to_string(),
			lines: vec![],
			selected_line_index: 1,
			is_noop: false,
			visual_index_start: None,
			comment_char: String::from(comment_char),
		}
	}

	pub(crate) fn set_lines(&mut self, lines: Vec<Line>) {
		self.is_noop = !lines.is_empty() && lines[0].get_action() == &Action::Noop;
		self.lines = if self.is_noop {
			vec![]
		}
		else {
			lines.into_iter().filter(|l| l.get_action() != &Action::Noop).collect()
		};
	}

	pub(crate) fn load_file(&mut self) -> Result<()> {
		let lines = read_to_string(Path::new(&self.filepath))
			.map_err(|err| anyhow!("Error reading file: {}", self.filepath).context(err))?
			.lines()
			.filter_map(|l| {
				if l.starts_with(self.comment_char.as_str()) || l.is_empty() {
					None
				}
				else {
					Some(Line::new(l).map_err(|err| anyhow!("Error reading file: {}", self.filepath).context(err)))
				}
			})
			.collect::<Result<Vec<Line>>>()?;
		self.set_lines(lines);
		Ok(())
	}

	pub(crate) fn write_file(&self) -> Result<()> {
		let mut file = File::create(&self.filepath)
			.map_err(|err| anyhow!(err).context(anyhow!("Error opening file: {}", self.filepath)))?;
		let file_contents = if self.is_noop {
			String::from("noop")
		}
		else {
			self.lines.iter().map(Line::to_text).collect::<Vec<String>>().join("\n")
		};
		writeln!(file, "{}", file_contents)
			.map_err(|err| anyhow!(err).context(anyhow!("Error writing file: {}", self.filepath)))?;
		Ok(())
	}

	pub(crate) fn clear(&mut self) {
		self.lines.clear();
	}

	pub(crate) fn set_selected_line_index(&mut self, selected_line_index: usize) {
		self.selected_line_index = selected_line_index;
	}

	pub(crate) fn set_visual_index(&mut self, visual_index: usize) {
		self.visual_index_start = Some(visual_index);
	}

	pub(crate) fn start_visual_mode(&mut self) {
		self.visual_index_start = Some(self.selected_line_index);
	}

	pub(crate) fn end_visual_mode(&mut self) {
		self.visual_index_start = None;
	}

	pub(crate) fn swap_lines(&mut self, a: usize, b: usize) {
		self.lines.swap(a, b);
	}

	pub(crate) fn edit_selected_line(&mut self, content: &str) {
		self.lines[self.selected_line_index - 1].edit_content(content);
	}

	pub(crate) fn set_range_action(&mut self, action: Action) {
		let end_index = self.visual_index_start.unwrap_or(self.selected_line_index);
		let start_index = self.selected_line_index;

		let range = if start_index <= end_index {
			start_index..=end_index
		}
		else {
			end_index..=start_index
		};

		for index in range {
			let selected_action = self.lines[index - 1].get_action();
			if *selected_action != Action::Exec && *selected_action != Action::Break {
				self.lines[index - 1].set_action(action);
			}
		}
	}

	pub(crate) fn add_line(&mut self, line_number: usize, line: Line) {
		self.lines.insert(line_number - 1, line);
	}

	pub(crate) fn remove_line(&mut self, line_number: usize) {
		self.lines.remove(line_number - 1);
	}

	pub(crate) const fn is_noop(&self) -> bool {
		self.is_noop
	}

	pub(crate) fn get_selected_line(&self) -> &Line {
		&self.lines[self.selected_line_index - 1]
	}

	pub(crate) const fn get_selected_line_index(&self) -> usize {
		self.selected_line_index
	}

	pub(crate) const fn get_visual_start_index(&self) -> Option<usize> {
		self.visual_index_start
	}

	pub(crate) fn get_filepath(&self) -> &str {
		self.filepath.as_str()
	}

	pub(crate) const fn get_lines(&self) -> &Vec<Line> {
		&self.lines
	}
}
