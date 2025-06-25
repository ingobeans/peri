use std::io::{stdin, stdout, Read, Write};

use crossterm::{
    cursor::MoveTo,
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind,
    },
    execute, queue,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear},
};
mod gradient;
use gradient::Gradient;

fn parse_elements() -> [Element; 118] {
    // theres 118 lines
    let mut data = include_str!("elements.txt").split('\n');
    let elements: [Element; 118] = core::array::from_fn(|_| {
        let entry = data.next().expect("elements.txt isnt 118 entries");
        let properties: Vec<&str> = entry.split(',').collect();
        let number = properties[0].parse().unwrap();

        let period;
        let group;
        if (57..=71).contains(&number) {
            group = number as u16 - 57 + 4;
            period = 8;
        } else if (89..=103).contains(&number) {
            group = number as u16 - 89 + 4;
            period = 9;
        } else {
            group = properties[8].parse().unwrap();
            period = properties[7].parse().unwrap();
        }

        Element {
            name: properties[1],
            symbol: properties[2],
            number,
            mass: properties[3].parse().unwrap(),
            _metal: properties[12] == "yes",
            electronegativity: properties[17].parse().ok(),
            period,
            group,
            ty: properties[15],
        }
    });
    elements
}

#[derive(Clone, Copy, Debug)]
struct Element {
    name: &'static str,
    symbol: &'static str,
    number: u8,
    mass: f32,
    electronegativity: Option<f32>,
    _metal: bool,
    period: u16,
    group: u16,
    ty: &'static str,
}

/// Draws a square at a position
fn draw_square(x: u16, y: u16, scaling: u16, foreground_color: Color, background_color: Color) {
    let mut stdout = stdout();
    queue!(
        stdout,
        SetBackgroundColor(background_color),
        SetForegroundColor(foreground_color)
    )
    .unwrap();
    for i in 0..scaling / 2 {
        queue!(stdout, MoveTo(x, y + i)).unwrap();
        print!("{}│", " ".repeat(scaling as usize - 1));
    }
    queue!(stdout, MoveTo(x, y + scaling / 2 - 1)).unwrap();
    print!("{}┘", "─".repeat(scaling as usize - 1));
}

/// Print details about selected element
fn draw_selected_info(element: Element, scaling: u16) {
    let x = 4 * scaling;
    let y = scaling / 2;
    let mut stdout = stdout();

    let texts = [
        format!("{} - {}", element.number, element.symbol),
        element.name.to_string(),
        element.mass.to_string(),
        element.ty.to_string(),
        element
            .electronegativity
            .map_or(String::new(), |f| f.to_string()),
    ];
    let max_horizontal_space = 12 * scaling - x;

    for (index, text) in texts.into_iter().enumerate() {
        queue!(stdout, MoveTo(x, y + index as u16)).unwrap();
        // draw new text, padded with whitespace at end
        // to overwrite previous entry (in case it was longer)
        print!(
            "{}{}",
            text,
            " ".repeat(max_horizontal_space as usize - text.len())
        );
    }
}

enum ColoringMode {
    /// Elements aren't colored
    None,
    /// Elements are colored based on their type
    TypeBased,
    /// Elements are colored based on their electronegativity
    ElectronegativityBased,
}

