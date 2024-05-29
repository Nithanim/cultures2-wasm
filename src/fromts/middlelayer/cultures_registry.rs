use std::collections::{HashMap, HashSet};
use crate::fromts::cif::read_cif;
use crate::fromts::middlelayer::cultures_fs::CulturesFS;
use crate::fromts::cif::definitions::{GfxLandscape, GfxPalette256, GfxPattern, IniCategory, PatternTransition, Transition};


pub struct CulturesRegistry {
    pub palettes: HashMap<String, GfxPalette256>,
    pub landscapes: HashMap<String, GfxLandscape>,
    pub patterns: HashMap<String, GfxPattern>,
    pub pattern_transitions: HashMap<String, PatternTransition>,
}

async fn load_palettes<'a>(fs: &CulturesFS) -> HashMap<String, GfxPalette256> {
    let PATH = "data\\engine2d\\inis\\palettes\\palettes.cif";
    let cif = read_cif(fs.open(PATH.to_owned())).await.unwrap();

    let mut m = HashMap::<String, GfxPalette256>::new();

    for section in cif {
        match section {
            IniCategory::GfxPalette256(e) => {
                m.insert(e.editname.clone(), e);
            }
            _ => {}
        }
    }
    m
}

async fn load_patterns<'a>(fs: &CulturesFS) -> HashMap<String, GfxPattern> {
    let PATH = "data\\engine2d\\inis\\patterns\\pattern.cif";
    let cif = read_cif(fs.open(PATH.to_owned())).await.unwrap();

    let mut m = HashMap::<String, GfxPattern>::new();

    for section in cif {
        match section {
            IniCategory::GfxPattern(e) => {
                m.insert(e.EditName.clone(), e);
            }
            _ => {}
        }
    }
    m
}

async fn load_pattern_transitions<'a>(fs: &CulturesFS) -> HashMap<String, Transition> {
    let PATH = "data\\engine2d\\inis\\patterntransitions\\transitions.cif";
    let cif = read_cif(fs.open(PATH.to_owned())).await.unwrap();

    let mut m = HashMap::<String, PatternTransition>::new();

    for section in cif {
        match section {
            IniCategory::Transition(e) => {
                m.insert(e.name.clone(), e);
            }
            _ => {}
        }
    }
    m
}

async fn load_landscapes<'a>(fs: &CulturesFS) -> HashMap<String, GfxLandscape> {
    let PATH = "data\\engine2d\\inis\\landscapes\\landscapes.cif";
    let cif = read_cif(fs.open(PATH.to_owned())).await.unwrap();

    let mut m = HashMap::<String, GfxLandscape>::new();

    for section in cif {
        match section {
            IniCategory::GfxLandscape(e) => {
                // TODO there is a default of 0 => [0] added to GfxFrames
                m.insert(e.EditName.clone(), e);
            }
            _ => {}
        }
    }
    m
}

pub async fn load_registry<'a>(fs: &CulturesFS) -> CulturesRegistry {
    return CulturesRegistry {
        palettes: load_palettes(fs).await,
        landscapes: load_landscapes(fs).await,
        patterns: load_patterns(fs).await,
        pattern_transitions: load_pattern_transitions(fs).await,
    };
}
