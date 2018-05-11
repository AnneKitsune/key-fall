extern crate midir;
extern crate ghakuf;
#[macro_use]
extern crate log;
extern crate amethyst;
extern crate amethyst_extra;

use midir::{MidiInput,Ignore};
use std::io::{stdout,stdin,Write};
use ghakuf::messages::*;
use ghakuf::reader::*;
use ghakuf::messages::MidiEvent::*;
use std::path::Path;

use amethyst::audio::{AudioBundle, SourceHandle};
use amethyst::assets::{ProgressCounter,Loader};
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::{Transform,GlobalTransform,TransformBundle};
use amethyst::core::Time;
use amethyst::core::cgmath::{Matrix4,Vector3};
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::utils::fps_counter::FPSCounterBundle;
use amethyst::renderer::{Camera,DisplayConfig, DrawFlat, Pipeline, PosTex, RenderBundle,
                         Stage, Projection,Event,Material,MeshHandle,MaterialDefaults};
use amethyst::shrev::{ReaderId,EventChannel};
use amethyst::ui::UiEvent;
use std::time::Duration;
use amethyst::Application;
use amethyst_extra::*;
use amethyst::ecs::prelude::*;

#[derive(Default,Debug)]
pub struct Note{
    key: u8,
    channel: u8,
    start: f64,
    end: f64,
    velocity: u8,
}

#[derive(Default,Debug)]
pub struct MidiSong{
    length: f64,
    bpm: f64,
    notes: Vec<Note>,
}

#[derive(Debug)]
pub struct MidiFileHandler{
    song: MidiSong,
    delta_accum: f64,
    current_bpm: f64,
    tpm: u16,
}

impl MidiFileHandler{
    pub fn new()->Self{
        MidiFileHandler{
            song: MidiSong::default(),
            delta_accum: 0.0,
            current_bpm: 120.0,
            tpm: 0,
        }
    }
    fn end_note(&mut self, note: u8,time: f64){
        if let Some(mut l) = self.song.notes.iter_mut().filter(|n| n.key == note).last().as_mut(){
            if l.end == 0.0{
                l.end = time;
            }
        }
    }

    fn time_for(&mut self, delta: u32)->f64{
        let microsec_per_beat = 60.0 * 1000000.0 / self.current_bpm;
        let microsec_per_tick = microsec_per_beat / self.tpm as f64;
        let sec_per_tick = microsec_per_tick / 1000000.0;
        let offset = sec_per_tick * delta as f64;
        self.delta_accum += offset;
        self.delta_accum
    }
}

impl Handler for MidiFileHandler {
    fn header(&mut self, format: u16, track: u16, time_base: u16) {
        self.tpm = time_base;
    }
    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
        // TODO: time changes
    }
    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent) {
        //println!("Midi event: {:?} at delta time {}",event,delta_time);
        let t = self.time_for(delta_time);
        match event{
            &NoteOn{ch,note,velocity} => {
                if velocity == 0 {
                    self.end_note(note, t);
                } else {
                    self.song.notes.push(Note {
                        key: note,
                        channel: ch,
                        start: t,
                        end: 0.0,
                        velocity,
                    });
                }
            },
            &NoteOff{ch,note,velocity} => self.end_note(note,t),
            _ => {},
        }
    }
    fn sys_ex_event(&mut self, delta_time: u32, event: &SysExEvent, data: &Vec<u8>) {
    }
    fn track_change(&mut self) {
    }
}



#[derive(Default)]
struct LoadState{
    progress: ProgressCounter,
}

struct MainMenuState{
    ui_event_reader: Option<ReaderId<UiEvent>>,
}

struct DeviceChooserState;

struct PauseState;

struct GameState;


/*impl State for LoadState{
    fn on_start(&mut self, mut world: &mut World) {
        // Load stuff
    }

    fn update(&mut self, mut world: &mut World) -> Trans {
        if self.progress.is_complete() {
            Trans::Switch(Box::new(MainMenuState))
        }else{
            Trans::None
        }
    }

    fn handle_event(&mut self, mut world: &mut World, event: Event) -> Trans {
        self.internal.handle_event(&mut world, event)
    }
}*/

/*impl State for MainMenuState{
    fn on_start(&mut self, mut world: &mut World) {
        self.ui_event_reader = Some(world.read::<EventChannel<UiEvent>>().register_reader());
        // add buttons

    }

    fn update(&mut self, mut world: &mut World) -> Trans {
        for ev in world.read::<EventChannel<UiEvent>>().read(&self.ui_event_reader){
            if let Some(nav) = world.read_storage::<NavigationButton>(){
                return nav.target();
            }
        }
    }

    fn handle_event(&mut self, mut world: &mut World, event: Event) -> Trans {
        self.internal.handle_event(&mut world, event)
    }
}*/


impl State for GameState{
    fn on_start(&mut self, mut world: &mut World) {
        println!("Sample midi file read test");
        //let path = Path::new("test.mid");
        let path = Path::new("Sayonara Heaven.mid");
        let mut handler = MidiFileHandler::new();
        {
            let mut reader = Reader::new(
                &mut handler,
                &path,
            ).unwrap();
            let res = reader.read();
        }
        world.add_resource(handler);


        let mesh = gen_rectangle_mesh(0.1,0.1,&world.read_resource(),&world.read_resource());
        let color = world.read_resource::<Loader>().load_from_data([0.1,0.5,0.3,1.0].into(), (), &world.read_resource());


        let mat_defaults = world.read_resource::<MaterialDefaults>().0.clone();
        let mat = Material {
            albedo: color.clone(),
            ..mat_defaults.clone()
        };

        let ad = AssetData{
            mesh,
            mat,
        };

        world.add_resource(ad);



        world
            .create_entity()
            .with(Camera::from(Projection::orthographic(
                0.0,
                1.0,
                1.0,
                0.0,
            )))
            .with(GlobalTransform(
                Matrix4::from_translation(Vector3::new(0.0, 0.0, 1.0)).into(),
            ))
            .build();
    }

