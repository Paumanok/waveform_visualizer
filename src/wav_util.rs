use hound::{WavReader, WavSpec};
use std::iter::zip;
use rustfft::{num_complex::{Complex, Complex32}, FftPlanner};
//do
//

pub struct WaveUtil {
    //contents: Vec<[f64; 2]>,
    contents: Vec<i16>,
    fft: Vec<Complex32>,
    spec: WavSpec,
    //these represent the window we can see
    //use these for lazy loading and
    //determining skip
    pub min: [f64; 2],
    pub max: [f64; 2],
    last_min_x: usize,
    last_max_x: usize,
    pub changed: bool
}

impl WaveUtil {
    pub fn new(input_file: String) -> Self {
        let (contents, spec) = load_wav(input_file);
        Self {
            contents,
            fft: Default::default(),
            spec,
            min: Default::default(),
            max: Default::default(),
            last_min_x: 0,
            last_max_x: 9,
            changed: false,
        }
    }

    pub fn set_range(&mut self, min: [f64; 2], max: [f64; 2]) {
        if self.last_min_x != min[0] as usize || self.last_max_x != max[0] as usize {
            self.changed = true;
            self.min = min;
            self.max = max;
        }
    }
    pub fn get_window_range( &self) -> (usize, usize) {

        let min = match self.min[0] {
            x if x > 0.0 => x as usize,
            _ => 0
        };

        let max = match self.max[0] {
            x if x < self.contents.len() as f64 && x > 1.0 => x as usize,
            _ => self.contents.len(),
        };

        (min, max)

    }
    pub fn get_samples(&self) -> Vec<[f64; 2]> {
        //self.contents.clone()
        let (mut min, mut max) = self.get_window_range();
        let n_samples = max - min;

        if n_samples == 0 {
            min = 0;
            max = self.contents.len();
        }
        let step = match n_samples {
            x if x > 50_000 => 10,
            x if x > 10_000 => 4,
            _ => 1,
        };

        println!("start: {:} end: {:} size: {:} skip: {:}", min, max,max-min, step);
        zip(
            (min..max).map(|i| i as f64),
            self.contents.clone().into_iter().skip(min).map(|s| s as f64),
        )
        .map(|z| [z.0, z.1])
        .step_by(step)
        .collect()
    }

    fn calc_fft(&self)-> Vec<Complex32> {

        let (min, max) = self.get_window_range();
        let size = max-min;

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(size);
        let subvec: Vec<_> = self.contents[min..max].iter().cloned().collect();
        let mut buffer = vec![Complex{ re: 0.0f32, im: 0.0f32 }; size];
        buffer = subvec.into_iter().map(|s| Complex{re: s as f32, im: 0.0f32}).collect();
        fft.process(&mut buffer);
        buffer
    }

    pub fn get_fft(&mut self) -> Vec<[f64; 2]> {
       
        let (min, max) = self.get_window_range();
        let size = max-min;

        println!("{:?}", size/2 -1);
        
        if self.changed {
            self.fft = self.calc_fft();
        }

        zip(
            (0..(size/2 -1)).map(|i| i as f64 * self.spec.sample_rate as f64 / size as f64),
            self.fft.clone().into_iter().map(|s| s.re as f64),
        )
        .map(|z| [z.0, z.1])
        .step_by(10)
        .collect()

    }
}
//we want to simply load the file and extract some info on the file
//I want to avoid tying this to the plot format just yet
pub fn load_wav(path: String) -> (Vec<i16>, WavSpec) {
    println!("opening: {:?}", path);
    let mut reader = WavReader::open(path.clone()).unwrap();

    (
        reader.samples::<i16>().map(|s| s.unwrap()).collect(),
        reader.spec(),
    )
}

