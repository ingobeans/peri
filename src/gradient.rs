fn lerp_1(value1: f32, value2: f32, t: f32) -> f32 {
    value1 + t * (value2 - value1)
}

pub struct Gradient {
    steps: Vec<f32>,
    values: Vec<[u8; 3]>,
}
impl Gradient {
    pub fn new(steps: Vec<f32>, values: Vec<[u8; 3]>) -> Self {
        assert_eq!(steps.len(), values.len());
        Self { steps, values }
    }
    pub fn get_at(&self, position: f32) -> [u8; 3] {
        let mut last = 0;
        for (index, step) in self.steps.iter().enumerate() {
            if *step > position {
                break;
            }
            last = index;
        }
        if last == self.steps.len() {
            return self.values[last];
        }

        let value_a = self.values[last];
        let value_b = self.values[last + 1];
        let step_a = self.steps[last];
        let step_b = self.steps[last + 1];
        let percent = (position - step_a) / (step_b - step_a);
        [
            lerp_1(value_a[0] as f32, value_b[0] as f32, percent) as u8,
            lerp_1(value_a[1] as f32, value_b[1] as f32, percent) as u8,
            lerp_1(value_a[2] as f32, value_b[2] as f32, percent) as u8,
        ]
    }
}
