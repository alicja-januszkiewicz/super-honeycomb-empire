struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) instance_position: vec2<f32>,
    @location(2) instance_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Static orientation, uploaded once
struct Orientation {
    f0: f32,
    f1: f32,
    f2: f32,
    f3: f32,
};
@group(0) @binding(0)
var<uniform> orientation: Orientation;

// Dynamic per-frame layout
struct LayoutDynamic {
    hex_size: f32,
    origin: vec2<f32>,
    rotation_angle: f32,
};
@group(0) @binding(1)
var<uniform> dynlayout: LayoutDynamic;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let rotation_angle = dynlayout.rotation_angle;
    // let rotation_angle = 30.0 * 3.14159265 / 180.0;
    // let rotation_angle = 88.;

    // Rotate vertex-local coordinates
    let s = sin(rotation_angle);
    let c = cos(rotation_angle);
    let rotated_vertex = vec2<f32>(
        input.position.x * c - input.position.y * s,
        input.position.x * s + input.position.y * c
    );

    // Compute world position
    let world_pos = rotated_vertex * dynlayout.hex_size + input.instance_position + dynlayout.origin;

    out.clip_position = vec4<f32>(world_pos, 0.0, 1.0);
    out.color = input.instance_color;
    return out;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { return in.color; }
