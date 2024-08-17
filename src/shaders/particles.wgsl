
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
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
    model: mat4x4<f32>,
    color: vec4<f32>,
    tex_scale: vec2<f32>,
    tex_pos: f32
}
@group(2) @binding(0)
var<uniform> player_model: PlayerModel;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    out.clip_position = camera.proj * player_model.model * vec4<f32>(model.position, 0.0, 1.0);
    out.tex_coords = vec2<f32>(model.tex_coords.x, model.tex_coords.y + player_model.tex_pos) * player_model.tex_scale;
    out.color = player_model.color;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords ) *in.color ;
}