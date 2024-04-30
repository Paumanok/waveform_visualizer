use crate::pcm::PCM;
use crate::transform::FftTransform;
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
    pcm: Option<PCM>,
    #[serde(skip)]
    fft: Option<FftTransform>,
}

impl Default for VisualizerApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 8_000.0,
            path: "".to_string(),
            load_file: true,
            pcm: None,
            fft: None,
        }
    }
}
impl VisualizerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, path: String) -> Self {
        Self {
            path: path.clone(),
            pcm: Some(PCM::new(path.clone())),
            fft: Some(FftTransform::new()),
            ..Default::default()
        }
    }
}

impl eframe::App for VisualizerApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        //eframe::set_value(storage, eframe::APP_KEY, self);
        println!("{:}", self.path);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.separator();
            if self.load_file {
                if let Some(pcm) = &mut self.pcm {
                    pcm.display(ui);
                }

                ui.separator();

                if let Some(pcm) = &mut self.pcm {
                    if let Some(fft) = &mut self.fft {
                        fft.display(self.value, pcm, ui);
                    }
                }
            }
            ui.style_mut().spacing.slider_width = 300.0;
            ui.add(
                egui::Slider::new(&mut self.value, 0.0..=24_000.0)
                    .text("Max frequency")
                    .smallest_positive(50.0),
            );
        });
    }
}
