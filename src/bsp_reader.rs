use std::fs::File;
use std::io::Read;
use std::io::Cursor;
use std::str;
use byteorder::{LittleEndian, ReadBytesExt};

pub struct BSPReader {
    pub data: Vec<u8>,
    marker: usize,
}

#[derive(Debug)]
pub struct Header {
    pub magic: String,
    pub version: i32,
    pub direntries: Direntries,
}

#[derive(Debug)]
pub struct Direntry {
    pub offset: i32,
    pub length: i32,
}

#[derive(Debug)]
pub struct Direntries {
    pub entities: Direntry,
    pub textures: Direntry,
    pub planes: Direntry,
    pub nodes: Direntry,
    pub leafs: Direntry,
    pub leaffaces: Direntry,
    pub leafbrushes: Direntry,
    pub models: Direntry,
    pub brushes: Direntry,
    pub brushsides: Direntry,
    pub vertexes: Direntry,
    pub meshverts: Direntry,
    pub effects: Direntry,
    pub faces: Direntry,
    pub lightmaps: Direntry,
    pub lightvols: Direntry,
    pub visdata: Direntry,
}

#[derive(Debug)]
pub struct Texture {
    pub name: String,
    pub flags: i32,
    pub contents: i32,
}

#[derive(Debug)]
pub struct Plane {
    pub normal: [f32; 3],
    pub dist: f32,
}

#[derive(Debug)]
pub struct Node {
    pub plane: i32,
    pub children: [i32; 2],
    pub mins: [i32; 3],
    pub maxs: [i32; 3],
}

#[derive(Debug)]
pub struct Leaf {
    pub cluster: i32,
    pub area: i32,
    pub mins: [i32; 3],
    pub maxs: [i32; 3],
    pub leafface: i32,
    pub n_leaffaces: i32,
    pub leafbrush: i32,
    pub n_leafbrushes: i32,
}

#[derive(Debug)]
pub struct Model {
    pub mins: [f32; 3],
    pub maxs: [f32; 3],
    pub face: i32,
    pub n_faces: i32,
    pub brush: i32,
    pub n_brushes: i32,
}

#[derive(Debug)]
pub struct Brush {
    pub brushside: i32,
    pub n_brushsides: i32,
    pub texture: i32,
}

#[derive(Debug)]
pub struct Brushside {
    pub plane: i32,
    pub texture: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texcoord: [[f32; 2]; 2],
    pub normal: [f32; 3],
    pub color: [u8; 4],
}

#[derive(Debug)]
pub struct Effect {
    pub name: String,
    pub brush: i32,
    pub unknown: i32,
}

#[derive(Debug)]
pub struct Face {
    pub texture: i32,
    pub effect: i32,
    pub f_type: i32,
    pub vertex: i32,
    pub n_vertexes: i32,
    pub meshvert: i32,
    pub n_meshverts: i32,
    pub lm_index: i32,
    pub lm_start: [i32; 2],
    pub lm_size: [i32; 2],
    pub lm_origin: [f32; 3],
    pub lm_vecs: [[f32; 3]; 2],
    pub normal: [f32; 3],
    pub size: [i32; 2],
}

