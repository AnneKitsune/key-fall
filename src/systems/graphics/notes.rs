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

use data::notes::*;
use data::midi::*;
use AssetData;

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
        let lower = time.absolute_time_seconds() - time.delta_seconds() as f64 + 1.0;
        let upper = time.absolute_time_seconds() + 1.0;
        for n in &midi.song.notes{
            if n.start < upper{
                if n.start >= lower{
                    println!("Spawning note: {:?}, scheduled for destruction at: {}, at time: {}",n,n.end + 0.1,time.absolute_time_seconds());
                    let e = entities.create();
                    gt.insert(e,GlobalTransform::default());
                    let mut t = Transform::default();
                    t.scale.x = 0.01;
                    t.scale.y = ((n.end - n.start) / 1.0) as f32;
                    tr.insert(e,t);
                    dat.insert(e,DestroyAtTime{ time: n.end + 0.1 });
                    meshes.insert(e,asset.mesh.clone());
                    mats.insert(e,asset.mat.clone());
                    nc.insert(e,NoteComponent{key: n.key,time: n.start});
                }
            }
        }
    }
}

pub struct NoteMoveSystem;

impl<'a> System<'a> for NoteMoveSystem {
    type SystemData = (
        Read<'a, Time>,
        WriteStorage<'a,Transform>,
        ReadStorage<'a,NoteComponent>);
    fn run(&mut self, (time,mut tr, nc): Self::SystemData) {
        for (mut tr, nc) in (&mut tr,&nc).join(){
            tr.translation.x = nc.key as f32 / 100.0;
            // 1.0 = scroll speed.
            tr.translation.y = ((nc.time - time.absolute_time_seconds()) / 1.0) as f32 + tr.scale.y / 2.0;
        }
    }
}