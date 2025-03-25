use std::ffi::CString;
use super::RcEffect;

#[derive(Clone, Copy)]
enum VecComps {
    /** 2 */ Two,
    /** 3 */ Three,
    /** 4 */ Four,
}

impl VecComps {
    #[inline]
    const fn as_byte(self) -> u8 {
        match self {
            Self::Two => b'2',
            Self::Three => b'3',
            Self::Four => b'4',
        }
    }
}

#[derive(Clone, Copy)]
enum SamplerType {
    /// `gsampler1D` (`GL_TEXTURE_1D`): 1D texture
    Sampler1D,

    /// `gsampler2D` (`GL_TEXTURE_2D`): 2D texture
    Sampler2D,

    /// `gsampler3D` (`GL_TEXTURE_3D`): 3D texture
    Sampler3D,

    /// `gsamplerCube` (`GL_TEXTURE_CUBE_MAP`): Cubemap Texture
    SamplerCube,

    /// `gsampler2DRect` (`GL_TEXTURE_RECTANGLE`): Rectangle Texture
    Sampler2DRect,

    /// `gsampler1DArray` (`GL_TEXTURE_1D_ARRAY`): 1D Array Texture
    Sampler1DArray,

    /// `gsampler2DArray` (`GL_TEXTURE_2D_ARRAY`): 2D Array Texture
    Sampler2DArray,

    /// `gsamplerCubeArray` (`GL_TEXTURE_CUBE_MAP_ARRAY`): Cubemap Array Texture \
    /// (requires GL 4.0 or `ARB_texture_cube_map_array`)
    SamplerCubeArray,

    /// `gsamplerBuffer` (`GL_TEXTURE_BUFFER`): Buffer Texture
    SamplerBuffer,

    /// `gsampler2DMS` (`GL_TEXTURE_2D_MULTISAMPLE`): Multisample Texture
    Sampler2DMS,

    /// `gsampler2DMSArray` (`GL_TEXTURE_2D_MULTISAMPLE_ARRAY`): Multisample Array Texture
    Sampler2DMSArray,

    // Shadow samplers

    /// `sampler1DShadow` (`GL_TEXTURE_1D`)
    Sampler1DShadow,

    /// `sampler2DShadow` (`GL_TEXTURE_2D`)
    Sampler2DShadow,

    /// `samplerCubeShadow` (`GL_TEXTURE_CUBE_MAP`)
    SamplerCubeShadow,

    /// `sampler2DRectShadow` (`GL_TEXTURE_RECTANGLE`)
    Sampler2DRectShadow,

    /// `sampler1DArrayShadow` (`GL_TEXTURE_1D_ARRAY`)
    Sampler1DArrayShadow,

    /// `sampler2DArrayShadow` (`GL_TEXTURE_2D_ARRAY`)
    Sampler2DArrayShadow,

    /// `samplerCubeArrayShadow` (`GL_TEXTURE_CUBE_MAP_ARRAY`)
    SamplerCubeArrayShadow,
}

impl SamplerType {
    #[inline]
    const fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Sampler1D => b"sampler1D",
            Self::Sampler2D => b"sampler2D",
            Self::Sampler3D => b"sampler3D",
            Self::SamplerCube => b"samplerCube",
            Self::Sampler2DRect => b"sampler2DRect",
            Self::Sampler1DArray => b"sampler1DArray",
            Self::Sampler2DArray => b"sampler2DArray",
            Self::SamplerCubeArray => b"samplerCubeArray",
            Self::SamplerBuffer => b"samplerBuffer",
            Self::Sampler2DMS => b"sampler2DMS",
            Self::Sampler2DMSArray => b"sampler2DMSArray",
            Self::Sampler1DShadow => b"sampler1DShadow",
            Self::Sampler2DShadow => b"sampler2DShadow",
            Self::SamplerCubeShadow => b"samplerCubeShadow",
            Self::Sampler2DRectShadow => b"sampler2DRectShadow",
            Self::Sampler1DArrayShadow => b"sampler1DArrayShadow",
            Self::Sampler2DArrayShadow => b"sampler2DArrayShadow",
            Self::SamplerCubeArrayShadow => b"samplerCubeArrayShadow",
        }
    }
}

#[derive(Clone, Copy)]
enum StorageType {
    /// `bool`: conditional type, values may be either true or false
    Bool,

    /// `int`: a signed, two's complement, 32-bit integer
    Int,