struct Peri {
    elements: [Element; 118],
    selection_index: Option<usize>,
    coloring_mode: ColoringMode,
}
impl Peri {
    fn find_element_by_symbol(&self, symbol: String) -> Option<Element> {
        let symbol = symbol.to_lowercase();
        self.elements
            .into_iter()
            .find(|&element| element.symbol.to_lowercase() == symbol)
    }
    fn find_element_by_pos(&self, x: u16, y: u16) -> Option<Element> {
        self.elements
            .into_iter()
            .find(|&element| element.group == x && element.period == y)
    }
    /// Returns which color an element should be according to coloring mode
    /// Returns (foreground_color, background_color)
    fn get_color(&self, element: &Element) -> (Color, Color) {
        match self.coloring_mode {
            ColoringMode::None => (Color::Reset, Color::Reset),
            ColoringMode::ElectronegativityBased => {
                let Some(electronegativity) = element.electronegativity else {
                    return (Color::Reset, Color::Reset);
                };
                let gradient = Gradient::new(
                    vec![
                        (0.7, [214, 251, 221]),
                        (1.9, [5, 246, 41]),
                        (2.2, [255, 232, 77]),
                        (4.0, [255, 6, 0]),
                    ],
                    false,
                );
                let rgb = gradient.get_at(electronegativity);
                (
                    Color::Black,
                    Color::Rgb {
                        r: rgb[0],
                        g: rgb[1],
                        b: rgb[2],
                    },
                )
            }
            ColoringMode::TypeBased => match element.ty {
                "Nonmetal" => (Color::Black, Color::Green),
                "Noble Gas" => (Color::Black, Color::Magenta),
                "Transition Metal" => (Color::Black, Color::Red),
                "Metal" => (Color::Black, Color::Blue),
                "Metalloid" => (
                    Color::Black,
                    Color::Rgb {
                        r: 141,
                        g: 234,
                        b: 216,
                    },
                ),
                "Halogen" => (Color::Black, Color::White),
                "Alkali Metal" => (
                    Color::Black,
                    Color::Rgb {
                        r: 228,
                        g: 145,
                        b: 95,
                    },
                ),
                "Alkaline Earth Metal" => (Color::Black, Color::Yellow),
                _ => (Color::Reset, Color::Reset),
            },
        }
    }
    fn draw_element_square(
        &self,
        element: &Element,
        highlight_color: Option<Color>,
        scale_factor: u16,
    ) {
        let x = element.group * scale_factor;
        let mut y = scale_factor / 2 * element.period; // divided by two because we multiply by two earlier

        // add a gap
        if element.period >= 8 {
            y = scale_factor / 2 * (element.period + 1);
        }
        let (foreground_color, background_color) = self.get_color(element);
        if scale_factor > 3 {
            draw_square(
                x,
                y,
                scale_factor,
                highlight_color.unwrap_or(Color::DarkGrey),
                background_color,
            );
        }
        queue!(
            stdout(),
            MoveTo(x, y),
            SetForegroundColor(highlight_color.unwrap_or(foreground_color))
        )
        .unwrap();
        print!("{}", element.symbol);
    }
    fn draw(&self) {
        let mut stdout = stdout();
        queue!(
            stdout,
            ResetColor,
            Clear(crossterm::terminal::ClearType::All)
        )
        .unwrap();

        let scale_factor = Peri::get_scale_factor();

        for element in self.elements {
            self.draw_element_square(&element, None, scale_factor);
        }

        self.handle_selection(scale_factor);
        Self::reset_cursor(scale_factor);
        stdout.flush().unwrap();
    }
    fn handle_selection(&self, scale_factor: u16) {
        queue!(stdout(), ResetColor).unwrap();
        if let Some(selection_index) = self.selection_index {
            let selection = self.elements[selection_index];
            // draw details about selected element
            draw_selected_info(selection, scale_factor);
            // draw border around selection
            self.draw_element_square(&selection, Some(Color::Blue), scale_factor);
        }
    }
    fn get_scale_factor() -> u16 {
        let (mut width, height) = size().unwrap();
        width -= 3;

        // width scale factor is screen width / 18 because 18 groups
        let width_scale_factor = width / 18;

        // height scale is screen height / 10 because 10 rows,
        // and multiplied by two because a character on a terminal is taller than it is wide
        // and this is so that squares are mostly even
        let height_scale_factor = height / 10 * 2;

        // use the smaller of the two factors as scale factor

        width_scale_factor.min(height_scale_factor)
    }
    fn reset_cursor(scale_factor: u16) {
        queue!(stdout(), MoveTo(0, scale_factor / 2 * 10), ResetColor).unwrap();
    }
    fn interactive(&mut self) {
        execute!(stdout(), EnableMouseCapture).unwrap();
        enable_raw_mode().unwrap();
        self.draw();
        loop {
            let event = crossterm::event::read().unwrap();
            let scale_factor = Peri::get_scale_factor();
            match event {
                Event::Key(key_event) => {
                    if !key_event.kind.is_press() {
                        continue;
                    }
                    match key_event.code {
                        KeyCode::Esc => {
                            self.selection_index = None;
                            self.draw();
                        }
                        KeyCode::Right => {
                            if self.selection_index.is_none() {
                                self.selection_index = Some(0);
                            } else {
                                // redraw old selected element to remove highlight
                                let old = self.elements[self.selection_index.unwrap()];
                                self.draw_element_square(&old, None, scale_factor);

                                self.selection_index =
                                    Some(self.selection_index.unwrap_or_default() + 1);
                                if self.selection_index.unwrap() >= 118 {
                                    self.selection_index = Some(0);
                                }
                            }
                            // highlight new selection
                            self.handle_selection(scale_factor);
                            stdout().flush().unwrap();
                        }
                        KeyCode::Left => {
                            if self.selection_index.is_none() {
                                self.selection_index = Some(0);
                            } else {
                                // redraw old selected element to remove highlight
                                let old = self.elements[self.selection_index.unwrap()];
                                self.draw_element_square(&old, None, scale_factor);

                                self.selection_index = Some(
                                    self.selection_index.unwrap_or_default().saturating_sub(1),
                                );
                            }
                            // highlight new selection
                            self.handle_selection(scale_factor);
                            stdout().flush().unwrap();
                        }
                        KeyCode::Down => {
                            if self.selection_index.is_none() {
                                self.selection_index = Some(0);
                            } else {
                                let old = self.elements[self.selection_index.unwrap()];

                                let target_element =
                                    self.find_element_by_pos(old.group, old.period + 1);
                                let Some(target_element) = target_element else {
                                    continue;
                                };
                                // redraw old selected element to remove highlight
                                self.draw_element_square(&old, None, scale_factor);

                                self.selection_index = Some(target_element.number as usize - 1);
                            }

                            // highlight new selection
                            self.handle_selection(scale_factor);
                            stdout().flush().unwrap();
                        }
                        KeyCode::Up => {
                            if self.selection_index.is_none() {
                                self.selection_index = Some(0);
                            } else {
                                let old = self.elements[self.selection_index.unwrap()];

                                let target_element =
                                    self.find_element_by_pos(old.group, old.period - 1);
                                let Some(target_element) = target_element else {
                                    continue;
                                };
                                // redraw old selected element to remove highlight
                                self.draw_element_square(&old, None, scale_factor);

                                self.selection_index = Some(target_element.number as usize - 1);
                            }
                            // highlight new selection
                            self.handle_selection(scale_factor);
                            stdout().flush().unwrap();
                        }
                        KeyCode::Char(char) => {
                            // quit if ctrl+c or Q
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) && char == 'c' {
                                break;
                            }
                            if char == 'q' {
                                break;
                            }
                            // make c open coloring settings prompt
                            if char == 'c' {
                                Self::reset_cursor(scale_factor);
                                print!(
                                "enter new coloring mode, [N]one, [E]lectronegativity or [T]ype: "
                            );
                                stdout().flush().unwrap();
                                let mut buf: [u8; 1] = [0];
                                stdin().read_exact(&mut buf).unwrap();
                                match buf[0] {
                                    b'n' => {
                                        self.coloring_mode = ColoringMode::None;
                                    }
                                    b'e' => {
                                        self.coloring_mode = ColoringMode::ElectronegativityBased;
                                    }
                                    b't' => {
                                        self.coloring_mode = ColoringMode::TypeBased;
                                    }
                                    _ => {}
                                }
                                self.draw();
                            }
                            // make s open search/select prompt
                            else if char == 's' {
                                Self::reset_cursor(scale_factor);
                                print!("select (symbol): ");
                                stdout().flush().unwrap();
                                let mut buf = String::new();
                                disable_raw_mode().unwrap();
                                stdin().read_line(&mut buf).unwrap();
                                enable_raw_mode().unwrap();
                                buf = buf.trim().to_string();
                                let result = self.find_element_by_symbol(buf);
                                if let Some(element) = result {
                                    self.selection_index = Some(element.number as usize - 1);
                                }
                                self.draw();
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse_event) => {
                    if let MouseEventKind::Down(_) = mouse_event.kind {
                        let mut x = mouse_event.column;
                        let mut y = mouse_event.row;

                        if scale_factor > 3 {
                            y += 1;
                        }

                        x /= scale_factor;
                        y /= scale_factor / 2;

                        if y >= 8 {
                            y -= 1;
                        }

                        let selected = self.find_element_by_pos(x, y);
                        if let Some(selected) = selected {
                            if let Some(old_selection_index) = self.selection_index {
                                // redraw old selected element to remove highlight
                                let old = self.elements[old_selection_index];
                                self.draw_element_square(&old, None, scale_factor);
                            }

                            self.selection_index = Some(selected.number as usize - 1);
                            // highlight new selection
                            self.handle_selection(scale_factor);
                            stdout().flush().unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
        Self::reset_cursor(Self::get_scale_factor());
        execute!(stdout(), DisableMouseCapture).unwrap();
        disable_raw_mode().unwrap();
    }
}

fn main() {
    let mut peri = Peri {
        elements: parse_elements(),
        selection_index: None,
        coloring_mode: ColoringMode::None,
    };
    peri.interactive();
}
