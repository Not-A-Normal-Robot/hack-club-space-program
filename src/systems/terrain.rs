use crate::components::{
    camera::{SimCameraOffset, SimCameraZoom},
    celestial::{CelestialBody, Terrain},
    frames::RootSpacePosition,
};
use bevy::{ecs::query::QueryData, prelude::*};
use fastnoise_lite::{FastNoiseLite, FractalType};

fn create_noisegen(terrain: Terrain) -> FastNoiseLite {
    let mut noisegen = FastNoiseLite::with_seed(terrain.seed);
    noisegen.fractal_type = FractalType::FBm;
    noisegen.octaves = terrain.octaves;
    noisegen.frequency = terrain.frequency;
    noisegen.gain = terrain.gain;
    noisegen.lacunarity = terrain.lacunarity;
    noisegen
}

fn get_terrain_height(offset: f64, multi: f64, noisegen: FastNoiseLite, theta: f64) -> f64 {
    let (x, y) = theta.sin_cos();

    let noise = noisegen.get_noise_2d(x, y) as f64;

    noise.mul_add(multi, offset)
}

#[derive(Clone, Copy, QueryData)]
#[query_data(mutable)]
struct EntityComponents {
    terrain: &'static Terrain,
    body: &'static CelestialBody,
    pos: &'static RootSpacePosition,
}

#[derive(Clone, Copy)]
struct GlobalData {
    zoom: SimCameraZoom,
    offset: SimCameraOffset,
}

fn update_mesh(mesh: &mut Mesh, entity: EntityComponents, data: GlobalData) {
    // TODO
}

#[cfg(test)]
mod tests {}