    /// `uint`: an unsigned 32-bit integer
    UInt,

    /// `float`: an IEEE-754 single-precision floating point number
    FLoat,

    /// `double`: an IEEE-754 double-precision floating-point number
    Double,

    /// `bvecn`: a vector of booleans
    BVec(VecComps),

    /// `ivecn`: a vector of signed integers
    IVec(VecComps),

    /// `uvecn`: a vector of unsigned integers
    UVec(VecComps),

    /// `vecn`: a vector of single-precision floating-point numbers
    Vec(VecComps),

    /// `dvecn`: a vector of double-precision floating-point numbers
    DVec(VecComps),

    /// `matnxm`: A matrix with *n* columns and *m* rows (examples: `mat2x2`, `mat4x3`). Note that this is backward from convention in mathematics!
    MatNxM { cols: VecComps, rows: VecComps },

    /// `matn`: Common shorthand for matnxn: a square matrix with *n* columns and *n* rows.
    MatN(VecComps),

    /// Texture access is not as simple as reading a value from a memory address.
    /// Filtering and other processes are applied to textures, and how texture coordinates are interpreted can be part of the texture access operation.
    /// For these reason, texture access is somewhat complicated.
    ///
    /// The sampler type is an opaque GLSL type that represents a texture bound to the OpenGL context.
    /// There are many sampler types, one for each type of texture (`2D`, `2D_ARRAY`, etc).
    /// Samplers can only access textures of the proper type.
    Sampler(SamplerType),
}

impl StorageType {
    #[inline]
    const fn as_bytes(self) -> &'static [u8] {
        use VecComps::*;
        match self {
            Self::Bool => b"bool",
            Self::Int => b"int",
            Self::UInt => b"uint",
            Self::FLoat => b"float",
            Self::Double => b"double",
            Self::BVec(Two)   => b"bvec2",
            Self::IVec(Two)   => b"ivec2",
            Self::UVec(Two)   => b"uvec2",
            Self::Vec (Two)   =>  b"vec2",
            Self::DVec(Two)   => b"dvec2",
            Self::MatN(Two)   =>  b"mat2",
            Self::BVec(Three) => b"bvec3",
            Self::IVec(Three) => b"ivec3",
            Self::UVec(Three) => b"uvec3",
            Self::Vec (Three) =>  b"vec3",
            Self::DVec(Three) => b"dvec3",
            Self::MatN(Three) =>  b"mat3",
            Self::BVec(Four)  => b"bvec4",
            Self::IVec(Four)  => b"ivec4",
            Self::UVec(Four)  => b"uvec4",
            Self::Vec (Four)  =>  b"vec4",
            Self::DVec(Four)  => b"dvec4",
            Self::MatN(Four)  =>  b"mat4",
            Self::MatNxM { cols: Two,   rows: Two   } => b"mat2x2",
            Self::MatNxM { cols: Two,   rows: Three } => b"mat2x3",
            Self::MatNxM { cols: Two,   rows: Four  } => b"mat2x4",
            Self::MatNxM { cols: Three, rows: Two   } => b"mat3x2",
            Self::MatNxM { cols: Three, rows: Three } => b"mat3x3",
            Self::MatNxM { cols: Three, rows: Four  } => b"mat3x4",
            Self::MatNxM { cols: Four,  rows: Two   } => b"mat4x2",
            Self::MatNxM { cols: Four,  rows: Three } => b"mat4x3",
            Self::MatNxM { cols: Four,  rows: Four  } => b"mat4x4",
            Self::Sampler(t) => t.as_bytes(),
        }
    }
}

#[derive(Clone, Copy)]
enum FieldCategory {
    Input,
    Output,
    Uniform,
    Constant,
}

impl FieldCategory {
    #[inline]
    const fn as_bytes(self) -> &'static [u8] {
        match self {
            FieldCategory::Input => b"in",
            FieldCategory::Output => b"out",
            FieldCategory::Uniform => b"uniform",
            FieldCategory::Constant => b"const",
        }
    }
}

struct Field {
    pub id: u32,
    pub cat: FieldCategory,
    pub ty: StorageType,
}

struct Arg {
    pub id: u32,
    pub ty: StorageType,
}

struct FuncDef {

}

struct Func {
    pub id: u32,
    pub ret: Option<StorageType>,
    pub args: Vec<Arg>,
    pub def: FuncDef,
}

#[derive(Clone, Copy)]
enum GlVersion {
    V330,
}

