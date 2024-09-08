struct Color {
  color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> color: Color;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,

}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) pos: vec2<f32>,
}


@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    out.clip_position = vec4<f32>(model.position + model.pos, 0.0, 1.0);
    out.tex_coords = model.position * vec2f(100.0,100.0);
    return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords) * color.color;
}

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     return vec4f(1.0);
// }

var<private> rand_seed : vec2<f32>;

fn init_rand(invocation_id : u32, seed : vec4<f32>) {
  rand_seed = seed.xz;
  rand_seed = fract(rand_seed * cos(35.456+f32(invocation_id) * seed.yw));
  rand_seed = fract(rand_seed * cos(41.235+f32(invocation_id) * seed.xw));
}

fn rand() -> f32 {
  rand_seed.x = fract(cos(dot(rand_seed, vec2<f32>(23.14077926, 232.61690225))) * 136.8168);
  rand_seed.y = fract(cos(dot(rand_seed, vec2<f32>(54.47856553, 345.84153136))) * 534.7645);
  return rand_seed.y;
}


// struct SimulationParams {
//   deltaTime : f32,
//   seed : vec4<f32>,
// }

struct Particle {
  position: vec2<f32>,
}

struct Particles {
  particles : array<Particle>,
}

@binding(0) @group(0) var<storage, read_write> data : Particles;


@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) global_invocation_id : vec3<u32>) {
  let idx = global_invocation_id.x;
 
    if (idx < arrayLength(&data.particles)) {
        var particle = data.particles[idx];

        particle.position += vec2f(vec2f(cos(0.5 * rand() * f32(idx)), -sin(0.5 * rand() * f32(idx)))) * 0.016 ;

        data.particles[idx] = particle;
    }
   
}