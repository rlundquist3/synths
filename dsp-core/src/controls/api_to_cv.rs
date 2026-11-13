pub const INPUT_COUNT: usize = 4;
pub const OUTPUT_COUNT: usize = 4;

#[derive(Debug, Clone)]
struct IOMapping {
    value: f32,
    output_index: usize,
}

#[derive(Debug, Clone)]
pub enum APIToCVModes {
    RealTime,
    Clocked,
}

pub struct APIToCV {
    mode: APIToCVModes,
    inputs: [IOMapping; INPUT_COUNT],
    outputs: [f32; OUTPUT_COUNT],
    // TODO: data transformation functions
    // TODO: clock? might be external, just calling next on tick?
}

impl APIToCV {
    pub fn new() -> Self {
        APIToCV {
            mode: APIToCVModes::RealTime,
            inputs: core::array::from_fn(|i| IOMapping {
                value: 0.0,
                output_index: i,
            }),
            outputs: [0.0; OUTPUT_COUNT],
        }
    }

    pub fn get_mode(&self) -> APIToCVModes {
        self.mode.clone()
    }

    pub fn map_input_to_output(&mut self, input_index: usize, output_index: usize) {
        self.inputs[input_index].output_index = output_index;
    }

    pub fn receive_input(&mut self, input_index: usize, value: f32) {
        self.inputs[input_index].value += value;
    }

    pub fn realtime_output(&mut self, input_index: usize) -> [f32; OUTPUT_COUNT] {
        let value = self.inputs[input_index].value;
        self.inputs[input_index].value = 0.0;

        self.outputs[self.inputs[input_index].output_index] = value;
        let result = self.outputs.clone();
        self.outputs = [0.0; OUTPUT_COUNT];

        result
    }

    pub fn tick(&mut self) -> [f32; OUTPUT_COUNT] {
        // input values accumulate between ticks, clear out when processing

        self.inputs
            .iter()
            .for_each(|input| self.outputs[input.output_index] += input.value);

        let result = self.outputs.clone();
        self.outputs = [0.0; OUTPUT_COUNT];

        result
    }
}
