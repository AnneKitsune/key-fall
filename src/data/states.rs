use midir::{MidiInput,Ignore,MidiInputConnection};
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

use data::midi::*;
use AssetData;
use KEYBOARD_EVENTS;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;
use std::rc::Rc;
use std::cell::RefCell;

pub struct GameState{
    pub keyboard_connection: Option<MidiInputConnection<Arc<Mutex<EventChannel<MidiEvent>>>>>,
    pub ev: Arc<Mutex<EventChannel<MidiEvent>>>,
}

impl<'a> GameState{
    pub fn new() -> Self{
        GameState{
            keyboard_connection: None,
            ev: Arc::new(Mutex::new(EventChannel::new())),
        }
    }

    pub fn create_keyboard_connection(&mut self){
        // eventchannel lifetime >= keyboard_connection lifetime


        // midir = hardware
        let mut midi_in = MidiInput::new("Key Fall").expect("Failed to create midi input port.");
        midi_in.ignore(Ignore::None);


        println!("Available input ports:");
        let mut input = String::new();
        for i in 0..midi_in.port_count() {
            println!("{}: {}", i, midi_in.port_name(i).expect("Failed to list port name for midi port."));
        }
        print!("Please select input port: ");
        stdout().flush().unwrap();

        // TODO: Add input protection, then menu
        stdin().read_line(&mut input).unwrap();
        let in_port: usize = input.trim().parse().unwrap();

        println!("Connecting...");

        //let mut evch = EventChannel::<MidiEvent>::new();

        let conn_in = midi_in.connect(in_port, "Midi Input", |stamp, message, ev| {
            if message.len() > 1{

                //println!("{}: {:?} (len = {})", stamp, message, message.len());

                // [1,2,3]
                // 1 = state. 144 = down, 128 = up
                // 2 = key.
                // 3 = velocity. [0,128?] down, 64 up

                let time_secs = stamp as f64 / 1000000.0;
                ev.lock().unwrap().single_write(MidiEvent::NoteOn{ch:5,note:5,velocity:5});
                println!("Time: {} -> {:?}",time_secs,message);
            }
        }, self.ev.clone()).expect("Failed to open midi connection.");

        // midi event to note
        // https://github.com/derekdreery/nom-midi-rs/blob/master/src/parser/event/midi.rs


        println!("Connection opened...");
        conn_in
    }
}

impl<'a,'b> State<GameData<'a,'b>> for GameState{
    fn on_start(&mut self, data: StateData<GameData>) {
        println!("Sample midi file read test");
        //let path = Path::new("test.mid");
        let path = Path::new("xi - akasha.mid");
        //let path = Path::new("Sayonara Heaven.mid");
        let mut handler = MidiFileHandler::new();
        {
            let mut reader = Reader::new(
                &mut handler,
                &path,
            ).unwrap();
            let res = reader.read();
        }

        //println!("{:?}",handler);

        data.world.add_resource(handler);


        let mesh = gen_rectangle_mesh(1.0,1.0,&data.world.read_resource(),&data.world.read_resource());
        let color = data.world.read_resource::<Loader>().load_from_data([0.1,0.5,0.3,1.0].into(), (), &data.world.read_resource());


        let mat_defaults = data.world.read_resource::<MaterialDefaults>().0.clone();
        let mat = Material {
            albedo: color.clone(),
            ..mat_defaults.clone()
        };

        let ad = AssetData{
            mesh,
            mat,
        };

        data.world.add_resource(ad);



        data.world
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


        // CREATE MIDI HARDWARE CONNECTION
        let keyboard_connection = self.create_keyboard_connection();


        //self.keyboard_connection = Some(keyboard_connection);
        //data.world.add_resource(keyboard_connection);


    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(data.world);
        Trans::None
    }
}