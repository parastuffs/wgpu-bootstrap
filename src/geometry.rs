use crate::default::Vertex;
use cgmath::prelude::*;
use std::f32::consts::PI;

pub fn icosahedron() -> (Vec<Vertex>, Vec<u16>) {
    let f = (1.0+5.0_f32.sqrt())/2.0;
    let positions = vec![
        [-1.0, f, 0.0],
        [1.0, f, 0.0,],
        [-1.0, -f, 0.0,],
        [1.0, -f, 0.0,],
        [0.0, -1.0, f,],
        [0.0, 1.0, f,],
        [0.0, -1.0, -f,],
        [0.0, 1.0, -f,],
        [f, 0.0, -1.0,],
        [f, 0.0, 1.0,],
        [-f, 0.0, -1.0,],
        [-f, 0.0, 1.0],
    ];

    let vertices: Vec<Vertex> = positions.iter().map(|position| {
        let v = cgmath::Vector3::from(position.to_owned());
        let normal = v.normalize();
        let colatitude = normal.y.acos();
        let longitude = normal.x.atan2(normal.z);
        Vertex {position: normal.into(), normal: normal.into(), tangent: [longitude.cos(), 0.0, -longitude.sin()], tex_coords: [(longitude + PI)/PI, colatitude/PI]}
    }).collect();


    let indices: Vec<u16> = vec![
        0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11,
        11, 10, 2, 5, 11, 4, 1, 5, 9, 7, 1, 8, 10, 7, 6,
        3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9,
        9, 8, 1, 4, 9, 5, 2, 4, 11, 6, 2, 10, 8, 6, 7
    ];

    (vertices, indices)
}

fn compute_triangle_tangent(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> (cgmath::Vector3<f32>, cgmath::Vector3<f32>) {
    let v1_position = cgmath::Vector3::from(v1.position);
    let v2_position = cgmath::Vector3::from(v2.position);
    let v3_position = cgmath::Vector3::from(v3.position);

    let v1_tex_coords = cgmath::Vector2::from(v1.tex_coords);
    let v2_tex_coords = cgmath::Vector2::from(v2.tex_coords);
    let v3_tex_coords = cgmath::Vector2::from(v3.tex_coords);

    let position_1_2 = v2_position - v1_position;
    let position_1_3 = v3_position - v1_position;

    let tex_coords_1_2 = v2_tex_coords - v1_tex_coords;
    let tex_coords_1_3 = v3_tex_coords - v1_tex_coords;

    let tangent = (tex_coords_1_3.y * position_1_2 - tex_coords_1_2.y * position_1_3).normalize();
    let bitangent = (-tex_coords_1_3.x * position_1_2 + tex_coords_1_2.x * position_1_3).normalize();

    (tangent, bitangent)
}

pub fn compute_tangent_vectors(vertices: &mut Vec<Vertex>, indices: &Vec<u16>) {
    let mut tangents: Vec<cgmath::Vector3<f32>> = vertices.iter().map(|_vertex| {
        cgmath::Vector3 {x: 0.0, y: 0.0, z: 0.0}
    }).collect();

    indices
        .as_slice()
        .chunks_exact(3)
        .map(|item| (item[0] as usize, item[1]as usize, item[2] as usize))
        .for_each( |indices| {
            let (tangent, _) = compute_triangle_tangent(
                &vertices[indices.0],
                &vertices[indices.1],
                &vertices[indices.2]
            );

            tangents[indices.0] = tangents[indices.0] + tangent;
            tangents[indices.1] = tangents[indices.1] + tangent;
            tangents[indices.2] = tangents[indices.2] + tangent;
        });
    
    vertices.iter_mut().enumerate().for_each(|(i, mut vertex)| {
        vertex.tangent = tangents[i].normalize().into();
    });
}