use egui::Vec2b;
use egui_plot::{Line, Plot, PlotPoints, PlotResponse};

use crate::wav_util::WaveUtil;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct VisualizerApp {
    path: String,
    // Example stuff:
    load_file: bool,
    label: String,
    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    #[serde(skip)]
    plot_handles: Vec<PlotResponse<()>>,
    #[serde(skip)]
    wav_util: Option<WaveUtil>,
}

impl Default for VisualizerApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            path: "".to_string(),
            load_file: true,
            plot_handles: Default::default(),
            wav_util: None,
        }
    }
}
impl VisualizerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, path: String) -> Self {
        Self {
            path: path.clone(),
            wav_util: Some(crate::wav_util::WaveUtil::new(path.clone())),
            ..Default::default()
        }
    }
}

impl eframe::App for VisualizerApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        //eframe::set_value(storage, eframe::APP_KEY, self);
        println!("{:}", self.path);
        ()
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();
            if self.load_file {
                //let we = crate::wav_util::WaveUtil::new(self.path.clone());
                let wav: PlotPoints = self
                    .wav_util
                    .as_ref()
                    .expect("couldn't get wave util")
                    .get_samples()
                    .into();
                let line = Line::new(wav);

                let resp = Plot::new("my_plot2")
                    .view_aspect(3.0)
                    .allow_drag(false)
                    .allow_zoom(Vec2b::new(true, false))
                    .allow_scroll(Vec2b::new(true, false))
                    .clamp_grid(true)
                    .show(ui, |plot_ui| {
                        println!("bounds: {:?}", plot_ui.plot_bounds());
                        self.wav_util
                            .as_mut()
                            .expect("couldnt get")
                            .set_range(plot_ui.plot_bounds().min(), plot_ui.plot_bounds().max());
                        plot_ui.line(line)
                    });
                let fft: PlotPoints = self
                    .wav_util
                    .as_ref()
                    .expect("couldn't get it")
                    .get_fft()
                    .into();
                let line2 = Line::new(fft);
                let resp = Plot::new("my_fft")
                    .view_aspect(3.0)
                    .allow_drag(false)
                    .allow_zoom(Vec2b::new(true, false))
                    .allow_scroll(Vec2b::new(true, false))
                    .clamp_grid(true)
                    .show(ui, |plot_ui| {
                        plot_ui.line(line2)
                    });
            }
            ui.separator();
            if ui.button("calc fft").clicked() {
                self.wav_util.as_ref().expect("couldn't get wave util").get_fft();
            }
        });
    }
}