impl GlVersion {
    #[inline]
    const fn as_bytes(self) -> &'static [u8] {
        match self {
            GlVersion::V330 => b"330",
        }
    }
}

pub struct EffectCode {
    version: GlVersion,
    idents: Vec<Box<[u8]>>,
    fields: Vec<Field>,
    funcs: Vec<Func>,
    main: FuncDef,
}

impl EffectCode {
    pub fn to_cstr(&self) -> Result<CString, std::ffi::FromVecWithNulError> {
        const VERSION_PREFIX: &[u8] = b"#version ";
        const MAIN_SIGNATURE: &[u8] = b"void main() {\n";

        let Self { version, idents, fields, funcs, main } = self;

        let mut capacity = 1; // +1 for null terminator

        capacity += VERSION_PREFIX.len();
        capacity += version.as_bytes().len();
        capacity += 1; // '\n'

        for field in &fields[..] {
            capacity += field.cat.as_bytes().len();
            capacity += 1; // ' '
            capacity += field.ty.as_bytes().len();
            capacity += 1; // ' '
            capacity += idents[field.id as usize].len();
            capacity += 2; // ";\n"
        }

        for func in &funcs[..] {
            capacity += func.ret.map(StorageType::as_bytes).unwrap_or(b"void").len();
            capacity += 1; // ' '
            capacity += idents[func.id as usize].len();
            capacity += 1; // '('
            capacity += 2 * (func.args.len() - 1); // ", "
            capacity += func.args.len(); // ' '
            for arg in &func.args {
                capacity += arg.ty.as_bytes().len();
                capacity += idents[arg.id as usize].len();
            }
            capacity += 4; // ") {\n"
            capacity += 2; // "}\n"
        }

        capacity += MAIN_SIGNATURE.len();
        capacity += 1; // '}'

        let mut s = Vec::with_capacity(capacity);

        s.extend_from_slice(VERSION_PREFIX);
        s.extend_from_slice(version.as_bytes());
        s.push(b'\n');

        for field in &fields[..] {
            s.extend_from_slice(field.cat.as_bytes());
            s.push(b' ');
            s.extend_from_slice(field.ty.as_bytes());
            s.push(b' ');
            s.extend_from_slice(&idents[field.id as usize]);
            s.extend_from_slice(b";\n");
        }

        for func in &funcs[..] {
            s.extend_from_slice(func.ret.map(StorageType::as_bytes).unwrap_or(b"void"));
            s.push(b' ');
            s.extend_from_slice(&idents[func.id as usize]);
            s.push(b'(');
            let mut is_first = true;
            for arg in &func.args[..] {
                if is_first { is_first = false; } else { s.extend_from_slice(b", "); }
                s.extend_from_slice(arg.ty.as_bytes());
                s.push(b' ');
                s.extend_from_slice(&idents[arg.id as usize]);
            }
            s.extend_from_slice(b") {\n");
            // ...
            s.extend_from_slice(b"}\n");
        }

        s.extend_from_slice(MAIN_SIGNATURE);
        // ...
        s.push(b'}');

        s.push(b'\0');

        CString::from_vec_with_nul(s)
    }
}

#[cfg(test)]
mod effect_code_tests {
    use super::*;

    #[test]
    fn test0() {
        let code = EffectCode {
            version: GlVersion::V330,
            idents: vec![
                Box::new(*b"apple"),
                Box::new(*b"orange"),
                Box::new(*b"banana"),
                Box::new(*b"mango"),
            ],
            fields: vec![
                Field {
                    id: 0, // apple
                    cat: FieldCategory::Input,
                    ty: StorageType::Vec(VecComps::Two),
                },
            ],
            funcs: vec![
                Func {
                    id: 1, // orange
                    ret: None, // void
                    args: vec![
                        Arg {
                            id: 2, // banana
                            ty: StorageType::Int,
                        },
                        Arg {
                            id: 3, // mango
                            ty: StorageType::Sampler(SamplerType::Sampler3D),
                        },
                    ],
                    def: FuncDef { },
                },
            ],
            main: FuncDef { },
        };

        let s = code.to_cstr().unwrap();
        const BAR: &str = "------------------------------------------------------";
        println!("DEBUG:\n{s:?}\n");
        let s = s.to_string_lossy();
        println!("DISPLAY:\n{BAR}\n{s}\n{BAR}");
    }
}

pub struct EffectEditor {
    effect: RcEffect,
}