impl BSPReader {
    pub fn new(path: &str) -> BSPReader {
        let mut f = File::open(path).unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf);
        BSPReader {
            data: buf,
            marker: 0,
        }
    }

    pub fn jump(&mut self, offset: usize) -> &mut BSPReader {
        self.marker = offset;
        self
    }

    pub fn read_ubyte(&mut self) -> u8 {
        let data = self.data[self.marker];
        self.marker += 1;
        data
    }

    pub fn read_int(&mut self) -> i32 {
        let bytes = [self.read_ubyte(), self.read_ubyte(), self.read_ubyte(), self.read_ubyte()];
        Cursor::new(&bytes[..]).read_i32::<LittleEndian>().unwrap()
    }

    pub fn read_string(&mut self, length: usize) -> String {
        let bytes = &(self.data)[self.marker..self.marker + length];
        self.marker += length;
        str::from_utf8(bytes).unwrap().to_owned()
    }

    pub fn read_float(&mut self) -> f32 {
        let bytes = [self.read_ubyte(), self.read_ubyte(), self.read_ubyte(), self.read_ubyte()];
        Cursor::new(&bytes[..]).read_f32::<LittleEndian>().unwrap()
    }

    pub fn read_direntry(&mut self) -> Direntry {
        Direntry {
            offset: self.read_int(),
            length: self.read_int(),
        }
    }

    pub fn read_direntries(&mut self) -> Direntries {
        Direntries {
            entities: self.read_direntry(),
            textures: self.read_direntry(),
            planes: self.read_direntry(),
            nodes: self.read_direntry(),
            leafs: self.read_direntry(),
            leaffaces: self.read_direntry(),
            leafbrushes: self.read_direntry(),
            models: self.read_direntry(),
            brushes: self.read_direntry(),
            brushsides: self.read_direntry(),
            vertexes: self.read_direntry(),
            meshverts: self.read_direntry(),
            effects: self.read_direntry(),
            faces: self.read_direntry(),
            lightmaps: self.read_direntry(),
            lightvols: self.read_direntry(),
            visdata: self.read_direntry(),
        }
    }

    pub fn read_header(&mut self) -> Header {
        Header {
            magic: self.read_string(4),
            version: self.read_int(),
            direntries: self.read_direntries(),
        }
    }

    pub fn read_entities(&mut self, direntries: &Direntries) -> String {
        self.jump(direntries.entities.offset as usize)
            .read_string(direntries.entities.length as usize)
    }

    pub fn read_list<T, F>(&mut self, direntry: &Direntry, entry_size: i32, read: F) -> Vec<T>
        where F: Fn(&mut BSPReader) -> T
    {
        self.jump(direntry.offset as usize);
        let mut list = Vec::new();
        let entries = direntry.length / entry_size;
        for _ in 0..entries {
            list.push(read(self));
        }
        list
    }

    pub fn read_textures(&mut self, direntries: &Direntries) -> Vec<Texture> {
        self.read_list(&direntries.textures, 64 + 4 + 4, |r| {
            Texture {
                name: r.read_string(64),
                flags: r.read_int(),
                contents: r.read_int(),
            }
        })
    }

    pub fn read_planes(&mut self, direntries: &Direntries) -> Vec<Plane> {
        self.read_list(&direntries.planes, 3 * 4 + 4, |r| {
            Plane {
                normal: [r.read_float(), r.read_float(), r.read_float()],
                dist: r.read_float(),
            }
        })
    }

    pub fn read_nodes(&mut self, direntries: &Direntries) -> Vec<Node> {
        self.read_list(&direntries.nodes, 4 + 2 * 4 + 3 * 4 + 3 * 4, |r| {
            Node {
                plane: r.read_int(),
                children: [r.read_int(), r.read_int()],
                mins: [r.read_int(), r.read_int(), r.read_int()],
                maxs: [r.read_int(), r.read_int(), r.read_int()],
            }
        })
    }

    pub fn read_leafs(&mut self, direntries: &Direntries) -> Vec<Leaf> {
        self.read_list(&direntries.leafs, 12 * 4, |r| {
            Leaf {
                cluster: r.read_int(),
                area: r.read_int(),
                mins: [r.read_int(), r.read_int(), r.read_int()],
                maxs: [r.read_int(), r.read_int(), r.read_int()],
                leafface: r.read_int(),
                n_leaffaces: r.read_int(),
                leafbrush: r.read_int(),
                n_leafbrushes: r.read_int(),
            }
        })
    }

    pub fn read_leaffaces(&mut self, direntries: &Direntries) -> Vec<i32> {
        self.read_list(&direntries.leaffaces, 4, |r| r.read_int())
    }

    pub fn read_leafbrushes(&mut self, direntries: &Direntries) -> Vec<i32> {
        self.read_list(&direntries.leafbrushes, 4, |r| r.read_int())
    }

    pub fn read_models(&mut self, direntries: &Direntries) -> Vec<Model> {
        self.read_list(&direntries.models, 10 * 4, |r| {
            Model {
                mins: [r.read_float(), r.read_float(), r.read_float()],
                maxs: [r.read_float(), r.read_float(), r.read_float()],
                face: r.read_int(),
                n_faces: r.read_int(),
                brush: r.read_int(),
                n_brushes: r.read_int(),
            }
        })
    }

    pub fn read_brushes(&mut self, direntries: &Direntries) -> Vec<Brush> {
        self.read_list(&direntries.brushes, 3 * 4, |r| {
            Brush {
                brushside: r.read_int(),
                n_brushsides: r.read_int(),
                texture: r.read_int(),
            }
        })
    }

    pub fn read_brushsides(&mut self, direntries: &Direntries) -> Vec<Brushside> {
        self.read_list(&direntries.brushsides, 2 * 4, |r| {
            Brushside {
                plane: r.read_int(),
                texture: r.read_int(),
            }
        })
    }

    pub fn read_vertexes(&mut self, direntries: &Direntries) -> Vec<Vertex> {
        self.read_list(&direntries.vertexes, 14 * 4, |r| {
            Vertex {
                position: [r.read_float(), r.read_float(), r.read_float()],
                texcoord: [[r.read_float(), r.read_float()], [r.read_float(), r.read_float()]],
                normal: [r.read_float(), r.read_float(), r.read_float()],
                color: [r.read_ubyte(), r.read_ubyte(), r.read_ubyte(), r.read_ubyte()],
            }
        })
    }

    pub fn read_meshverts(&mut self, direntries: &Direntries) -> Vec<i32> {
        self.read_list(&direntries.meshverts, 4, |r| r.read_int())
    }

    pub fn read_effects(&mut self, direntries: &Direntries) -> Vec<Effect> {
        self.read_list(&direntries.effects, 64 + 2 * 4, |r| {
            Effect {
                name: r.read_string(64),
                brush: r.read_int(),
                unknown: r.read_int(),
            }
        })
    }

    pub fn read_faces(&mut self, direntries: &Direntries) -> Vec<Face> {
        self.read_list(&direntries.faces, 26 * 4, |r| {
            Face {
                texture: r.read_int(),
                effect: r.read_int(),
                f_type: r.read_int(),
                vertex: r.read_int(),
                n_vertexes: r.read_int(),
                meshvert: r.read_int(),
                n_meshverts: r.read_int(),
                lm_index: r.read_int(),
                lm_start: [r.read_int(), r.read_int()],
                lm_size: [r.read_int(), r.read_int()],
                lm_origin: [r.read_float(), r.read_float(), r.read_float()],
                lm_vecs: [[r.read_float(), r.read_float(), r.read_float()],
                          [r.read_float(), r.read_float(), r.read_float()]],
                normal: [r.read_float(), r.read_float(), r.read_float()],
                size: [r.read_int(), r.read_int()],
            }
        })
    }
}
