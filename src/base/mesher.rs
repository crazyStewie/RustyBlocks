use raylib;

pub mod greed_mesher {
    use crate::base::voxel::ChunkData;
    use std::ops::Deref;

    pub fn generate_mesh(chunk_data : ChunkData) -> raylib::models::Mesh {
         unsafe { std::mem::transmute(generate_chunk_mesh(chunk_data))}
    }
    unsafe fn generate_chunk_mesh(chunk_data: ChunkData) -> raylib::ffi::Mesh {
        let mut vertices = Box::new([0f32; 3*3]);
        let mut texcoords = Box::new([0f32; 3*2]);
        let mut normals = Box::new([0f32; 3*3]);
        //FIXME change type to c_ushort to match raylib ffi
        let mut indices = Box::new([0u16; 3*1]);

        *vertices = [
            1.0f32, 0.0, 0.0,
            0.0, 0.0, 1.0,
            0.0, 0.0, 0.0];
        *texcoords = [
            1.0f32, 0.0,
            0.0, 1.0,
            0.0, 0.0];
        *normals = [
            0.0f32, 1.0, 0.0,
            0.0f32, 1.0, 0.0,
            0.0f32, 1.0, 0.0];
        *indices = [0u16, 1, 2];

        raylib::ffi::Mesh {
            vertexCount: 3,
            triangleCount: 1,
            vertices: vertices.as_mut_ptr(),
            texcoords: texcoords.as_mut_ptr(),
            texcoords2: std::ptr::null_mut(),
            normals: normals.as_mut_ptr(),
            tangents: std::ptr::null_mut(),
            colors: std::ptr::null_mut(),
            indices: indices.as_mut_ptr(),
            animVertices: std::ptr::null_mut(),
            animNormals: std::ptr::null_mut(),
            boneIds: std::ptr::null_mut(),
            boneWeights: std::ptr::null_mut(),
            vaoId: 0u32,
            vboId: [0u32; 7usize]
        }
    }
}