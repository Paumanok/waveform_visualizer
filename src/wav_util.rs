//terribly named file awaiting refactoring
//opens the wav file into a buffer
//

use hound::{WavReader, WavSpec};
use std::iter::zip;
use rustfft::{num_complex::{Complex, Complex32}, FftPlanner};
use egui::Vec2b;
use egui_plot::{Line, Plot, PlotPoints, PlotResponse};


pub struct PCM {
    contents: Vec<i16>,
    sample_rate: u32,
    pub min: [f64; 2],
    pub max: [f64; 2],
    pub changed: bool,
}

impl PCM {
    pub fn new(input_file: String) -> Self {
        let (contents, spec) = load_wav(input_file);
        Self {
            contents,
            sample_rate: spec.sample_rate,
            min: Default::default(),
            max: Default::default(),
            changed: false,
        }
    }
    /// Update window range 
    /// Alters internal changed state 
    pub fn set_range(&mut self, min: [f64; 2], max: [f64; 2]) {
        if self.get_min_x_idx() != min[0] as usize || self.get_max_x_idx() != max[0] as usize {
            println!("boundary changed");
            self.changed = true;
            self.min = min;
            self.max = max;
        }
    }
    /// Get min x axis index of window
    pub fn get_min_x_idx(&self) -> usize {
        self.min[0] as usize
    }
    /// Get max x axis index of window
    pub fn get_max_x_idx(&self) -> usize {
        self.max[0] as usize
    }
    /// Translate window range to buffer range, constrained by buffer limits
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

    /// Get samples from current window in the format that egui::plot expects
    pub fn get_samples(&self) -> Vec<[f64; 2]> {
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

    //pub fn display(&self, ui: egui::Ui) -> egui_plot::PlotResponse<> {
    //    Plot::new("my_plot2")
    //                .view_aspect(3.0)
    //                .allow_drag(false)
    //                .allow_zoom(Vec2b::new(true, false))
    //                .allow_scroll(Vec2b::new(true, false))
    //                .clamp_grid(true)
    //                .show(ui, |plot_ui| {
    //                    println!("bounds: {:?}", plot_ui.plot_bounds());
    //                    self.wav_util
    //                        .as_mut()
    //                        .expect("couldnt get")
    //                        .set_range(plot_ui.plot_bounds().min(), plot_ui.plot_bounds().max());
    //                    plot_ui.line(line)
    //                })

    //}
    
}

pub struct WaveUtil {
    contents: Vec<i16>,
    fft: Vec<Complex32>,
    spec: WavSpec,
    //these represent the window we can see
    //use these for lazy loading and
    //determining skip
    pub min: [f64; 2],
    pub max: [f64; 2],
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
            changed: false,
        }
    }

    pub fn set_range(&mut self, min: [f64; 2], max: [f64; 2]) {
        if self.get_min_x_idx() != min[0] as usize || self.get_max_x_idx() != max[0] as usize {
            println!("boundary changed");
            self.changed = true;
            self.min = min;
            self.max = max;
        }
    }

    pub fn get_min_x_idx(&self) -> usize {
        self.min[0] as usize
    }
    pub fn get_max_x_idx(&self) -> usize {
        self.max[0] as usize
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

    pub fn get_fft(&mut self, f_max: f64) -> Vec<[f64; 2]> {
       
        let (min, max) = self.get_window_range();
        let size = max-min;

        println!("{:?}", size/2 -1);

        let max_i = size/2 - 1;
        //let f_max = 10_000.0;
        let max_n = (( f_max * size as f64 ) / self.spec.sample_rate as f64) as usize;

        let end = if max_n < max_i { max_n } else { max_i }; 
        
        if self.changed {
            self.fft = self.calc_fft();
            self.changed = false;
        }
        //[freq, real_component]
        let ret: Vec<_> = zip(
            (0..end).map(|i| i as f64 * self.spec.sample_rate as f64 / size as f64),
            self.fft.clone().into_iter().map(|s| s.re as f64),
        )
        .map(|z| [z.0, z.1])
        .step_by(10)
        .collect();

        let peaks = find_peaks(ret.iter().map(|x| x[1]).collect());
        for peak in peaks {
            calc_note(ret[peak+1][0]);
        }
        //println!("ret len: {:}", ret[ret.len()-1][0]);
        ret

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

pub fn find_peaks(buffer: Vec<f64>) -> Vec<usize> {
    
    let mut window = [0.0_f64; 2];
    let mut peaks = Vec::new();
    window[0] = buffer[0];
    window[1] = buffer[1];

    let mut max = 0.0;

    for sample in &buffer {
        if sample.abs() > max {
            max = sample.abs();
        }
    }
    println!("max: {:}", max);
    for (i,sample) in buffer.iter().skip(2).enumerate() {
        if window[0] < window[1] && sample.abs() < window[1] {
            if window[1] > max * 0.20 {
                peaks.push(i);
            }
        }
        window[0] = window[1];
        window[1] = sample.abs();
    }
    for peak in &peaks {
        println!("[{:?}, {:?}]", peak, buffer[*peak+1].abs()); 

    }
    peaks
}

pub fn calc_note(freq: f64)  {
    let notes = ["A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#"];
    // solving for N steps above note: 55Hz
    // where f = note * 2 ^(N/12)
    let steps = (12.0 * (freq.abs() / 55.0).log10() / 2.0f64.log10()).round() as usize;

    println!("Note freq: {:}, Note: {:},  steps above 55Hz {:}", freq,  notes[steps % 12],steps);


}

