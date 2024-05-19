use std::collections::{HashMap, HashSet};
use std::iter::Map;
use wasm_bindgen::Clamped;
use web_sys::ImageData;
use crate::fromts::middlelayer::cultures_fs::CulturesFS;
use crate::fromts::middlelayer::cultures_registry::CulturesRegistry;
use crate::fromts::pcx::{Pcx, pcx_read};

pub struct CulturesResourceManager {
    fs: CulturesFS,
    registry: CulturesRegistry,
    // worker_pool: WorkerPool;
    private pattern_cache: Map<string, Promise<ImageData>> = new Map();

    constructor(fs: CulturesFS,
    registry: CulturesRegistry) {
    this.fs = fs;
    // this.worker_pool = new WorkerPool(worker, 10);
    this.registry = registry;
    }
}



impl CulturesResourceManager {

async fn load_pattern(&self, name: String) -> ImageData {
    let path = &self.registry.patterns.get(&name).unwrap().GfxTexture;
    let cache = self.pattern_cache.get(path);
    if let Ok(o) = cache {
        return o;
    }

    let blob = self.fs.open(path);
    let img_p = pcx_read(blob, None).await;

    self.pattern_cache.set(path, img_p);

    return img_p;
}

    fn pxc_to_image_data(pcx: Pcx) {
        crate::fromts::pcx::JsImageData::new_with_u8_clamped_array_and_sh(Clamped(&*pcx.data), pcx.width, pcx.height).unwrap();
    }

async fn load_landscape_bmd(&self, bmds: Vec<String>) {

    // TODO just don't care about filtering for actually used right now.
    let landscapes = self.registry.landscapes.values();

    // TODO also skip the dedup of same images too
    let paths = landscapes.map(|e|e.GfxBobLibs).collect();

    let paths_index: Record<string, number> = Object.fromEntries(Object.entries(paths).map(([k, v]) => [v.bmd, parseInt(k)]));
landscapes.sort((l1, l2) => paths_index[l1.GfxBobLibs.bmd] - paths_index[l2.GfxBobLibs.bmd]);

    let palette_paths = uniq(Array.from(this.registry.palettes.values())).map(p => p.gfxfile);
    let palettes_index = Object.fromEntries(Object.entries(palette_paths).map(([k, v]) => [v, parseInt(k)]));

// Frame instances per BMD file
let bmd_frame_instances = new Map<number, Set<number>>(paths.map(p => [paths_index[p.bmd], new Set()]));
for (let lnd of landscapes) {
let c = bmd_frame_instances.get(paths_index[lnd.GfxBobLibs.bmd])!;
let pal = palettes_index[this.registry.palettes.get(lnd.GfxPalette[0])!.gfxfile];

for (let level of Object.keys(lnd.GfxFrames)) {
for (let f of lnd.GfxFrames[parseInt(level)]) {
c.add(f * 1000 + pal);
}
}
}

let bmd_frame_instance_count = [...bmd_frame_instances.values()].reduce((s, c) => s + c.size, 0);
    let frame_palette_index = new Uint32Array(paths.length * 2 + 2 * bmd_frame_instance_count);
frame_palette_index.set(Array.from(bmd_frame_instances.entries()).sort((a, b) => paths_index[a[0]] - paths_index[b[0]]).map(a => a[1].size));

// start after bmd_frame_instance_count table
let frame_palette_index_ptr = paths.length;
    let layers_index = new Map<number, number>();
    let bmd_frame_ptr = new Map<number, number>();

for (let [path_idx, frame_instances] of bmd_frame_instances.entries()) {
for (let fi of frame_instances) {
let pal = fi % 1000;
let f = Math.floor(fi / 1000);

frame_palette_index.set([
f,
pal,
], frame_palette_index_ptr);
frame_palette_index_ptr += 2;

            let layer = bmd_frame_ptr.get(path_idx) || 0;
layers_index.set(path_idx * 1000000 + fi, layer);
bmd_frame_ptr.set(path_idx, layer + 1);
}
}

    let bmd_tables = paths.reduce<{ index: Uint32Array; has_shadow: Uint8Array; acc_length: number; acc_frames: number; }>((s, path, i) => {
        let bob_stats = this.fs.stats(path.bmd);
        let shadow_stats = path.shadow ? this.fs.stats(path.shadow) : null;

s.index[i] = s.acc_length;
s.has_shadow[i] = shadow_stats ? 1 : 0;
s.acc_length += bob_stats.length + (shadow_stats ? shadow_stats.length : 0);

return s;
}, {
index: new Uint32Array(paths.length),
has_shadow: new Uint8Array(paths.length),
acc_length: 0,
acc_frames: 0,
});

    let palette_tables = palette_paths.reduce<{ index: Uint32Array; acc_length: number }>((s, path, i) => {
        let stats = this.fs.stats(path);

s.index[i] = s.acc_length;
s.acc_length += stats.length;

return s;
}, {
index: new Uint32Array(palette_paths.length),
acc_length: 0,
});

    let buf = new Uint8Array(bmd_tables.acc_length);

await Promise.all(paths.map(async (path, i) => {
const blob = this.fs.open(path.bmd);
const bmd_buf = await read_file(blob);
buf.set(new Uint8Array(bmd_buf), bmd_tables.index[i]);

if (path.shadow) {
const shadow = this.fs.open(path.shadow);
const shadow_buf = await read_file(shadow);
buf.set(new Uint8Array(shadow_buf), bmd_tables.index[i] + bmd_buf.byteLength);
}
}));

const palettes_buf = new Uint8Array(palette_tables.acc_length);

await Promise.all(palette_paths.map(async (path, i) => {
const blob = this.fs.open(path);

const buf = await read_file(blob);
palettes_buf.set(new Uint8Array(buf), palette_tables.index[i]);
}));
console.timeEnd('load_landscape_bmd');

let res_buf = create_bmd_texture_array(
buf,
palettes_buf,
bmd_tables.index,
new Uint32Array(Array.from(bmd_frame_ptr.entries()).sort((a, b) => paths_index[a[0]] - paths_index[b[0]]).map(a => a[1])),
bmd_tables.has_shadow,
palette_tables.index,
frame_palette_index
);

return {
layers_index,
paths_index,
palettes_index,
buf: res_buf
};
}

async load_all_patterns(): Promise<{ paths: string[]; image: ArrayBufferView; width: number; height: number }> {
const { create_2d_texture } = await import('cultures2-wasm');
const paths = uniq(Array.from(this.registry.patterns.values()).map(p => p.GfxTexture));

const index_tables = paths.reduce<{ index: Uint32Array; acc_length: number }>((s, path, i) => {
const stats = this.fs.stats(path);

s.index[i] = s.acc_length;
s.acc_length += stats.length;

return s;
}, {
index: new Uint32Array(paths.length),
acc_length: 0,
});

const buf = new Uint8Array(index_tables.acc_length);

await Promise.all(paths.map(async (path, i) => {
const blob = this.fs.open(path);

const tex_buf = await read_file(blob);
buf.set(new Uint8Array(tex_buf), index_tables.index[i]);
}));
const img_buf = create_2d_texture(256, 256, buf, index_tables.index);

return {
paths,
image: img_buf,
width: 256,
height: 256
};
}

async load_all_pattern_transitions(): Promise<{ paths: string[]; image: ArrayBufferView; width: number; height: number; }> {
const { create_2d_texture_masked } = await import('cultures2-wasm');

const transitions = new Map<string, PatternTransition>();
for (const tr of this.registry.pattern_transitions.values()) {
if (transitions.has(tr.GfxTexture)) continue;
transitions.set(tr.GfxTexture, tr);
}

const paths = Array.from(transitions.keys());
const index_tables = paths.reduce<{ index: Uint32Array; mask_index: Uint32Array; acc_length: number }>((s, path, i) => {
const texture_stats = this.fs.stats(path);
const mask_stats = this.fs.stats(transitions.get(path)!.GfxTextureAlpha);

s.index[i] = s.acc_length;
s.mask_index[i] = s.acc_length + texture_stats.length;
s.acc_length += texture_stats.length + mask_stats.length;

return s;
}, {
index: new Uint32Array(paths.length),
mask_index: new Uint32Array(paths.length),
acc_length: 0,
});

const buf = new Uint8Array(index_tables.acc_length);

await Promise.all(paths.map(async (path, i) => {
const blob = this.fs.open(path);
const mask = this.fs.open(transitions.get(path)!.GfxTextureAlpha);

return Promise.all([
read_file(blob).then(tex_buf => buf.set(new Uint8Array(tex_buf), index_tables.index[i]), ex => Promise.reject(ex)),
read_file(mask).then(mask_buf => buf.set(new Uint8Array(mask_buf), index_tables.mask_index[i]), ex => Promise.reject(ex)),
]);
}));

const img_buf = create_2d_texture_masked(256, 256, buf, index_tables.index, index_tables.mask_index);

return {
paths,
image: img_buf,
width: 256,
height: 256
};
}

async load_map(path: string) {
const blob = this.fs.open(path);
return read_map_data(blob);
}