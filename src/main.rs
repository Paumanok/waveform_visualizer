use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input WAV file to transform
    #[arg(short, long)]
    input_file: String,

    /// Show gui window of plots
    #[arg(short, long, action=ArgAction::SetTrue)]
    gui: bool,
}

fn main() {
    let args = Args::parse();
    println!("opening {:}", args.input_file);

    //let path = Path::new(&args.input_file);
    //let mut _reader = WavReader::open(path).unwrap();

    if args.gui {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1000.0, 400.0])
                .with_min_inner_size([1000.0, 1000.0]),
            ..Default::default()
        };
        let _ = eframe::run_native(
            "eframe template",
            native_options,
            Box::new(|cc| Box::new(fft::VisualizerApp::new(cc, args.input_file))),
        );
    }
}
