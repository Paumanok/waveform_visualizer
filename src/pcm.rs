use egui::Vec2b;
use egui_plot::{Legend, Line, Plot};
use hound::{WavReader, WavSpec};
use std::iter::zip;

pub struct PCM {
    pub contents: Vec<i16>,
    pub sample_rate: u32,
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
        else {
            self.changed = false;
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
    pub fn get_window_range(&self) -> (usize, usize) {
        let min = match self.min[0] {
            x if x > 0.0 => x as usize,
            _ => 0,
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

        //println!(
        //    "start: {:} end: {:} size: {:} skip: {:}",
        //    min,
        //    max,
        //    max - min,
        //    step
        //);
        zip(
            (min..max).map(|i| i as f64),
            self.contents
                .clone()
                .into_iter()
                .skip(min)
                .map(|s| s as f64),
        )
        .map(|z| [z.0, z.1])
        .step_by(step)
        .collect()
    }

    pub fn display(&mut self, ui: &mut egui::Ui) {
        let line = Line::new(self.get_samples());
        let sr = self.sample_rate as f64;        
        Plot::new("my_plot2")
            //.view_aspect(3.0)
            .height(300.0)
            .allow_drag(false)
            .allow_zoom(Vec2b::new(true, false))
            .allow_scroll(Vec2b::new(true, false))
            .clamp_grid(true)
            .x_axis_label("Time (m:s:ms)") 
            .y_axis_label("PCM value")
            .x_axis_formatter(move |gm, size, range|
                format!("{:?}:{:?}:{:?}",
                    ((gm.value / sr ) / 60.0) as u32,
                    (gm.value / sr) as u32,
                    ((gm.value / sr) % 1.0 * 1000.0) as u32)
            ) //minute:second:milisecond
                //-> sample/sr/
            .show(ui, |plot_ui| {
                //println!("bounds: {:?}", plot_ui.plot_bounds());
                self.set_range(plot_ui.plot_bounds().min(), plot_ui.plot_bounds().max());
                plot_ui.line(line)
            });
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
