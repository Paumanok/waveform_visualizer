
use std::iter::zip;
use rustfft::{num_complex::{Complex, Complex32}, FftPlanner};


pub struct FftTransform {
    changed: &bool,
    fft: Vec<Complex32>,
}


impl FftTransform {


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
