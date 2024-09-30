struct Camera {
  proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) position: vec2<f32>,
}


@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    out.clip_position = camera.proj * vec4<f32>(((model.position * 0.5) + model.pos) , 0.0, 1.0);
    out.tex_coords = model.position ;
    out.color = model.color;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
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


struct SimulationParams {
  delta_time: f32,
  total: f32,
  position: vec2<f32>,
  color: vec4<f32>,
  dir: vec2<f32>
}

struct Particle {
  position: vec2<f32>,
  dir: vec2<f32>,
  color: vec4<f32>,
  velocity: f32,
  lifetime: f32,
}

var<private> sim_group_index: u32 = 0; 
var<private> idx_count: u32 = 0; 

@binding(0) @group(0) var<storage, read_write> particles_dst : array<Particle>;
@binding(1) @group(0) var<storage> sim_params_groups: array<SimulationParams>;

@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) global_invocation_id : vec3<u32>) {
  let total = arrayLength(&particles_dst);
  let idx = global_invocation_id.x;


  let object_id = idx /  1000;
  let sim_params = sim_params_groups[object_id]; 

  if (idx >= total) {
    return;
  }

  if (f32(idx_count) == sim_params.total) {
    sim_group_index += u32(1);
    idx_count = u32(0);
  }

  var particle: Particle = particles_dst[idx];

  init_rand(idx, vec4f(particle.lifetime, sim_params.delta_time, sim_params.position.x, sim_params.position.y));

  particle.lifetime -= sim_params.delta_time;
  
  if (particle.lifetime < 0.0) {
    particle.position.x = sim_params.position.x;
    particle.position.y = sim_params.position.y;
    particle.dir.x = sim_params.dir.x + rand();
    particle.dir.y = sim_params.dir.y + rand();
    particle.lifetime = rand();
  }
  particle.color = sim_params.color;

  particle.position.x += particle.velocity * particle.dir.x * sim_params.delta_time;
  particle.position.y -= particle.velocity * particle.dir.y * sim_params.delta_time;
  

  idx_count += u32(1);

  particles_dst[idx] = particle;
}