    fn update(&mut self, mut world: &mut World) -> Trans {
        Trans::None
    }
}

pub struct NoteDebugSystem;

impl<'a> System<'a> for NoteDebugSystem {
    type SystemData = (Read<'a, Time>, ReadExpect<'a, MidiFileHandler>);
    fn run(&mut self, (time, midi): Self::SystemData) {
        let lower = time.absolute_time_seconds() - time.delta_seconds() as f64;
        let upper = time.absolute_time_seconds();
        for n in &midi.song.notes{
            if n.start >= lower && n.start < upper{
                println!("Note Start: {:?}",n);
            }
        }
    }
}

pub struct AssetData{
    mesh: MeshHandle,
    mat: Material,
}

pub struct NoteComponent{
    key: u8,
    time: f64,
}

impl Component for NoteComponent{
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct NoteSpawnSystem{
    last_spawn: f64,
}

impl NoteSpawnSystem{
    pub fn can_spawn(&mut self, time: f64, note_time: f64, last_frame: f64)->bool{
        // 1.0 = note scroll delay
        note_time < time - 1.0 && note_time > self.last_spawn
    }
}

impl<'a> System<'a> for NoteSpawnSystem {
    type SystemData = (
                       Entities<'a>,
                       Read<'a, Time>,
                       ReadExpect<'a,AssetData>,
                       ReadExpect<'a, MidiFileHandler>,
                       WriteStorage<'a,GlobalTransform>,
                       WriteStorage<'a,Transform>,
                       WriteStorage<'a,DestroyAtTime>,
                       WriteStorage<'a,MeshHandle>,
                       WriteStorage<'a,Material>,
                       WriteStorage<'a,NoteComponent>,);
    fn run(&mut self, (entities,time, asset, midi, mut gt, mut tr, mut dat, mut meshes, mut mats, mut nc): Self::SystemData) {
        let lower = time.absolute_time_seconds() - time.delta_seconds() as f64;
        let upper = time.absolute_time_seconds();
        for n in &midi.song.notes{
            if n.start < upper{
                if n.start >= lower{
                    println!("Spawning note: {:?}, scheduled for destruction at: {}",n,time.absolute_time_seconds() + n.end + 0.1);
                    let e = entities.create();
                    gt.insert(e,GlobalTransform::default());
                    tr.insert(e,Transform::default());
                    dat.insert(e,DestroyAtTime{ time: time.absolute_time_seconds() + n.end + 0.1 });
                    meshes.insert(e,asset.mesh.clone());
                    mats.insert(e,asset.mat.clone());
                    nc.insert(e,NoteComponent{key: n.key,time: n.start});
                }
            }else{
                break;
            }
        }
    }
}

pub struct NoteMoveSystem;

impl<'a> System<'a> for NoteMoveSystem {
    type SystemData = (
        Read<'a, Time>,
        ReadExpect<'a, MidiFileHandler>,
        WriteStorage<'a,Transform>,
        ReadStorage<'a,NoteComponent>);
    fn run(&mut self, (time,midi,mut tr, nc): Self::SystemData) {
        for (mut tr, nc) in (&mut tr,&nc).join(){
            tr.translation.x = (nc.key as f32 - 50.0) / 30.0;
            // 1.0 = scroll speed.
            tr.translation.y = (nc.time + (nc.time - time.absolute_time_seconds()) / 1.0) as f32;
        }
    }
}

fn main() {

    let path = format!("{}/assets/main/config/display.ron", env!("CARGO_MANIFEST_DIR"));
    let input_path = format!("{}/assets/main/config/input.ron", env!("CARGO_MANIFEST_DIR"));

    let display_config = DisplayConfig::load(path);


    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            //.clear_target([255.0, 105.0, 180.0, 1.0], 1.0)
            .clear_target([1.0, 0.5, 0.75, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new()),
    );
    //let maps_dir = format!("{}/resources/assets/maps/", env!("CARGO_MANIFEST_DIR"));
    let game = Application::build("", GameState)
        .unwrap()
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        //.with_bundle(FPSCounterBundle::new(20))
        //.expect("Failed to create FPSCounterBundle")
        .with_bundle(InputBundle::<String, String>::new().with_bindings_from_file(&input_path))
        .expect("Failed to load input bindings")
        .with_bundle(AudioBundle::new(|music: &mut Time| None))
        .expect("Failed to build dj bundle")
        .with_bundle(RenderBundle::new(pipe, Some(display_config)))
        .expect("Failed to load render bundle")
        .with(NoteSpawnSystem::default(),"note_spawn",&[])
        .with(NoteMoveSystem,"note_move",&["note_spawn"])
        .with_bundle(TransformBundle::new().with_dep(&["note_move"]))
        .expect("Failed to build transform bundle");
    game.build().expect("Failed to build game").run();

    /*println!("Sample midi file read test");
    //let path = Path::new("test.mid");
    let path = Path::new("Sayonara Heaven.mid");
    let mut handler = MidiFileHandler::new();
    {
        let mut reader = Reader::new(
            &mut handler,
            &path,
        ).unwrap();
        let res = reader.read();
    }

    println!("Midi data: {:?}",handler);

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
