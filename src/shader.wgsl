
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct Camera {
    proj: mat4x4<f32>,
};
@group(1) @binding(0) 
var<uniform> camera: Camera;

struct PlayerModel {
    position: vec2<f32>,
}
@group(2) @binding(0)
var<uniform> player_model: PlayerModel;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    out.clip_position = camera.proj * vec4<f32>(model.position * vec2<f32>(10.0, 10.0) + player_model.position, 0.0, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords) * vec4<f32>(155.0, 100.0, 243.0, 1.0);
}