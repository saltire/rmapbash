use std::cmp::{min, max};
use std::fmt;
use std::ops::Range;
use std::path::Path;

use super::sizes::*;
use super::types::*;

#[derive(Debug)]
pub enum View {
    Isometric,
    Orthographic,
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq)]
pub enum Lighting {
    Day,
    Night,
    Nether,
    End,
}

impl fmt::Display for Lighting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Options<'a> {
    pub inpath: &'a Path,
    pub outpath: &'a Path,
    pub view: View,
    pub lighting: Lighting,
    pub blimits: Option<Edges<isize>>,
    pub ylimits: Range<usize>,
}

pub fn get_options<'a>(matches: &'a clap::ArgMatches) -> Options<'a> {
    let inpath = Path::new(matches.value_of("INPATH").unwrap());
    let outpath = Path::new(matches.value_of("OUTPATH").unwrap());

    Options {
        inpath,
        outpath,
        view: if matches.is_present("i") { View::Isometric } else { View::Orthographic },
        lighting: match inpath.file_stem().unwrap().to_str() {
            Some("DIM-1") => Lighting::Nether,
            Some("DIM1") => Lighting::End,
            _ => if matches.is_present("n") { Lighting::Night } else { Lighting::Day },
        },
        blimits: matches.values_of("b").and_then(|mut b| {
            let x1 = b.next().unwrap().parse::<isize>().unwrap();
            let z1 = b.next().unwrap().parse::<isize>().unwrap();
            let x2 = b.next().unwrap().parse::<isize>().unwrap();
            let z2 = b.next().unwrap().parse::<isize>().unwrap();
            Some(Edges {
                n: min(z1, z2),
                e: max(x1, x2),
                s: max(z1, z2),
                w: min(x1, x2),
            })
        }),
        ylimits: match matches.values_of("y") {
            Some(mut y) => {
                let y1 = min(y.next().unwrap().parse::<usize>().unwrap(), MAX_BLOCK_IN_CHUNK_Y);
                let y2 = min(y.next().unwrap().parse::<usize>().unwrap(), MAX_BLOCK_IN_CHUNK_Y);
                min(y1, y2)..(max(y1, y2) + 1)
            },
            None => 0..BLOCKS_IN_CHUNK_Y,
        },
    }
}
