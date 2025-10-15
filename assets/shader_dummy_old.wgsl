struct Uniforms {
    mvp: mat4x4<f32>,
    hex_size: f32, // add this
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) instance_position: vec2<f32>,
    @location(2) instance_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = input.position + input.instance_position;
    let pos = vec4<f32>(world_pos, 0.0, 1.0);
    out.clip_position = uniforms.mvp * pos;
    out.color = input.instance_color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

