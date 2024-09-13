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
    
    out.clip_position = camera.proj * vec4<f32>(((model.position * 0.8) + model.pos) , 0.0, 1.0);
    out.tex_coords = model.position ;
    out.color = model.color;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let uv = in.tex_coords;
    let d = sqrt(dot(uv,uv));
    let radius = 1.0;
    let bt = 0.0;
    let t1 = 1.0 - smoothstep(radius - bt, radius, d);
    let t2 = 1.0 - smoothstep(radius, radius  + bt, d);

    return vec4(mix(in.color.rgb, in.color.rgb, t1), t2);
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
  screen_dimensions: vec2<f32>,
  target_position: vec2<f32>
}

struct Particle {
  position: vec2<f32>,
  dir: vec2<f32>,
  color: vec4<f32>,
  velocity: f32,
  lifetime: f32
}

  

@binding(0) @group(0) var<storage, read_write> particles_dst : array<Particle>;
@binding(1) @group(0) var<uniform> sim_params: SimulationParams;

@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) global_invocation_id : vec3<u32>) {
  let total = arrayLength(&particles_dst);
  
  let idx = global_invocation_id.x;
  if (idx >= total) {
    return;
  }
    var particle: Particle = particles_dst[idx];
    particle.color.w = smoothstep(0.0, 1.0, particle.lifetime); 
    
    // i don't use lifetime here :P
    particle.lifetime -= 0.2;
    
    particle.position.x += particle.velocity * particle.dir.x * 0.04;
    particle.position.y += particle.velocity * particle.dir.y * 0.04;
    let screen_padding = 50.0;
    let top = 300.0;
    let bottom = -300.0;
    let left = -400.0;
    let right = 400.0;

    if(particle.position.x > sim_params.target_position.x - (left - screen_padding)) {
      particle.position.x = sim_params.target_position.x + (left - screen_padding);
    }
    if(particle.position.x < sim_params.target_position.x - (right + screen_padding)) {
      particle.position.x = sim_params.target_position.x + (right + screen_padding);
    }
    if(particle.position.y < sim_params.target_position.y - (top + screen_padding)) {
      particle.position.y = sim_params.target_position.y + (top + screen_padding);
    }

    if(particle.position.y > sim_params.target_position.y - (bottom - screen_padding)) {
      particle.position.y = sim_params.target_position.y + (bottom - screen_padding);
    }


    particles_dst[idx] = particle;
}