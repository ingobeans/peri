fn lerp_1(value1: f32, value2: f32, t: f32) -> f32 {
    value1 + t * (value2 - value1)
}

type Rgb = [u8; 3];

pub struct Gradient {
    /// Repeat dictates whether the gradient should wrap around, meaning in practice that the steps of the gradient infinitely repeat
    pub repeat: bool,
    /// Steps of the gradient, each step is a tuple of (position: f32, color: Rgb)
    steps: Vec<(f32, Rgb)>,
}
impl Gradient {
    pub fn new(steps: Vec<(f32, Rgb)>, repeat: bool) -> Self {
        Self { steps, repeat }
    }
    pub fn get_at(&self, mut position: f32) -> [u8; 3] {
        if self.repeat {
            let final_step = self.steps.last().unwrap().0;
            position %= final_step;
        }

        let mut bottom_step = 0;
        for (index, (step, _value)) in self.steps.iter().enumerate() {
            if *step > position {
                break;
            }
            bottom_step = index;
        }
        if !self.repeat && bottom_step == self.steps.len() - 1 {
            return self.steps[bottom_step].1;
        }
        if bottom_step == 0 && position < self.steps[bottom_step].0 {
            return self.steps[bottom_step].1;
        }

        let value_a = self.steps[bottom_step].1;
        let value_b = self.steps[bottom_step + 1].1;

        let step_a = self.steps[bottom_step].0;
        let step_b = self.steps[bottom_step + 1].0;

        let percent = (position - step_a) / (step_b - step_a);
        [
            lerp_1(value_a[0] as f32, value_b[0] as f32, percent) as u8,
            lerp_1(value_a[1] as f32, value_b[1] as f32, percent) as u8,
            lerp_1(value_a[2] as f32, value_b[2] as f32, percent) as u8,
        ]
    }
}
