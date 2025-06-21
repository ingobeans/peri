use core::panic;
use std::io::stdout;

use crossterm::{
    cursor::MoveTo,
    event::{Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Color, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear},
};

fn parse_elements() -> [Element; 118] {
    // theres 118 lines
    let mut data = include_str!("elements.txt").split('\n');
    let elements: [Element; 118] = core::array::from_fn(|_| {
        let entry = data.next().expect("elements.txt isnt 118 entries");
        let properties: Vec<&str> = entry.split(',').collect();
        let number = properties[0].parse().unwrap();
        let period;
        let group;
        if number >= 57 && number <= 71 {
            group = number as u16 - 57 + 4;
            period = 9;
        } else if number >= 89 && number <= 103 {
            group = number as u16 - 89 + 4;
            period = 10;
        } else {
            println!("{}", number);
            group = properties[8].parse().unwrap();
            period = properties[7].parse().unwrap();
        }

        Element {
            name: properties[1],
            symbol: properties[2],
            number,
            mass: properties[3].parse().unwrap(),
            metal: properties[12] == "yes",
            electronegativity: properties[17].parse().ok(),
            period,
            group,
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
    metal: bool,
    period: u16,
    group: u16,
}

/// Draws a square at a position
fn draw_square(x: u16, y: u16, scaling: u16, color: Color) {
    let mut stdout = stdout();
    queue!(
        stdout,
        SetBackgroundColor(color),
        SetForegroundColor(Color::DarkGrey)
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
    ];
    for (index, text) in texts.into_iter().enumerate() {
        queue!(stdout, MoveTo(x, y + index as u16)).unwrap();
        print!("{}", text);
    }
}

struct Peri {
    elements: [Element; 118],
    selection_index: Option<usize>,
}
impl Peri {
    fn find_element_by_pos(&self, x: u16, y: u16) -> Option<Element> {
        for element in self.elements {
            if element.group == x && element.period == y {
                return Some(element);
            }
        }
        None
    }
    fn draw(&self) {
        queue!(stdout(), Clear(crossterm::terminal::ClearType::All)).unwrap();
        let (mut width, height) = size().unwrap();
        width -= 3;

        // calculate scale factor
        // width scale factor is screen width / 18 because 18 groups
        let width_scale_factor = width / 18;
        // height scale is screen height / 10 because 10 rows,
        // and multiplied by two because a character on a terminal is taller than it is wide
        // and this is so that squares are mostly even
        let height_scale_factor = height / 10 * 2;
        // use the smaller of the two factors as scale factor
        let scale_factor = width_scale_factor.min(height_scale_factor);

        for (index, element) in self.elements.iter().enumerate() {
            let x = element.group * scale_factor;
            let y = element.period * scale_factor / 2; // divided by two because we multiply by two earlier

            let mut selected = false;
            if let Some(selection_index) = self.selection_index {
                if index == selection_index {
                    selected = true;
                }
            }

            if scale_factor > 3 {
                let mut color = Color::Reset;
                if selected {
                    color = Color::Blue;
                }
                draw_square(x, y, scale_factor, color);
            }
            queue!(stdout(), MoveTo(x, y), SetForegroundColor(Color::Reset)).unwrap();
            print!("{}", element.symbol);
            //std::thread::sleep(std::time::Duration::from_secs(1));
        }

        if let Some(selection_index) = self.selection_index {
            draw_selected_info(self.elements[selection_index], scale_factor);
        }

        // move cursor back to bottom of screen
        execute!(stdout(), MoveTo(0, 10 * scale_factor / 2)).unwrap();
    }
    fn interactive(&mut self) {
        enable_raw_mode().unwrap();
        self.draw();
        loop {
            let event = crossterm::event::read().unwrap();
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
                                self.selection_index =
                                    Some(self.selection_index.unwrap_or_default() + 1);
                            }
                            self.draw();
                        }
                        KeyCode::Left => {
                            if self.selection_index.is_none() {
                                self.selection_index = Some(0);
                            } else {
                                self.selection_index = Some(
                                    self.selection_index.unwrap_or_default().saturating_sub(1),
                                );
                            }
                            self.draw();
                        }
                        KeyCode::Down => {
                            if self.selection_index.is_none() {
                                self.selection_index = Some(0);
                            } else {
                                let current = self.elements[self.selection_index.unwrap()];
                                let target_element =
                                    self.find_element_by_pos(current.group, current.period + 1);
                                let Some(target_element) = target_element else {
                                    continue;
                                };
                                self.selection_index = Some(target_element.number as usize - 1);
                            }
                            self.draw();
                        }
                        KeyCode::Up => {
                            if self.selection_index.is_none() {
                                self.selection_index = Some(0);
                            } else {
                                let current = self.elements[self.selection_index.unwrap()];
                                let target_element =
                                    self.find_element_by_pos(current.group, current.period - 1);
                                let Some(target_element) = target_element else {
                                    continue;
                                };

                                self.selection_index = Some(target_element.number as usize - 1);
                            }
                            self.draw();
                        }
                        KeyCode::Char(char) => {
                            // quit if ctrl+c or Q
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                                if char == 'c' {
                                    break;
                                }
                            }
                            if char == 'q' {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        disable_raw_mode().unwrap();
    }
}

fn main() {
    let mut peri = Peri {
        elements: parse_elements(),
        selection_index: None,
    };
    peri.interactive();
}
