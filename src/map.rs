use std::error::Error;
use std::time::Instant;

use super::blocktypes;
use super::isomap;
use super::options::{Options, View};
use super::orthomap;
use super::world;

pub fn create_map(options: &Options) -> Result<(), Box<Error>> {
    println!("View:              {}", options.view);
    println!("Lighting:          {}", options.lighting);
    println!("Horizontal limits: {}", match options.blimits {
        Some(blimits) => format!("({}, {}) - ({}, {})", blimits.w, blimits.n, blimits.e, blimits.s),
        _ => "none".to_string(),
    });
    println!("Vertical limits:   {} - {}", options.ylimits.start, options.ylimits.end - 1);

    let start = Instant::now();

    std::fs::create_dir_all(options.outpath.parent().unwrap())?;

    println!("Getting world info from world dir {}", options.inpath.display());
    let world = world::get_world(options.inpath, &options.blimits, &options.ylimits)?;

    println!("Getting block types");
    let blocktypes = blocktypes::get_block_types(&options.lighting);

    println!("Starting block map");
    let result = match options.view {
        View::Isometric =>
            isomap::draw_iso_map(&world, options.outpath, &blocktypes),
        View::Orthographic =>
            orthomap::draw_ortho_map(&world, options.outpath, &blocktypes),
    };

    let elapsed = start.elapsed();
    let mins = elapsed.as_secs() / 60;
    let secs = elapsed.as_secs() % 60;
    let ms = elapsed.subsec_millis();
    println!("Time elapsed: {}:{:02}.{:03}", mins, secs, ms);

    result
}
