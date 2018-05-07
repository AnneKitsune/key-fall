extern crate midir;
extern crate ghakuf;
#[macro_use]
extern crate log;
extern crate amethyst;

use midir::{MidiInput,Ignore};
use std::io::{stdout,stdin,Write};
use ghakuf::messages::*;
use ghakuf::reader::*;
use std::path::Path;

use amethyst::audio::{AudioBundle, SourceHandle};
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::TransformBundle;
use amethyst::core::Time;
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::utils::fps_counter::FPSCounterBundle;
use amethyst::renderer::{DisplayConfig, DrawFlat, Pipeline, PosTex, RenderBundle,
                         Stage};
use std::time::Duration;
use amethyst::Application;

struct MidiFileHandler;
impl Handler for MidiFileHandler {
    fn header(&mut self, format: u16, track: u16, time_base: u16) {
        // Something
    }
    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
        // you
    }
    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
        // want
        println!("Midi event: {:?} at delta time {}",event,delta_time);
    }
    fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
        // to
    }
    fn track_change(&mut self) {
        // do
    }
}


fn main() {

    let path = format!("{}/resources/config.ron", env!("CARGO_MANIFEST_DIR"));
    let display_config = DisplayConfig::load(path);

    let paths = Paths::from_file(&format!("{}/paths.ron", env!("CARGO_MANIFEST_DIR")));
    let input_path = paths
        .path("input")
        .expect("Failed to find input config path")
        .clone();
    println!("{}", input_path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            //.clear_target([255.0, 105.0, 180.0, 1.0], 1.0)
            .clear_target([1.0, 0.5, 0.75, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new()),
    );
    //let maps_dir = format!("{}/resources/assets/maps/", env!("CARGO_MANIFEST_DIR"));
    let game = Application::build("", MenuState)
        .unwrap()
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .with_resource(paths)
        .with_bundle(FPSCounterBundle::new(20))
        .expect("Failed to create FPSCounterBundle")
        .with_bundle(InputBundle::<String, String>::new().with_bindings_from_file(&input_path))
        .expect("Failed to load input bindings")
        .with_bundle(TransformBundle::new())
        .expect("Failed to build transform bundle")
        .with_bundle(AudioBundle::new(|music: &mut Time| None))
        .expect("Failed to build dj bundle")
        .with_bundle(RenderBundle::new(pipe, Some(display_config)))
        .expect("Failed to load render bundle");
    game.build().expect("Failed to build game").run();

    /*println!("Sample midi file read test");
    //let path = Path::new("test.mid");
    let path = Path::new("Sayonara Heaven.mid");
    let mut handler = MidiFileHandler;
    let mut reader = Reader::new(
        &mut handler,
        &path,
    ).unwrap();
    let _ = reader.read();

    //println!("Midi data: {:?}",midi_data);

    println!("Midi file read end");


    let mut midi_in = MidiInput::new("Key Fall").expect("Failed to create midi input");
    midi_in.ignore(Ignore::None);


    println!("Available input ports:");
    let mut input = String::new();
    for i in 0..midi_in.port_count() {
        println!("{}: {}", i, midi_in.port_name(i).unwrap());
    }
    print!("Please select input port: ");
    stdout().flush().unwrap();
    stdin().read_line(&mut input).unwrap();
    let in_port: usize = input.trim().parse().unwrap();

    println!("Connecting...");
    let log_all_bytes = Vec::new(); // We use this as an example custom data to pass into the callback
    let conn_in = midi_in.connect(in_port, "Midi Input", |stamp, message, log| {
        // The last of the three callback parameters is the object that we pass in as last parameter of `connect`.
        if message.len() > 1{

            //println!("{}: {:?} (len = {})", stamp, message, message.len());

            // [1,2,3]
            // 1 = state. 144 = down, 128 = up
            // 2 = key.
            // 3 = velocity. [0,128?] down, 64 up

            let time_secs = stamp as f64 / 1000000.0;
            println!("Time: {} -> {:?}",time_secs,message);
        }
        log.extend_from_slice(message);
    }, log_all_bytes).expect("Failed to open midi connection.");


    // midi event to note
    // https://github.com/derekdreery/nom-midi-rs/blob/master/src/parser/event/midi.rs


    println!("Connection opened.");

    loop{}

    println!("Closing connections");
    let (midi_in_, log_all_bytes) = conn_in.close();*/

}
