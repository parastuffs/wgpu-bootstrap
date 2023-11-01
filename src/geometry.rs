use cgmath::{prelude::*, Vector2, Vector3};
use std::{collections::HashMap, f32::consts::PI};

pub fn icosphere(order: u32) -> (Vec<Vector3<f32>>, Vec<u32>) {
    let f = (1.0 + 5.0_f32.sqrt()) / 2.0;
    let mut positions = vec![
        cgmath::Vector3 {
            x: -1.0,
            y: f,
            z: 0.0,
        },
        cgmath::Vector3 {
            x: 1.0,
            y: f,
            z: 0.0,
        },
        cgmath::Vector3 {
            x: -1.0,
            y: -f,
            z: 0.0,
        },
        cgmath::Vector3 {
            x: 1.0,
            y: -f,
            z: 0.0,
        },
        cgmath::Vector3 {
            x: 0.0,
            y: -1.0,
            z: f,
        },
        cgmath::Vector3 {
            x: 0.0,
            y: 1.0,
            z: f,
        },
        cgmath::Vector3 {
            x: 0.0,
            y: -1.0,
            z: -f,
        },
        cgmath::Vector3 {
            x: 0.0,
            y: 1.0,
            z: -f,
        },
        cgmath::Vector3 {
            x: f,
            y: 0.0,
            z: -1.0,
        },
        cgmath::Vector3 {
            x: f,
            y: 0.0,
            z: 1.0,
        },
        cgmath::Vector3 {
            x: -f,
            y: 0.0,
            z: -1.0,
        },
        cgmath::Vector3 {
            x: -f,
            y: 0.0,
            z: 1.,
        },
    ];

    let mut indices: Vec<u32> = vec![
        0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 11, 10, 2, 5, 11, 4, 1, 5, 9, 7, 1, 8, 10,
        7, 6, 3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 9, 8, 1, 4, 9, 5, 2, 4, 11, 6, 2, 10, 8,
        6, 7,
    ];

    let mut v: u32 = 12;
    let mut mid_cache = HashMap::new();

    let mut add_mid_point = |a: u32, b: u32| -> u32 {
        // Cantor's Pairing Function
        let key: u32 = ((a + b) * (a + b + 1) / 2) + std::cmp::min(a, b);

        mid_cache
            .entry(key)
            .or_insert_with(|| {
                let i = v;
                positions.push((positions[a as usize] + positions[b as usize]) / 2.0);
                v += 1;
                i
            })
            .to_owned()
    };

    let mut subdivide = |indices: Vec<u32>| -> Vec<u32> {
        let mut res = Vec::new();
        for k in 0..indices.len() / 3 {
            let k = k * 3;
            let v1 = indices[k];
            let v2 = indices[k + 1];
            let v3 = indices[k + 2];
            let a = add_mid_point(v1, v2);
            let b = add_mid_point(v2, v3);
            let c = add_mid_point(v3, v1);
            res.push(v1);
            res.push(a);
            res.push(c);
            res.push(v2);
            res.push(b);
            res.push(a);
            res.push(v3);
            res.push(c);
            res.push(b);
            res.push(a);
            res.push(b);
            res.push(c);
        }
        res
    };

    for _ in 0..order {
        indices = subdivide(indices);
    }

    let positions: Vec<Vector3<f32>> = positions
        .iter()
        .map(|position| position.normalize())
        .collect();

    (positions, indices)
}

pub fn compute_spherical_uv(position: Vector3<f32>) -> Vector2<f32> {
    let u = (position.x.atan2(position.z) + PI) / PI;
    let v = (position.y / position.magnitude()).acos() / PI;

    Vector2 { x: u, y: v }
}

pub fn compute_triangle_normal(
    v1: &Vector3<f32>,
    v2: &Vector3<f32>,
    v3: &Vector3<f32>,
) -> Vector3<f32> {
    let position_1_2 = v2 - v1;
    let position_1_3 = v3 - v1;

    position_1_2.cross(position_1_3).normalize()
}

