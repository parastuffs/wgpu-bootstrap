struct CameraUniform {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = model.position;
    out.normal = model.normal;
    out.color = model.color;
    out.clip_position = camera.proj * camera.view * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light = vec3<f32>(4.0, 5.0, 4.0);
    let light_dir = normalize(light - in.position);
    let normal = normalize(in.normal);
    let shading = clamp(dot(light_dir, normal), 0.1, 1.0);
    return vec4<f32>(in.color * shading, 1.0);
}
