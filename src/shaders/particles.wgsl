
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

struct ParticleModel {
    model: mat4x4<f32>,
    color: vec4<f32>,
    tex_scale: vec2<f32>,
    tex_pos: f32
}
@group(2) @binding(0)
var<uniform> particle_model:ParticleModel;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    out.clip_position = camera.proj * particle_model.model * vec4<f32>(model.position, 0.0, 1.0);
    out.tex_coords = vec2<f32>(model.tex_coords.x, model.tex_coords.y + particle_model.tex_pos) * particle_model.tex_scale;
    out.color = particle_model.color;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texSize = vec2<f32>(textureDimensions(t_diffuse));
    let uv = in.tex_coords;

    // Sample original color
    let originalColor = textureSample(t_diffuse, s_diffuse, uv);

    // Define the blur radius
    let blurRadius = 2.0 / texSize;

    // Accumulate surrounding pixel colors for blur effect
    var blurredColor = vec4<f32>(0.0);
    for (var x: i32 = -2; x <=2; x++) {
        for (var y: i32 = -2; y <= 2; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * blurRadius;
            blurredColor += textureSample(t_diffuse, s_diffuse, uv + offset);
        }
    }

    // Average the accumulated color
    blurredColor /= 25.0;
    blurredColor *= in.color;
    // Combine the original color with the blurred color
    let threshold = 0.5; // Example threshold for glowing areas
    let glow = smoothstep(threshold, 1.0, length(originalColor.rgb)) * 1.5;
    let finalColor = mix(originalColor, blurredColor, glow);

    return finalColor ;
    
    // return textureSample(t_diffuse, s_diffuse, in.tex_coords ) *in.color ;
}