pub fn compute_triangle_tangent(
    pos1: &Vector3<f32>,
    uv1: &Vector2<f32>,
    pos2: &Vector3<f32>,
    uv2: &Vector2<f32>,
    pos3: &Vector3<f32>,
    uv3: &Vector2<f32>,
) -> (cgmath::Vector3<f32>, cgmath::Vector3<f32>) {
    let position_1_2 = pos2 - pos1;
    let position_1_3 = pos3 - pos1;

    let tex_coords_1_2 = uv2 - uv1;
    let tex_coords_1_3 = uv3 - uv1;

    let tangent = (tex_coords_1_3.y * position_1_2 - tex_coords_1_2.y * position_1_3).normalize();
    let bitangent =
        (-tex_coords_1_3.x * position_1_2 + tex_coords_1_2.x * position_1_3).normalize();

    (tangent, bitangent)
}

pub fn compute_normal_vectors(
    positions: &mut [Vector3<f32>],
    indices: &Vec<u32>,
) -> Vec<Vector3<f32>> {
    let mut normals: Vec<cgmath::Vector3<f32>> = positions
        .iter()
        .map(|_| cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        })
        .collect();

    indices
        .as_slice()
        .chunks_exact(3)
        .map(|item| (item[0] as usize, item[1] as usize, item[2] as usize))
        .for_each(|indices| {
            let normal = compute_triangle_normal(
                &positions[indices.0],
                &positions[indices.1],
                &positions[indices.2],
            );

            normals[indices.0] += normal;
            normals[indices.1] += normal;
            normals[indices.2] += normal;
        });

    normals.iter().map(|n| n.normalize()).collect()
}

pub fn compute_tangent_vectors(
    positions: &mut [Vector3<f32>],
    uvs: Vec<Vector2<f32>>,
    normals: Vec<Vector3<f32>>,
    indices: &Vec<u32>,
) -> Vec<Vector3<f32>> {
    let mut tangents: Vec<Vector3<f32>> = positions
        .iter()
        .map(|_| cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        })
        .collect();

    indices
        .as_slice()
        .chunks_exact(3)
        .map(|item| (item[0] as usize, item[1] as usize, item[2] as usize))
        .for_each(|indices| {
            let (tangent, _) = compute_triangle_tangent(
                &positions[indices.0],
                &uvs[indices.0],
                &positions[indices.1],
                &uvs[indices.1],
                &positions[indices.2],
                &uvs[indices.2],
            );

            tangents[indices.0] += tangent;
            tangents[indices.1] += tangent;
            tangents[indices.2] += tangent;
        });

    (0..tangents.len())
        .map(|i| {
            let tangent = tangents[i].normalize();
            let tangent = tangent - normals[i].dot(tangent) * normals[i];
            tangent.normalize()
        })
        .collect()
}

pub fn compute_line_list(triangle_list: Vec<u32>) -> Vec<u32> {
    let mut lines = HashMap::new();

    let mut add_line = |v1: u32, v2: u32| {
        let a = std::cmp::min(v1, v2);
        let b = std::cmp::max(v1, v2);

        // Cantor's Pairing Function
        let key: u32 = ((a as u32 + b as u32) * (a as u32 + b as u32 + 1) / 2) + a as u32;
        lines.entry(key).or_insert_with(|| (a, b));
    };

    for i in 0..triangle_list.len() / 3 {
        let i = i * 3;
        let v1 = triangle_list[i];
        let v2 = triangle_list[i + 1];
        let v3 = triangle_list[i + 2];
        add_line(v1, v2);
        add_line(v2, v3);
        add_line(v3, v1);
    }

    let mut res = Vec::new();
    for (_key, value) in lines {
        res.push(value.0);
        res.push(value.1);
    }

    res
}
