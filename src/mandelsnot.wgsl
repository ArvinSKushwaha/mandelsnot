// vim: ft=wgsl

struct VertexOut {
    @location(0) uv: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

struct MandelbrotFrame {
    min: vec2<f32>,
    max: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> frame: MandelbrotFrame;

var<private> v_positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var uv = v_positions[v_idx] / 2. + 0.5;

    return VertexOut(uv, vec4<f32>(v_positions[v_idx], 0., 1.));
}

fn get_color(i: u32, iterations: u32, z: vec2<f32>) -> vec4<f32> {
    var alpha: f32 = f32(i) - log2(log(pow(length(z), 2.)));

    let color_scaling = 0u;
    let factor = 1.;
    let shift = 0.;

    if color_scaling == 0u {
        alpha = alpha / f32(i);
    } else if color_scaling == 1u {
        alpha = log(alpha);
    } else if color_scaling == 2u {
        alpha = 1. / (1. + alpha);
    } else if color_scaling == 3u {
        alpha = sqrt(alpha);
    }

    alpha = alpha * factor;
    alpha = alpha + shift;

    let red = cos(alpha);
    let blue = 1. + sin(-alpha);
    let green = (red + blue) * 0.667;

    return vec4(red, green, blue, 1.);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let c = in.uv * frame.max + (vec2<f32>(1., 1.) - in.uv) * frame.min;

    var z = vec2<f32>(0., 0.);
    var iterations = 0u;

    for(; iterations < 1000u; iterations = iterations + 1u){

        z = vec2<f32>(z.x * z.x - z.y * z.y, 2. * z.x * z.y);
        z = z + c;

        if length(z) >= 2. {
            break;
        }

        iterations = iterations + 1u;
    }

    return get_color(iterations, 1000u, z);
}
