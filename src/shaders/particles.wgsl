const PI: f32 = 3.14159265358;

// the circle shape modifies the 
const CIRCLE_SHAPE: u32 = 0;
const CONE_SHAPE: u32 = 1;

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

fn direction_vector(angle: f32) -> vec2f {
  let x = cos(radians(angle));
  let y = sin(radians(angle));

  return normalize(vec2f(x, y));
}

// cone just emits the paricles from the base. 
fn cone(angle: f32, arc: f32, x: f32, y: f32) -> vec4f {  
  let cone = vec2f(
    cos(gen_range(
      radians(degrees(angle) - arc / 2.0), 
      radians(degrees(angle) + arc / 2.0))), 
    sin(gen_range(
      radians(degrees(angle) - arc / 2.0),  
      radians(degrees(angle) + arc / 2.0)
    )));
  
  return vec4f(x, y, cone);
}

fn circle(invocation_id: u32, radius: f32, x: f32, y: f32, emit_from_edge: u32, mode: u32) -> vec4f {
  var length = length(gen_range(0.0, exp2(radius))); // this indeed emit from any position in the circle.
  if (emit_from_edge == 1) {
    length = length(exp2(radius));
  }
  var degree = 0.0;
  // 1 = loop
  if (mode == 1) {
    degree = f32( invocation_id % 360);
  } else { // 2 and default = random
    degree = degrees(gen_range(0.0, 1.0) * 2.0 * PI);
  }
  let dir = direction_vector(degree);
  let dx = x + length * dir.x;
  let dy = y + length * dir.y;

  return vec4f(dx, dy, dir.x, dir.y);
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

struct Cone {
  arc: f32,
  angle: f32,
}

struct Circle {
  radius: f32,
  emit_from_edge: u32,
}

struct SimulationParams {
  interval: f32,
  total: f32,
  position: vec2<f32>,
  color: vec4<f32>,
  dir: vec2<f32>,
  color_over_lifetime: f32,
  rate_over_distance: f32,
  distance_traveled: f32,
  lifetime_factor: f32,
  start_speed: f32,
  // determine if they will respawn. 0 = false, 1 = true
  infinite: u32,
  // determine the mode emision, if it is 1 then the particles are emitted in a loop, and randomly if it's 0.
  mode: u32,
  shape_selected: u32,
  cone: Cone,
  circle: Circle,
}

struct Uniforms {
  delta_time: f32,
  time: f32,
}

//shapes
//circle
// circle_radius
// randomize

// cone
// arc,
// randomize

struct Particle {
  position: vec2<f32>,
  dir: vec2<f32>,
  color: vec4<f32>,
  velocity: f32,
  lifetime: f32,
  actived: f32,
}


@binding(0) @group(0) var<storage, read_write> particles_dst : array<Particle>;
@binding(1) @group(0) var<storage> sim_params_groups: array<SimulationParams>;
@binding(2) @group(0) var<uniform> uniforms: Uniforms;

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
  
  var particle: Particle = particles_dst[idx];
  init_rand(idx, vec4f(f32(idx), uniforms.delta_time, particle.position.x, particle.position.y));


  
  particle.lifetime -= uniforms.delta_time;

  let local_idx = idx - (cumulative_total - u32(sim_params.total));


  if (idx > cumulative_total) {
    particle.color = vec4(0.0);
  }

  // here the particle is initialized

  if (particle.actived == 0.0 && idx < cumulative_total) {
  // if (particle.lifetime < 0.0){
      // particle.lifetime = 1.0 * sim_params.lifetime_factor;
      particle.actived = 1.0;
      var position = vec2f(0.0, 0.0);
      var dir = vec2f(0.0, 0.0);

      switch sim_params.shape_selected {
        case CIRCLE_SHAPE: {
          let position_dir = circle(
            local_idx,
            sim_params.circle.radius, 
            sim_params.position.x, 
            sim_params.position.y, 
            sim_params.circle.emit_from_edge,
            sim_params.mode
          );

          position = position_dir.xy;
          dir = position_dir.zw;
        }
        case CONE_SHAPE: {
          let position_dir = cone(
            sim_params.cone.angle,
            sim_params.cone.arc,
            sim_params.position.x, 
            sim_params.position.y
          );
          position = position_dir.xy;
          dir = position_dir.zw;
        }
        default: {
          let position_dir = circle(
            local_idx,
            5.0, 
            sim_params.position.x, 
            sim_params.position.y, 
            0u,
            sim_params.mode
          );
          position = position_dir.xy;
          dir = position_dir.zw;
        }
      }
      particle.position.x = position.x;
      particle.position.y = position.y ;
      particle.dir.x = dir.x;
      particle.dir.y = dir.y;
      particle.lifetime = rand() * sim_params.lifetime_factor;
      

      particle.velocity = rand() * sim_params.start_speed;      

      if(sim_params.rate_over_distance == -1.0 
      || sim_params.distance_traveled > 1.0
       ) {
        particle.color = sim_params.color;
      } else {
        particle.color = vec4f(0.0, 0.0, 0.0, 0.0);
      }
  }

  if (particle.lifetime < 0.0 && particle.actived == 1.0) {
    if (sim_params.infinite == 1) {
      particle.actived = 0.0;
    }
  }

  if(sim_params.color_over_lifetime == 1.0) {
    particle.color.a = smoothstep(0.0, 0.5, particle.lifetime);
  } 

  particle.position.x += particle.velocity * particle.dir.x * uniforms.delta_time;
  particle.position.y -= particle.velocity * particle.dir.y * uniforms.delta_time;
  
  particles_dst[idx] = particle;
}