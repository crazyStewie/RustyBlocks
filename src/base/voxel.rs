use raylib::prelude::*;
use specs::prelude::*;

pub const CHUNK_SIZE : usize = 64;

type BlockData = bool;

pub struct ChunkData([BlockData; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]);

impl ChunkData {
    pub fn new() -> Self {
        return ChunkData([false; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]);
    }

    pub fn from_array(arr : [bool; CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE]) -> ChunkData {
        return ChunkData(arr);
    }

    pub fn set(&mut self,value: BlockData, x: usize, y:usize, z:usize) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            panic!("Invalid chunk position: Tried to access position ({}, {}, {}) on chunk of size {}",
                x, y, z, CHUNK_SIZE);
        }

        self.0[Self::position_from_coordinates(x, y, z)] = value;
    }

    pub fn get(&mut self, x: usize, y:usize, z:usize) -> BlockData {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            panic!("Invalid chunk position: Tried to access position ({}, {}, {}) on chunk of size {}",
                   x, y, z, CHUNK_SIZE);
        }

        return self.0[Self::position_from_coordinates(x, y, z)];
    }

    pub fn position_from_coordinates(x: usize, y:usize, z:usize) -> usize {
        return CHUNK_SIZE*(CHUNK_SIZE*x + y) + z;
    }

    pub fn coordinates_from_position(pos: usize) ->(usize, usize, usize) {
        let x = pos%(CHUNK_SIZE*CHUNK_SIZE);
        let y = (pos - x*CHUNK_SIZE)%CHUNK_SIZE;
        let z = (pos - y*CHUNK_SIZE - x*CHUNK_SIZE*CHUNK_SIZE)%CHUNK_SIZE;
        return (x, y, z);
    }
    //TODO
    //unsafe fn generate_mesh(&self) -> Mesh {
    //    return mesh;
    //}
}

struct ChunkComponent {
    chunk_data : ChunkData,
    must_rebuild: bool,
}

impl Component for ChunkComponent {
    type Storage = VecStorage<Self>;
}

struct VoxelSystem{

}