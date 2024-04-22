use hound::{WavReader, WavSpec};
use std::iter::zip;

//do
//

pub struct WaveUtil {
    contents: Vec<[f64; 2]>,
    //these represent the window we can see
    //use these for lazy loading and
    //determining skip
    pub min: [f64; 2],
    pub max: [f64; 2],
}

impl WaveUtil {
    pub fn new(input_file: String) -> Self {
        Self {
            contents: load_samples(input_file),
            min: Default::default(),
            max: Default::default(),
        }
    }

    pub fn set_range(&mut self, min: [f64; 2], max: [f64; 2]) {
        self.min = min;
        self.max = max;
    }

    pub fn get_samples(&self) -> Vec<[f64; 2]> {
        self.contents.clone()
    }
}

pub fn load_samples(path: String) -> Vec<[f64; 2]> {
    println!("opening: {:?}", path);
    let mut reader = WavReader::open(path.clone()).unwrap();

    zip(
        (0..reader.len()).map(|i| i as f64),
        reader.samples::<i16>().map(|s| s.unwrap() as f64),
    )
    .map(|z| [z.0, z.1])
    .step_by(10)
    .collect()
}
