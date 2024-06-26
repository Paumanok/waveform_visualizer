use egui_plot::{Line, Plot};
use rustfft::{
    num_complex::{Complex, Complex32},
    FftPlanner,
};
use std::iter::zip;
use std::time::{Duration, Instant};

use crate::pcm::PCM;
use egui::Vec2b;

#[derive(Default)]
pub struct FftTransform {
    fft: Vec<Complex32>,
    peaks: Vec<usize>,
    update_count: usize,
    first_run: bool,
    last_update_timer: Option<Instant>,
}

//impl Default for FftTransform {
//    fn default() -> Self {
//        Self {
//            last_update_timer: Instant::now(),
//            ..Default::default()
//        }
//    }
//}

impl FftTransform {
    pub fn new() -> Self {
        Self {
            first_run: true,
            last_update_timer: Some(Instant::now()),
            ..Default::default()
        }
    }

    fn calc_fft(&mut self, pcm: &PCM) {
        self.update_count += 1;
        let (min, max) = pcm.get_window_range();
        let size = max - min;

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(size);
        let subvec: Vec<_> = pcm.contents[min..max].iter().cloned().collect();
        //let mut buffer = vec![Complex{ re: 0.0f32, im: 0.0f32 }; size];
        let mut buffer: Vec<Complex<f32>> = subvec
            .into_iter()
            .map(|s| Complex {
                re: s as f32,
                im: 0.0f32,
            })
            .collect();
        fft.process(&mut buffer);

        self.fft = buffer;
    }

    pub fn get_notes(&mut self, pcm: &mut PCM) -> Vec<String> {
        let fft = self.get_fft(1200.0, pcm);  
        if pcm.changed {
            self.peaks = find_peaks(fft.iter().map(|x| x[1]).collect());
        }
    
        let mut notes: Vec<String> = Vec::new();
        for peak in &self.peaks {
            notes.push(calc_note(fft[peak + 1][0]));
        }
        notes
    }

    pub fn get_fft(&mut self, f_max: f64, pcm: &mut PCM) -> Vec<[f64; 2]> {
        let (min, max) = pcm.get_window_range();
        let size = max - min;

        //println!("{:?}", size / 2 - 1);

        let max_i = size / 2 - 1;
        //let f_max = 10_000.0;
        let max_n = ((f_max * size as f64) / pcm.sample_rate as f64) as usize;

        let end = if max_n < max_i { max_n } else { max_i };

        let wait = Duration::from_millis(match size {
            n if n > 200_000 => 3000,
            n if n > 150_000 => 2000,
            n if n > 100_000 => 1000,
            _ => 500,
        });
        
        if pcm.changed && self.last_update_timer.expect("timer bork").elapsed() > wait || self.first_run {
            self.last_update_timer = Some(Instant::now());
            self.calc_fft(pcm);
            //let duration = self.last_update_timer.expect("timer bork").elapsed();
            //println!("fft time: {:?}", duration);
            if self.first_run { self.first_run = false };
            //pcm.changed = false;
        }
        //[freq, real_component]
        let ret: Vec<_> = zip(
            (0..end).map(|i| i as f64 * pcm.sample_rate as f64 / size as f64),
            self.fft.clone().into_iter().map(|s| s.re as f64),
        )
        .map(|z| [z.0, z.1])
        .step_by(10)
        .collect();
        //if pcm.changed {
        //    let peaks = find_peaks(ret.iter().map(|x| x[1]).collect());
        //    for peak in peaks {
        //        calc_note(ret[peak + 1][0]);
        //    }
        //}
        //println!("ret len: {:}", ret[ret.len()-1][0]);
        ret
    }

    pub fn display(&mut self, f_max: f32, pcm: &mut PCM, ui: &mut egui::Ui) {
        let line = Line::new(self.get_fft(f_max as f64, pcm));
        if pcm.changed { println!("update_count: {:}", self.update_count)}
        Plot::new("fft")
            //.view_aspect(3.0)
            .height(300.0)
            .allow_drag(false)
            .allow_zoom(Vec2b::new(true, false))
            .allow_scroll(Vec2b::new(true, false))
            .clamp_grid(true)
            .x_axis_label("Frequency (Hz)")
            .show(ui, |plot_ui| {
                //println!("bounds: {:?}", plot_ui.plot_bounds());
                plot_ui.line(line)
            });
    }
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
    for (i, sample) in buffer.iter().skip(2).enumerate() {
        if window[0] < window[1] && sample.abs() < window[1] && window[1] > max * 0.20  {
            peaks.push(i);
        }
        window[0] = window[1];
        window[1] = sample.abs();
    }
    for peak in &peaks {
        println!("[{:?}, {:?}]", peak, buffer[*peak + 1].abs());
    }
    peaks
}

pub fn calc_note(freq: f64)-> String {
    let notes = [
        "A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#",
    ];
    // solving for N steps above note: 55Hz
    // where f = note * 2 ^(N/12)
    // N = 12 * round(log10(abs(f) / note) / log10(2))
    let steps = (12.0 * (freq.abs() / 55.0).log10() / 2.0f64.log10()).round() as usize;

    println!(
        "Note freq: {:.3}, Note: {:},  steps above 55Hz {:}",
        freq,
        notes[steps % 12],
        steps
    );
    //+2, one offset because 55hz is A1, one for compensation
    format!("{:}-{:}", notes[steps % 12], (steps / 11) as usize + 1)}
