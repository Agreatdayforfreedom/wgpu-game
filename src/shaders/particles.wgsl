
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

// struct Camera {
//     proj: mat4x4<f32>,
// };
// @group(1) @binding(0) 
// var<uniform> camera: Camera;

// struct ParticleModel {
//     model: mat4x4<f32>,
//     color: vec4<f32>,
//     tex_scale: vec2<f32>,
//     tex_pos: f32
// }
// @group(2) @binding(0)
// var<uniform> particle_model:ParticleModel;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    out.clip_position =  vec4<f32>(model.position, 0.0, 1.0);
    out.tex_coords = vec2<f32>(model.tex_coords.x, model.tex_coords.y);

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;



// @group(0) @binding(0) var t_screen: texture_2d<f32>;
// @group(0) @binding(1) var s_linear: sampler;

// const radius: f32 = 10.0; 
// const shift: f32 = 3.0; 
// const power: f32 = 10.0; 
const offset: f32 = 1.0 / 300.0;
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
var offsets: array<vec2<f32>, 9> = array(
        vec2(-offset,  offset), // top-left
        vec2( 0.0,    offset), // top-center
        vec2( offset,  offset), // top-right
        vec2(-offset,  0.0),   // center-left
        vec2( 0.0,    0.0),   // center-center
        vec2( offset,  0.0),   // center-right
        vec2(-offset, -offset), // bottom-left
        vec2( 0.0,   -offset), // bottom-center
        vec2( offset, -offset)  // bottom-right    
);
    // var glow = vec4(0.0);
    // var c = 0.0;

    // for (var x = -radius; x <= radius; x += 1.0) {
    //     for (var y = -radius; y <= radius; y += 1.0) {
    //         let px_size: vec2<u32> = 1u / textureDimensions(t_diffuse);
    //         let offset = vec2<f32>(x, y) * shift * vec2f(f32(px_size.x), f32(px_size.y));
    //         glow += textureSample(t_diffuse, s_diffuse, in.tex_coords + offset) * vec4<f32>(0.1, 0.6, 0.2, 1.0);
    //         c += 1.0;
    //     }
    // }
    // glow *= power  c;
    // return vec4<f32>(vec3(1.0 - textureSample(t_diffuse, s_diffuse, in.tex_coords).rgb), 1.0)

// 0.0000000000000000000, 0.04416589065853191, 0.0922903086524308425, 0.10497808951021347, 0.0922903086524308425, 0.04416589065853191, 0.0000000000000000000,
// 0.0112445223775533675, 0.10497808951021347, 0.1987116566428735725, 0.40342407932501833, 0.1987116566428735725, 0.10497808951021347, 0.0112445223775533675,
// 0.0000000000000000000, 0.04416589065853191, 0.0922903086524308425, 0.10497808951021347, 0.0922903086524308425, 0.04416589065853191, 0.0000000000000000000
    // var kernel: array<f32, 9> = array(
    //     -1.0, -1.0, -1.0,
    //     -1.0, 8.0, -1.0,
    //     -1.0, -1.0, -1.0,
    // );
    // // var kernel: array<f32, 9> = array(
    // //    1.0 / 16, 2.0 / 16, 1.0 / 16,
    // // 2.0 / 16, 4.0 / 16, 2.0 / 16,
    // // 1.0 / 16, 2.0 / 16, 1.0 / 16 
    // // );
    
    // var sampleTex: array<vec4<f32>, 9>;
    // for(var i = 0; i < 9; i++)
    // {
    //     sampleTex[i] = textureSample(t_diffuse, s_diffuse, in.tex_coords + offsets[i]);
    // }
    // var col = vec3<f32>(0.0);
    // var alpha: f32 = sampleTex[4].a;
    // for(var i = 0; i < 9; i++) {
    //     col += sampleTex[i].rgb * kernel[i];
    // }
    var screenTexture = textureSample(t_diffuse, s_diffuse, in.tex_coords );
    var average = (screenTexture.r + screenTexture.g + screenTexture.b) / 3.0;  
    return vec4f(average, average, average, 1.0);    // return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}


// fn Box4(p0: vec4<f32>, p1: vec4<f32>, p2: vec4<f32>, p3: vec4<f32>) -> vec4<f32> {
//     return (p0 + p1 + p2 + p3) * 0.25;
// }

// // Extracts the pixels we want to blur
// @fragment
// fn ExtractPS(in: VertexOutput) -> @location(0) vec4<f32> {
//     let color = textureSample(t_screen, s_linear, in.tex_coords);
//     let avg = (color.r + color.g + color.b) / 3.0;

//     if (avg > Threshold) {
//         return color * (avg - Threshold) / (1.0 - Threshold);
//     }

//     return vec4<f32>(0.0, 0.0, 0.0, 0.0);
// }

// // Extracts the pixels we want to blur, but considers luminance instead of average rgb
// @fragment
// fn ExtractLuminancePS(in: VertexOutput) -> @location(0) vec4<f32> {
//     let color = textureSample(t_screen, s_linear, in.tex_coords);
//     let luminance = color.r * 0.21 + color.g * 0.72 + color.b * 0.07;

