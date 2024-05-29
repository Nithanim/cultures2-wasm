/*
E.g.

[GfxPattern]
EditName "border"
EditGroups "misc"
LogicType 0
GfxTexture "data\engine2d\bin\textures\text_000.pcx"
GfxCoordsA 0 0 63 63 0 63
GfxCoordsB 0 0 63 0 63 63



[GfxLandscape]
EditName "player01 sign 01"
EditGroups "misc_signs"
LogicType 1
LogicMaximumValency 1
LogicIsWorkable 0
logicispileableonmap 0
GfxBobLibs "data\engine2d\bin\bobs\ls_temp.bmd" "data\engine2d\bin\bobs\ls_temp_s.bmd"
GfxPalette "human_Player01"
GfxFrames 1 33
GfxStatic 0
GfxLoopAnimation 0
GfxShadingFactor 1.000000
GfxUserFXMatrix 0
GfxDynamicBackground 0
gfxdrawvoidever 0

 */

use std::collections::{HashMap, HashSet};
use derive_builder::Builder;
use crate::fromts::cif::Section;

pub enum IniCategoryType {
    Text,
    GfxLandscape,
    GfxPalette256,
    GfxPattern,
    Transition,
}

pub enum IniCategory {
    Text(Text),
    GfxLandscape(GfxLandscape),
    GfxPalette256(GfxPalette256),
    GfxPattern(GfxPattern),
    Transition(Transition),
    Unknown(Section),
}


/**
Defines text for stuff based on context.
Examples are game definitions files "chesttypes.ini" or "experiences.ini".
Both define strings starting at 1 but they define completely different things.
The same happens for use maps, where the strings can also be defined freely, like starting with 1.

Normally, strings are defined with `stringn <id> "<string>"` but also `string "<string>"` can be used.
`string` is the shorter form of `stringn`, just assuming the next higher id.
E.g.:
```ini
[text]
stringn 1 "Small nourishing potion"
string "Big nourishing potion"
string "Small stamina potion"
```
Is the same as
```ini
[text]
stringn 1 "Small nourishing potion"
stringn 2 "Big nourishing potion"
stringn 3 "Small stamina potion"
```
 */
pub struct Text {
    strings: HashMap<u32, String>,
}

/**
```ini
[GfxLandscape]
EditName "player01 sign 01"
EditGroups "misc_signs"
LogicType 1
LogicMaximumValency 1
LogicIsWorkable 0
logicispileableonmap 0
GfxBobLibs "data\engine2d\bin\bobs\ls_temp.bmd" "data\engine2d\bin\bobs\ls_temp_s.bmd"
GfxPalette "human_Player01"
GfxFrames 1 33
GfxStatic 0
GfxLoopAnimation 0
GfxShadingFactor 1.000000
GfxUserFXMatrix 0
GfxDynamicBackground 0
gfxdrawvoidever 0
GfxTransition 3 "tree trunk 01"
GfxTransition 2 "tree debris small"
```
 */
#[allow(non_snake_case)]
#[derive(Builder)]
pub struct GfxLandscape {
    pub EditName: String,
    pub EditGroups: String,
    pub LogicType: u8,
    pub LogicMaximumValency: u8,
    pub LogicIsWorkable: bool,
    pub logicispileableonmap: bool,
    pub LogicWalkBlockArea: ((i8, i8), (i8, i8)),
    pub LogicBuildBlockArea: ((i8, i8), (i8, i8)),
    pub LogicWorkArea: ((i8, i8), (i8, i8)),
    /// Path to bmd file
    pub GfxBobLibs: GfxBobLibs,
    /// Palette name
    pub GfxPalette: Option<Vec<String>>,
    /// The first number is an "id" (or index? or whatever) and is always "1" for landscapes ("0" for others).
    /// The rest are the frame ids.
    /// Defining multiple ids can be done in multiple lines (for non-landscapes, like "particels")
    /// But again, not used for landscapes
    #[builder(field(public))]
    pub GfxFrames: HashMap<u8, Vec<u8>>,
    pub GfxStatic: bool,
    pub GfxLoopAnimation: bool,
    pub GfxShadingFactor: f32,
    /// Who knows what this is. Always 0 when defined.
    pub GfxUserFXMatrix: u8,
    pub GfxDynamicBackground: bool,
    pub gfxdrawvoidever: bool,
    /**

    E.g.
    ```ini
    GfxTransition 3 "tree trunk 01"
    GfxTransition 2 "tree debris small"
    ```
     */
    #[builder(field(public))]
    pub GfxTransition: HashMap<u8, String>,
}

#[derive(Clone)]
pub struct GfxBobLibs {
    pub bmd: String,
    pub shadow: Option<String>,
}

/**
```ini
[GfxPalette256]
editname "Ship_house"
gfxfile "data\engine2d\bin\palettes\creatures\Ship_house.pcx"
```
 */
#[allow(non_snake_case)]
pub struct GfxPalette256 {
    pub editname: String,
    pub gfxfile: String,
    pub gfxpreshade: bool,
    pub gfxremaptopreshaded: Option<String>,
}

/**
```ini
[GfxPattern]
EditName "block mountain 00 01 02"
EditGroups "mountain 3x3" "mountain all"
LogicType 3
GfxTexture "data\engine2d\bin\textures\text_200.pcx"
GfxCoordsA 64 128 127 191 64 191
GfxCoordsB 64 128 127 128 127 191
```
 */
#[allow(non_snake_case)]
pub struct GfxPattern {
    pub EditName: String,
    pub EditGroups: HashSet<String>,
    pub LogicType: u8,
    pub GfxTexture: String,
    pub GfxCoordsA: Box<[u8]>,
    pub GfxCoordsB: Box<[u8]>,
}

/**
```ini
[transition]
name "coast 2"
pointtype "meadow"
GfxTexture "data\engine2d\bin\textures\tran_water_coast.pcx"
GfxTextureAlpha "data\engine2d\bin\textures\tran_water_coast_a.pcx"
GfxCoordsA 0 128 63 191 0 191
GfxCoordsB 0 128 63 128 63 191
GfxCoordsA 64 128 127 191 64 191
GfxCoordsB 64 128 127 128 127 191
GfxCoordsA 128 128 191 191 128 191
GfxCoordsB 128 128 191 128 191 191
GfxCoordsA 0 192 63 255 0 255
GfxCoordsB 0 192 63 192 63 255
GfxCoordsA 64 192 127 255 64 255
GfxCoordsB 64 192 127 192 127 255
GfxCoordsA 128 192 191 255 128 255
GfxCoordsB 128 192 191 192 191 255
```
 */
#[allow(non_snake_case)]
pub struct Transition {
    pub name: String,
    pub pointtype: String,
    pub GfxTexture: String,
    pub GfxTextureAlpha: String,
    pub GfxCoordsA: Vec<Vec<u8>>,
    pub GfxCoordsB: Vec<Vec<u8>>,
}

// TODO needed?
pub use Transition as PatternTransition;
