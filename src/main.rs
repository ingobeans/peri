use std::io::{stdout, Write};

use crossterm::{
    cursor::MoveTo,
    queue,
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
    period: u8,
    group: Option<u8>,
}

struct Peri {
    elements: [Element; 118],
}
impl Peri {
    fn draw(&self) {
        queue!(stdout(), Clear(crossterm::terminal::ClearType::All)).unwrap();
        let (mut width, height) = size().unwrap();
        width -= 3;
        for element in self.elements {
            if let Some(group) = element.group {
                let x = (group as f32 / 18.0 * width as f32) as u16;
                let y = (element.period as f32 / 7.0 * height as f32) as u16;
                queue!(stdout(), MoveTo(x, y)).unwrap();
                print!("{}", element.symbol);
                stdout().flush().unwrap();
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
    let peri = Peri {
        elements: parse_elements(),
    };
    peri.draw();
}
