use std::cmp::{min, max};
use std::path::Path;

use super::types::*;

#[derive(Debug)]
pub enum View {
    Isometric,
    Orthographic,
}

#[derive(Debug, PartialEq)]
pub enum Lighting {
    Day,
    Night,
    Nether,
    End,
}

pub struct Options<'a> {
    pub inpath: &'a Path,
    pub view: View,
    pub lighting: Lighting,
    pub blimits: Option<Edges<isize>>,
}

pub fn get_options<'a>(matches: &'a clap::ArgMatches) -> Options<'a> {
    let inpath = Path::new(matches.value_of("PATH").unwrap());

    Options {
        inpath,
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
    }
}
