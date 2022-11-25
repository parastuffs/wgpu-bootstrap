use crate::default::Vertex;
use cgmath::prelude::*;
use std::{f32::consts::PI, collections::HashMap};

pub fn icosahedron(order: u32) -> (Vec<Vertex>, Vec<u16>) {
    let f = (1.0+5.0_f32.sqrt())/2.0;
    let mut positions = vec![
        cgmath::Vector3 {x: -1.0, y: f, z: 0.0},
        cgmath::Vector3 {x: 1.0, y: f, z: 0.0},
        cgmath::Vector3 {x: -1.0, y: -f, z: 0.0},
        cgmath::Vector3 {x: 1.0, y: -f, z: 0.0},
        cgmath::Vector3 {x: 0.0, y: -1.0, z: f},
        cgmath::Vector3 {x: 0.0, y: 1.0, z: f},
        cgmath::Vector3 {x: 0.0, y: -1.0, z: -f},
        cgmath::Vector3 {x: 0.0, y: 1.0, z: -f},
        cgmath::Vector3 {x: f, y: 0.0, z: -1.0},
        cgmath::Vector3 {x: f, y: 0.0, z: 1.0},
        cgmath::Vector3 {x: -f, y: 0.0, z: -1.0},
        cgmath::Vector3 {x: -f, y: 0.0, z: 1.},
    ];

    let mut indices: Vec<u16> = vec![
        0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11,
        11, 10, 2, 5, 11, 4, 1, 5, 9, 7, 1, 8, 10, 7, 6,
        3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9,
        9, 8, 1, 4, 9, 5, 2, 4, 11, 6, 2, 10, 8, 6, 7
    ];

    let mut v: u16 = 12;
    let mut mid_cache = HashMap::new();

    let mut add_mid_point = |a: u16, b: u16| -> u16 {
        let a: u32 = a.into();
        let b: u32 = b.into();
        // Cantor's Pairing Function
        let key: u32 = ((a + b) * (a + b + 1) / 2) + std::cmp::min(a, b); 

        mid_cache.entry(key).or_insert_with(|| {
            let i = v;
            positions.push((positions[a as usize] + positions[b as usize]) / 2.0);
            v += 1;
            i
        }).to_owned()
    };

    let mut subdivide = |indices: Vec<u16>| -> Vec<u16> {
        let mut res = Vec::new();
        for k in 0..indices.len()/3 {
            let k = k * 3;
            let v1 = indices[k];
            let v2 = indices[k + 1];
            let v3 = indices[k + 2];
            let a = add_mid_point(v1, v2);
            let b = add_mid_point(v2, v3);
            let c = add_mid_point(v3, v1);
            res.push(v1); res.push(a); res.push(c);
            res.push(v2); res.push(b); res.push(a);
            res.push(v3); res.push(c); res.push(b);
            res.push(a); res.push(b); res.push(c);
        }
        res
    };

    for _ in 0..order {
        indices = subdivide(indices);
    }

    let vertices: Vec<Vertex> = positions.iter().map(|position| {
        //let v = cgmath::Vector3::from(position.to_owned());
        let normal = position.normalize();
        let colatitude = normal.y.acos();
        let longitude = normal.x.atan2(normal.z);
        Vertex {position: normal.into(), normal: normal.into(), tangent: [longitude.cos(), 0.0, -longitude.sin()], tex_coords: [(longitude + PI)/PI, colatitude/PI]}
    }).collect();

    (vertices, indices)
}

fn compute_triangle_normal(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> cgmath::Vector3<f32> {
    let v1_position = cgmath::Vector3::from(v1.position);
    let v2_position = cgmath::Vector3::from(v2.position);
    let v3_position = cgmath::Vector3::from(v3.position);

    let position_1_2 = v2_position - v1_position;
    let position_1_3 = v3_position - v1_position;

    position_1_2.cross(position_1_3).normalize()
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

pub fn compute_normal_vectors(vertices: &mut Vec<Vertex>, indices: &Vec<u16>) {

    let mut normals: Vec<cgmath::Vector3<f32>> = vertices.iter().map(|_vertex| {
        cgmath::Vector3 {x: 0.0, y: 0.0, z: 0.0}
    }).collect();
    
    indices
        .as_slice()
        .chunks_exact(3)
        .map(|item| (item[0] as usize, item[1]as usize, item[2] as usize))
        .for_each( |indices| {
            let normal = compute_triangle_normal(
                &vertices[indices.0],
                &vertices[indices.1],
                &vertices[indices.2]
            );

            normals[indices.0] = normals[indices.0] + normal;
            normals[indices.1] = normals[indices.1] + normal;
            normals[indices.2] = normals[indices.2] + normal;
        });
    
    vertices.iter_mut().enumerate().for_each(|(i, mut vertex)| {
        vertex.normal = normals[i].normalize().into();
    });
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
        let tangent = tangents[i].normalize();
        let normal = cgmath::Vector3::from(vertex.normal);
        let tangent = tangent - normal.dot(tangent)*normal;
        vertex.tangent = tangent.normalize().into();
    });
}

pub fn compute_line_list(triangle_list: Vec<u16>) -> Vec<u16> {
    let mut lines = HashMap::new();

    let mut add_line = |v1: u16, v2: u16| {
        let a = std::cmp::min(v1, v2);
        let b = std::cmp::max(v1, v2);

        // Cantor's Pairing Function
        let key: u32 = ((a as u32 + b as u32) * (a as u32 + b as u32 + 1) / 2) + a as u32; 
        lines.entry(key).or_insert_with(||{
            (a, b)
        });
    };

    for i in 0..triangle_list.len()/3 {
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
