extern crate midir;
extern crate ghakuf;
#[macro_use]
extern crate log;
extern crate amethyst;
extern crate amethyst_extra;
#[macro_use]
extern crate lazy_static;


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
use amethyst::Application;
use amethyst_extra::*;
use amethyst::ecs::prelude::*;
use amethyst::Error;

use std::time::Duration;

pub mod systems;
pub mod data;
pub use systems::graphics::notes::*;
pub use data::notes::*;
pub use data::midi::*;
pub use data::states::*;



lazy_static! {
    pub static ref KEYBOARD_EVENTS: EventChannel<MidiEvent> = EventChannel::new();
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


/*pub struct NoteDebugSystem;

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
}*/

pub struct AssetData{
    mesh: MeshHandle,
    mat: Material,
}


fn main() -> Result<(), Error> {

    let path = format!("{}/assets/main/config/display.ron", env!("CARGO_MANIFEST_DIR"));
    let input_path = format!("{}/assets/main/config/input.ron", env!("CARGO_MANIFEST_DIR"));

    //let display_config = DisplayConfig::load(path);


    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([1.0, 0.5, 0.75, 1.0], 1.0)
            .with_pass(DrawFlat::<PosTex>::new()),
    );
    //let maps_dir = format!("{}/resources/assets/maps/", env!("CARGO_MANIFEST_DIR"));


    let game_data = GameDataBuilder::default()
        .with_bundle(InputBundle::<String, String>::new().with_bindings_from_file(&input_path))?
        .with_bundle(AudioBundle::new(|music: &mut Time| None))?
        //.with_bundle(RenderBundle::new(pipe, Some(display_config)))
        .with_basic_renderer(path,DrawFlat::<PosTex>::new(),false)?
        .with(NoteSpawnSystem::default(),"note_spawn",&[])
        .with(NoteMoveSystem,"note_move",&["note_spawn"])
        .with(TimedDestroySystem,"timed_destroy",&[])
        .with_bundle(TransformBundle::new().with_dep(&["note_move"]))?;

    let game = Application::build("", GameState::new())?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?
        .run();


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

    println!("Midi file read end");*/


    // create midi input from state
    // move to EventChannel<ghakuf::messages::MidiEvent::MidiEvent>
    // read from the game's systems

    /*println!("Closing connections");
    let (midi_in_, log_all_bytes) = conn_in.close();*/

    Ok(())
}
