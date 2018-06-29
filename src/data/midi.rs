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

use data::notes::Note;


#[derive(Default,Debug)]
pub struct MidiSong{
    pub length: f64,
    pub bpm: f64,
    pub notes: Vec<Note>,
}

#[derive(Debug)]
pub struct MidiFileHandler{
    pub song: MidiSong,
    pub delta_accum: f64,
    pub current_bpm: f64,
    pub resolution: u16,
    pub signature: f64,
    pub base_offset: f64,
}

impl MidiFileHandler{
    pub fn new()->Self{
        let mut h = MidiFileHandler{
            song: MidiSong::default(),
            delta_accum: 0.0,
            current_bpm: 120.0,
            resolution: 0,
            signature: 1.0,
            base_offset: 3.0,
        };
        h.reset_delta_accum();
        h
    }
    fn end_note(&mut self, note: u8,time: f64){
        if let Some(mut l) = self.song.notes.iter_mut().filter(|n| n.key == note && n.end == 0.0).last().as_mut(){
            if l.end == 0.0{
                l.end = time;
            }
        }else{
            error!("Could not add NoteOff event to song. Midi file integrity check failed on note key {} at time {}.",note,time);
        }
    }

    fn time_for(&mut self, delta: u32)->f64{
        //let sec_per_beat = 60.0 / self.current_bpm;
        //let sec_per_tick = sec_per_beat / self.resolution as f64;
        let sec_per_tick = 60.0 / (self.current_bpm * self.resolution as f64);
        let offset = sec_per_tick * delta as f64 /* self.signature*/;
        //println!("offset: {}",offset);
        self.delta_accum += offset;
        self.delta_accum
    }
    fn reset_delta_accum(&mut self){
        self.delta_accum = self.base_offset;
    }
}

impl Handler for MidiFileHandler {
    fn header(&mut self, format: u16, track: u16, time_base: u16) {
        self.resolution = time_base;
    }
    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>) {
        match event{
            &MetaEvent::SetTempo => {
                // TODO: support other time signatures than */4
                let b1 = (data[0] as i32) << 16;
                let b2 = (data[1] as i32) << 8;
                let b3 = data[2] as i32;
                let microsec_per_quarter = b1 + b2 + b3;
                self.current_bpm = (60_000_000 / microsec_per_quarter) as f64;
            },
            &MetaEvent::TimeSignature => self.signature = data[0] as f64 / data[1] as f64,
            &MetaEvent::KeySignature => error!("Midi Key Signature meta event not supported yet!"),
            _ => {},
        }
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
        self.reset_delta_accum();
    }
}
