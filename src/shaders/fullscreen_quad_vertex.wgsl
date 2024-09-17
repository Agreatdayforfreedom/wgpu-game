var<private> pos : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2(-1.0, -1.0), 
    vec2(-1.0, 3.0), 
    vec2(3.0, -1.0)
);

struct VertexOutput {
    @builtin(position) position : vec4<f32>,
    @location(0) texCoord : vec2<f32>,
};

@vertex
fn vs_main( @builtin(vertex_index) vertexIndex : u32) -> VertexOutput {
    var output: VertexOutput;

    output.position = vec4(pos[vertexIndex], 1.0, 1.0);
    output.texCoord = pos[vertexIndex] * 0.5 + 0.5;
    output.texCoord.y = output.texCoord.y * -1.0;

    return output;
}