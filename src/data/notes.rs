use amethyst::ecs::{DenseVecStorage,Component};

pub struct NoteComponent{
    pub key: u8,
    pub time: f64,
}

impl Component for NoteComponent{
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default,Debug)]
pub struct Note{
    pub key: u8,
    pub channel: u8,
    pub start: f64,
    pub end: f64,
    pub velocity: u8,
}