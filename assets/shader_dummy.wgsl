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

    // Convert cube coordinates to pixel offsets
    let q = input.instance_position.x;
    let r = input.instance_position.y;
    
    // Apply orientation matrix (f0..f3) and hex_size
    let x_offset = dynlayout.hex_size * (orientation.f0 * q + orientation.f1 * r);
    let y_offset = dynlayout.hex_size * (orientation.f2 * q + orientation.f3 * r);

    let world_pos = input.position * dynlayout.hex_size + vec2<f32>(x_offset, y_offset) + dynlayout.origin;

    // Optional: per-frame rotation
    let c = cos(dynlayout.rotation_angle);
    let s = sin(dynlayout.rotation_angle);
    let rotated_pos = vec2<f32>(
        world_pos.x * c - world_pos.y * s,
        world_pos.x * s + world_pos.y * c
    );

    out.clip_position = vec4<f32>(rotated_pos, 0.0, 1.0);
    out.color = input.instance_color;
    return out;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { return in.color; }
