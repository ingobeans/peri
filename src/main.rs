use std::io::{stdout, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, SetBackgroundColor, SetForegroundColor},
    terminal::{size, Clear},
};

fn parse_elements() -> [Element; 118] {
    // theres 118 lines
    let mut data = include_str!("elements.txt").split('\n');
    let elements: [Element; 118] = core::array::from_fn(|_| {
        let entry = data.next().expect("elements.txt isnt 118 entries");
        let properties: Vec<&str> = entry.split(',').collect();

        Element {
            name: properties[1],
            symbol: properties[2],
            number: properties[0].parse().unwrap(),
            mass: properties[3].parse().unwrap(),
            metal: properties[12] == "yes",
            electronegativity: properties[17].parse().ok(),
            period: properties[7].parse().unwrap(),
            group: properties[8].parse().ok(),
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
    group: Option<u16>,
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

struct Peri {
    elements: [Element; 118],
}
impl Peri {
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

        for element in self.elements {
            if let Some(group) = element.group {
                let x = group * scale_factor;
                let y = element.period * scale_factor / 2; // divided by two because we multiply by two earlier
                draw_square(x, y, scale_factor, Color::Reset);
                queue!(stdout(), MoveTo(x, y), SetForegroundColor(Color::Reset)).unwrap();
                print!("{}", element.symbol);
                stdout().flush().unwrap();
                //std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}

fn main() {
    let peri = Peri {
        elements: parse_elements(),
    };
    peri.draw();
}
