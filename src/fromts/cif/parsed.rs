use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use casey::lower;
use crate::fromts::cif::definitions::{GfxBobLibs, GfxLandscape, GfxLandscapeBuilder, IniCategory};
use crate::fromts::cif::definitions::IniCategory::Unknown;
use crate::fromts::cif::{Item, Section};

pub fn reduce_sections(sections: Vec<Section>) -> Vec<IniCategory> {
    sections.into_iter().map(parse_section).collect()
}

pub fn parse_section(section: Section) -> IniCategory {
    match section.name.to_lowercase().as_str() {
        "GfxLandscape" => IniCategory::GfxLandscape(parse_GfxLandscape(section.items)),
        _ => Unknown(section),
    }
}

fn parse_GfxLandscape(items: Vec<Item>) -> GfxLandscape {
    let mut builder = GfxLandscapeBuilder::default();

    for item in items {
        match item.key.to_lowercase().as_str() {
            lower!("EditName") => builder.EditName(item.value),
            lower!("EditGroups") => builder.EditGroups(item.value),
            lower!("LogicType") => builder.LogicType(item.value.parse().unwrap()),
            lower!("LogicMaximumValency") => builder.LogicMaximumValency(item.value.parse().unwrap()),
            lower!("LogicIsWorkable") => builder.LogicIsWorkable(item.value.parse().unwrap()),
            lower!("logicispileableonmap") => builder.logicispileableonmap(item.value.parse().unwrap()),
            lower!("LogicWalkBlockArea") => builder.LogicWalkBlockArea(parse_coords(&item.value)),
            lower!("LogicBuildBlockArea") => builder.LogicBuildBlockArea(parse_coords(&item.value)),
            lower!("LogicWorkArea") => builder.LogicWorkArea(parse_coords(&item.value)),
            lower!("GfxBobLibs") => builder.GfxBobLibs(parse_GfxBobLibs(&item.value)),
            lower!("GfxPalette") => builder.GfxPalette(Some(parse_GfxPalette(&item.value))),
            lower!("GfxFrames") => parse_GfxFrames(&mut builder, &item.value),
            lower!("GfxStatic") => builder.GfxStatic(item.value.parse().unwrap()),
            lower!("GfxLoopAnimation") => builder.GfxLoopAnimation(item.value.parse().unwrap()),
            lower!("GfxShadingFactor") => builder.GfxShadingFactor(item.value.parse().unwrap()),
            lower!("GfxUserFXMatrix") => builder.GfxUserFXMatrix(item.value.parse().unwrap()),
            lower!("GfxDynamicBackground") => builder.GfxDynamicBackground(item.value.parse().unwrap()),
            lower!("gfxdrawvoidever") => builder.gfxdrawvoidever(item.value.parse().unwrap()),
            lower!("GfxTransition") => parse_GfxTransition(&mut builder, &item.value),
        };
    }

    return builder.build().unwrap();
}

fn parse_GfxBobLibs(value: &String) -> GfxBobLibs {
    let mut parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => GfxBobLibs {
            bmd: parts.remove(0).to_owned(),
            shadow: None,
        },
        2 => GfxBobLibs {
            bmd: parts.remove(0).to_owned(),
            shadow: Some(parts.remove(0).to_owned()),
        },
        _ => panic!("Too many or too little fields for GfxBobLibs")
    }
}

fn parse_GfxPalette(value: &String) -> Vec<String> {
    value.split_whitespace().map(str::to_owned).collect()
}

fn parse_GfxTransition<'a>(builder: &'a mut GfxLandscapeBuilder, value: &String) -> &'a GfxLandscapeBuilder {
    if builder.GfxTransition == None {
        builder.GfxTransition(HashMap::new());
    }
    let existing = builder.GfxTransition.as_mut().unwrap();
    let mut split: Vec<String> = value.split_whitespace().map(str::to_owned).collect();
    let k: u8 = split.get(0).unwrap().parse().unwrap();
    let v: String = split.remove(1).to_owned();
    existing.entry(k).or_insert(v);
    &builder
}

fn parse_GfxFrames<'a>(builder: &'a mut GfxLandscapeBuilder, value: &String) -> &'a GfxLandscapeBuilder {
    if builder.GfxFrames == None {
        builder.GfxFrames(HashMap::new());
    }
    let existing = builder.GfxFrames.as_mut().unwrap();
    for (k, v) in parse_GfxFrames_parts(value) {
        existing.entry(k).or_insert(v);
    }
    &builder
}

fn parse_GfxFrames_parts(s: &String) -> HashMap<u8, Vec<u8>> {
    let parts: Vec<u8> = split_string_to_ints(s);
    let mut i = parts.into_iter();
    let id = i.next().unwrap();

    let mut r = HashMap::new();
    r.insert(id, i.collect());
    return r;
}

fn parse_coords(s: &String) -> ((i8, i8), (i8, i8)) {
    let r: Vec<i8> = split_string_to_ints(s);
    if r.len() != 4 {
        panic!("Expected exactly 4!")
    }
    (
        (*r.get(0).unwrap(), *r.get(1).unwrap()),
        (*r.get(2).unwrap(), *r.get(3).unwrap())
    )
}

fn split_string_to_ints<T: FromStr>(s: &String) -> Vec<T> where <T as FromStr>::Err: Debug {
    s.split_whitespace().map(|x| x.parse::<T>().unwrap()).collect()
}