//     if (luminance > Threshold) {
//         return color * (luminance - Threshold) / (1.0 - Threshold);
//     }

//     return vec4<f32>(0.0, 0.0, 0.0, 0.0);
// }

// Downsample to the next mip, blur in the process
// @fragment
// fn DownsamplePS(in: VertexOutput) -> @location(0) vec4<f32> {
//     let offset = vec2<f32>(StreakLength * InverseResolution.x, 1.0 * InverseResolution.y);

//     let c0 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(-2.0, -2.0) * offset);
//     let c1 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(0.0, -2.0) * offset);
//     let c2 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(2.0, -2.0) * offset);
//     let c3 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(-1.0, -1.0) * offset);
//     let c4 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(1.0, -1.0) * offset);
//     let c5 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(-2.0, 0.0) * offset);
//     let c6 = textureSample(t_screen, s_linear, in.tex_coords);
//     let c7 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(2.0, 0.0) * offset);
//     let c8 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(-1.0, 1.0) * offset);
//     let c9 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(1.0, 1.0) * offset);
//     let c10 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(-2.0, 2.0) * offset);
//     let c11 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(0.0, 2.0) * offset);
//     let c12 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(2.0, 2.0) * offset);

//     return Box4(c0, c1, c5, c6) * 0.125 +
//            Box4(c1, c2, c6, c7) * 0.125 +
//            Box4(c5, c6, c10, c11) * 0.125 +
//            Box4(c6, c7, c11, c12) * 0.125 +
//            Box4(c3, c4, c8, c9) * 0.5;
// }

// // Upsample to the former MIP, blur in the process
// @fragment
// fn UpsamplePS(in: VertexOutput) -> @location(0) vec4<f32> {
//     let offset = vec2<f32>(StreakLength * InverseResolution.x, 1.0 * InverseResolution.y) * Radius;

//     let c0 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(-1.0, -1.0) * offset);
//     let c1 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(0.0, -1.0) * offset);
//     let c2 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(1.0, -1.0) * offset);
//     let c3 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(-1.0, 0.0) * offset);
//     let c4 = textureSample(t_screen, s_linear, in.tex_coords);
//     let c5 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(1.0, 0.0) * offset);
//     let c6 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(-1.0, 1.0) * offset);
//     let c7 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(0.0, 1.0) * offset);
//     let c8 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(1.0, 1.0) * offset);

//     // Tent filter 0.0625
//     return 0.0625 * (c0 + 2.0 * c1 + c2 + 2.0 * c3 + 4.0 * c4 + 2.0 * c5 + c6 + 2.0 * c7 + c8) * Strength;
// }

// Upsample to the former MIP, blur in the process, change offset depending on luminance
// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let c4 = textureSample(t_screen, s_linear, in.tex_coords); // Middle one
//     let offset = vec2<f32>(StreakLength * InverseResolution.x, 1.0 * InverseResolution.y) * Radius;

//     let c0 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(-1.0, -1.0) * offset);
//     let c1 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(0.0, -1.0) * offset);
//     let c2 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(1.0, -1.0) * offset);
//     let c3 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(-1.0, 0.0) * offset);
//     let c5 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(1.0, 0.0) * offset);
//     let c6 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(-1.0, 1.0) * offset);
//     let c7 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(0.0, 1.0) * offset);
//     let c8 = textureSample(t_screen, s_linear, in.tex_coords + vec2<f32>(1.0, 1.0) * offset);

//     return 0.0625 * (c0 + 2.0 * c1 + c2 + 2.0 * c3 + 4.0 * c4 + 2.0 * c5 + c6 + 2.0 * c7 + c8) * Strength;
// }

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let texSize = vec2<f32>(textureDimensions(t_diffuse));
//     let uv = in.tex_coords;

//     // Sample original color
//     let originalColor = textureSample(t_diffuse, s_diffuse, uv);

//     // Define the blur radius
//     let blurRadius = 2.0 / texSize;

//     // Accumulate surrounding pixel colors for blur effect
//     var blurredColor = vec4<f32>(0.0);
//     for (var x: i32 = -2; x <=2; x++) {
//         for (var y: i32 = -2; y <= 2; y++) {
//             let offset = vec2<f32>(f32(x), f32(y)) * blurRadius;
//             blurredColor += textureSample(t_diffuse, s_diffuse, uv + offset);
//         }
//     }

//     // Average the accumulated color
//     blurredColor /= 25.0;
//     blurredColor *= in.color;
//     // Combine the original color with the blurred color
//     let threshold = 0.5; // Example threshold for glowing areas
//     let glow = smoothstep(threshold, 1.0, length(originalColor.rgb)) * 1.5;
//     let finalColor = mix(originalColor, blurredColor, glow);

//     return finalColor ;
    
//     // return textureSample(t_diffuse, s_diffuse, in.tex_coords ) *in.color ;
// }