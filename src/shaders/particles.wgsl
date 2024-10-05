const PI: f32 = 3.14159265358;

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

fn gen_range(min: f32, max: f32) -> f32 {
  return min + (max - min) * rand();
}

fn opposite(degrees: f32) -> f32 {
  return (degrees + 180) % 360; 
}


//generate a cone from a direction vector
fn cone(dir: vec2f, theta: f32) -> vec2f {
  let angle = atan2(dir.y, dir.x);
  
  return vec2f(
    cos(gen_range(
      radians(degrees(angle) - theta / 2.0), 
      radians(degrees(angle) + theta / 2.0))), 
    sin(gen_range(
      radians(degrees(angle) - theta / 2.0),  
      radians(degrees(angle) + theta / 2.0)
    )));
}

fn circle(radius: f32, x: f32, y: f32) -> vec2f {
  let length = length(gen_range(0.0, exp2(radius)));
  let degree = gen_range(0.0, 1.0) * 2.0 * PI;
  let dx = x + length * cos(degree);
  let dy = y + length * sin(degree);

  return vec2f(dx, dy);
}

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
    
    out.clip_position = camera.proj * vec4<f32>(((model.position * 1.0) + model.pos) , 0.0, 1.0);
    out.tex_coords = model.position ;
    out.color = model.color;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

struct SimulationParams {
  delta_time: f32,
  total: f32,
  position: vec2<f32>,
  color: vec4<f32>,
  dir: vec2<f32>,
  color_over_lifetime: f32,
  arc: f32,
  rate_over_distance: f32,
  distance_traveled: f32,

}

// struct EmitterData {
//   prev_position: vec2<f32>,
// }

struct Particle {
  position: vec2<f32>,
  dir: vec2<f32>,
  color: vec4<f32>,
  velocity: f32,
  lifetime: f32,
}


@binding(0) @group(0) var<storage, read_write> particles_dst : array<Particle>;
@binding(1) @group(0) var<storage> sim_params_groups: array<SimulationParams>;
// @binding(2) @group(0) var<storage, read_write> emitter_data_groups: array<EmitterData>;

@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) global_invocation_id : vec3<u32>) {
  let total = arrayLength(&particles_dst);
  let total_sim_params = arrayLength(&sim_params_groups);
  
  let idx = global_invocation_id.x;
  
  if (idx >= total) {
    return;
  }
  // 2796202 

  var group_id = 0u;
  var cumulative_total = 0u;

  for (var i = 0u; i < arrayLength(&sim_params_groups); i++) {
      cumulative_total += u32(sim_params_groups[i].total);
      if (idx < cumulative_total) {
          group_id = i;
          break;
      }
  }
  
  var sim_params = sim_params_groups[group_id]; 
  // var emitter_data = emitter_data_groups[group_id];
  
  var particle: Particle = particles_dst[idx];
  init_rand(idx, vec4f(f32(idx), sim_params.delta_time, particle.position.x, particle.position.y));


  let dir: vec2f = normalize(cone(sim_params.dir, sim_params.arc)); 
  
  particle.lifetime -=  sim_params.delta_time;


  if (particle.lifetime < 0.0) {
      let p = circle(2.0, sim_params.position.x, sim_params.position.y);
      particle.position.x =  p.x;
      particle.position.y = p.y ;
      particle.dir.x = dir.x;
      particle.dir.y = dir.y;
      particle.lifetime = rand() * 0.5;
      particle.velocity = 0.0;
    if(sim_params.distance_traveled > sim_params.rate_over_distance) {
      particle.color = sim_params.color;
    } else {
      particle.color = vec4f(0.0, 0.0, 0.0, 0.0);
    }
  }


  if(sim_params.color_over_lifetime == 1.0) {
    particle.color.a = smoothstep(0.0, 0.5, particle.lifetime);
  } 

  particle.position.x += particle.velocity * particle.dir.x * sim_params.delta_time;
  particle.position.y -= particle.velocity * particle.dir.y * sim_params.delta_time;

  let dist = distance(particle.position, sim_params.position);
  particle.color.a -= dist * 0.01;

  
  // emitter_data.prev_position = sim_params.position;
  particles_dst[idx] = particle;
}