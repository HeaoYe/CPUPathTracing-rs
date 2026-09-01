use crate::THREAD_POOL;
use std::ops::Range;

#[derive(Default)]
struct ChunkMeta {
    vertex_count: usize,
    normal_count: usize,
    triangle_count: usize,

    vertex_base: usize,
    normal_base: usize,
}

#[derive(Default, Clone, Copy)]
pub struct VertexIndices {
    pub vertex: usize,
    pub normal: usize,
}

#[derive(Default)]
struct Chunk {
    range: Range<usize>,
    meta: ChunkMeta,
    data: ParesdObj,
}

#[derive(Default)]
pub struct ParesdObj {
    pub vertices: Vec<glam::Vec3>,
    pub normals: Vec<glam::Vec3>,
    pub triangles: Vec<[VertexIndices; 3]>,
}

pub fn parse_obj(filename: impl AsRef<std::path::Path>) -> Result<ParesdObj, std::io::Error> {
    let source = std::fs::read_to_string(filename)?;

    let mut parsed_obj = ParesdObj::default();
    let bytes = source.as_bytes();
    let target_chunks = THREAD_POOL.workers() * 4;
    let chunk_size = bytes.len().div_ceil(target_chunks);

    let mut chunks = Vec::with_capacity(target_chunks);
    let mut current = 0;

    while current < bytes.len() {
        let target_end = current + chunk_size;

        let end = if target_end >= bytes.len() {
            bytes.len()
        } else {
            bytes[target_end..]
                .iter()
                .position(|b| b == &b'\n')
                .map_or(bytes.len(), |offset| target_end + offset + 1)
        };

        chunks.push(Chunk {
            range: current..end,
            ..Default::default()
        });
        current = end;
    }

    THREAD_POOL.parallel_for_1d(&mut chunks, |_, chunk| {
        let chunk_source = &source[chunk.range.clone()];

        for line in chunk_source.lines() {
            let mut tokens = line.split_whitespace();
            match tokens.next() {
                Some("v") => chunk.meta.vertex_count += 1,
                Some("vn") => chunk.meta.normal_count += 1,
                Some("f") => match line.split_whitespace().count() - 1 {
                    3 => chunk.meta.triangle_count += 1,
                    4 => chunk.meta.triangle_count += 2,
                    _ => panic!("unsupported obj format"),
                },
                _ => {}
            }
        }
    });

    let mut vertex_base = 0;
    let mut normal_base = 0;
    let mut triangle_base = 0;
    for chunk in chunks.iter_mut() {
        chunk.meta.vertex_base = vertex_base;
        chunk.meta.normal_base = normal_base;
        vertex_base += chunk.meta.vertex_count;
        normal_base += chunk.meta.normal_count;
        triangle_base += chunk.meta.triangle_count;
    }
    parsed_obj.vertices.reserve(vertex_base);
    parsed_obj.normals.reserve(normal_base);
    parsed_obj.triangles.reserve(triangle_base);

    THREAD_POOL.parallel_for_1d(&mut chunks, |_, chunk| {
        let Chunk { meta, data, .. } = chunk;
        data.vertices.reserve(meta.vertex_count);
        data.normals.reserve(meta.normal_count);
        data.triangles.reserve(meta.triangle_count);

        let mut vertex_seen = meta.vertex_base;
        let mut normal_seen = meta.normal_base;

        let chunk_source = &source[chunk.range.clone()];
        for line in chunk_source.lines() {
            let mut tokens = line.split_whitespace();
            match tokens.next() {
                Some("v") => {
                    let vertex = glam::Vec3::new(
                        tokens.next().unwrap().parse().unwrap(),
                        tokens.next().unwrap().parse().unwrap(),
                        tokens.next().unwrap().parse().unwrap(),
                    );
                    data.vertices.push(vertex);
                    vertex_seen += 1;
                }
                Some("vn") => {
                    let normal = glam::Vec3::new(
                        tokens.next().unwrap().parse().unwrap(),
                        tokens.next().unwrap().parse().unwrap(),
                        tokens.next().unwrap().parse().unwrap(),
                    );
                    data.normals.push(normal);
                    normal_seen += 1;
                }
                Some("f") => {
                    let v0 = parse_indices(tokens.next().unwrap(), vertex_seen, normal_seen);
                    let v1 = parse_indices(tokens.next().unwrap(), vertex_seen, normal_seen);
                    let v2 = parse_indices(tokens.next().unwrap(), vertex_seen, normal_seen);

                    match tokens.next() {
                        None => data.triangles.push([v0, v1, v2]),
                        Some(token) => {
                            let v3 = parse_indices(token, vertex_seen, normal_seen);
                            data.triangles.push([v0, v1, v2]);
                            data.triangles.push([v0, v2, v3]);
                        }
                    }
                }
                _ => {}
            }
        }
    });

    for chunk in chunks {
        parsed_obj.vertices.extend(chunk.data.vertices);
        parsed_obj.normals.extend(chunk.data.normals);
        parsed_obj.triangles.extend(chunk.data.triangles);
    }

    Ok(parsed_obj)
}

fn resolve_index(index: isize, seen: usize) -> usize {
    match index {
        1.. => (index - 1) as usize,
        0 => panic!("unexpected obj format"),
        ..=-1 => (seen as isize + index) as usize,
    }
}

fn parse_indices(token: &str, vertex_seen: usize, normal_seen: usize) -> VertexIndices {
    let mut indices = token.split('/');

    let vertex = indices.next().unwrap().parse::<isize>().unwrap();
    let _tex_coord = indices.next();
    let normal = indices.next().unwrap().parse::<isize>().unwrap();

    VertexIndices {
        vertex: resolve_index(vertex, vertex_seen),
        normal: resolve_index(normal, normal_seen),
    }
}
