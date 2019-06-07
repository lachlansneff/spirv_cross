use crate::bindings as br;
use crate::{compiler, spirv, ErrorCode};
use std::collections::BTreeMap;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr;

/// A MSL target.
#[derive(Debug, Clone)]
pub enum Target {}

pub struct TargetData {
    vertex_attribute_overrides: Vec<br::SPIRV_CROSS_NAMESPACE::MSLVertexAttr>,
    resource_binding_overrides: Vec<br::SPIRV_CROSS_NAMESPACE::MSLResourceBinding>,
}

impl spirv::Target for Target {
    type Data = TargetData;
}

/// Location of a vertex attribute to override
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct VertexAttributeLocation(pub u32);

/// Format of the vertex attribute
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Format {
    Other,
    Uint8,
    Uint16,
}

impl Format {
    fn as_raw(&self) -> br::SPIRV_CROSS_NAMESPACE::MSLVertexFormat {
        use self::Format::*;
        use crate::bindings::root::SPIRV_CROSS_NAMESPACE::MSLVertexFormat as R;
        match self {
            Other => R::MSL_VERTEX_FORMAT_OTHER,
            Uint8 => R::MSL_VERTEX_FORMAT_UINT8,
            Uint16 => R::MSL_VERTEX_FORMAT_UINT16,
        }
    }
}

/// Vertex attribute description for overriding
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct VertexAttribute {
    pub buffer_id: u32,
    pub offset: u32,
    pub stride: u32,
    pub step: spirv::VertexAttributeStep,
    pub format: Format,
    pub built_in: Option<spirv::BuiltIn>,
}

/// Location of a resource binding to override
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ResourceBindingLocation {
    pub stage: spirv::ExecutionModel,
    pub desc_set: u32,
    pub binding: u32,
}

/// Resource binding description for overriding
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct ResourceBinding {
    pub buffer_id: u32,
    pub texture_id: u32,
    pub sampler_id: u32,
}

/// A MSL shader platform.
#[repr(u8)]
#[allow(non_snake_case, non_camel_case_types)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Platform {
    iOS = 0,
    macOS = 1,
}

/// A MSL shader model version.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Version {
    V1_0,
    V1_1,
    V1_2,
    V2_0,
    V2_1,
}

impl Version {
    fn as_raw(self) -> u32 {
        use self::Version::*;
        match self {
            V1_0 => 10000,
            V1_1 => 10100,
            V1_2 => 10200,
            V2_0 => 20000,
            V2_1 => 20100,
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CompilerVertexOptions {
    pub invert_y: bool,
    pub transform_clip_space: bool,
}

impl Default for CompilerVertexOptions {
    fn default() -> Self {
        CompilerVertexOptions {
            invert_y: false,
            transform_clip_space: false,
        }
    }
}

/// MSL compiler options.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CompilerOptions {
    /// The target platform.
    pub platform: Platform,
    /// The target MSL version.
    pub version: Version,
    /// Vertex compiler options.
    pub vertex: CompilerVertexOptions,
    /// The buffer index to use for swizzle.
    pub swizzle_buffer_index: u32,
    // The buffer index to use for indirect params.
    pub indirect_params_buffer_index: u32,
    /// The buffer index to use for output.
    pub output_buffer_index: u32,
    /// The buffer index to use for patch output.
    pub patch_output_buffer_index: u32,
    /// The buffer index to use for tessellation factor.
    pub tessellation_factor_buffer_index: u32,
    /// The buffer index to use for buffer size.
    pub buffer_size_buffer_index: u32,
    /// Whether the built-in point size should be enabled.
    pub enable_point_size_builtin: bool,
    /// Whether rasterization should be enabled.
    pub enable_rasterization: bool,
    /// Whether to capture output to buffer.
    pub capture_output_to_buffer: bool,
    /// Whether to swizzle texture samples.
    pub swizzle_texture_samples: bool,
    /// Whether to place the origin of tessellation domain shaders in the lower left.
    pub tessellation_domain_origin_lower_left: bool,
    /// Whether to enable use of argument buffers (only compatible with MSL 2.0).
    pub enable_argument_buffers: bool,
    /// Whether to pad fragment output to have at least the number of components as the render pass.
    pub pad_fragment_output_components: bool,
    /// MSL resource bindings overrides.
    pub resource_binding_overrides: BTreeMap<ResourceBindingLocation, ResourceBinding>,
    /// MSL vertex attribute overrides.
    pub vertex_attribute_overrides: BTreeMap<VertexAttributeLocation, VertexAttribute>,
    
}

impl CompilerOptions {
    fn as_raw(&self) -> br::ScMslCompilerOptions {
        br::ScMslCompilerOptions {
            vertex_invert_y: self.vertex.invert_y,
            vertex_transform_clip_space: self.vertex.transform_clip_space,
            platform: self.platform as _,
            version: self.version.as_raw(),
            enable_point_size_builtin: self.enable_point_size_builtin,
            disable_rasterization: !self.enable_rasterization,
            swizzle_buffer_index: self.swizzle_buffer_index,
            indirect_params_buffer_index: self.indirect_params_buffer_index,
            shader_output_buffer_index: self.output_buffer_index,
            shader_patch_output_buffer_index: self.patch_output_buffer_index,
            shader_tess_factor_buffer_index: self.tessellation_factor_buffer_index,
            buffer_size_buffer_index: self.buffer_size_buffer_index,
            capture_output_to_buffer: self.capture_output_to_buffer,
            swizzle_texture_samples: self.swizzle_texture_samples,
            tess_domain_origin_lower_left: self.tessellation_domain_origin_lower_left,
            argument_buffers: self.enable_argument_buffers,
            pad_fragment_output_components: self.pad_fragment_output_components,
        }
    }
}

impl Default for CompilerOptions {
    fn default() -> Self {
        CompilerOptions {
            platform: Platform::macOS,
            version: Version::V1_2,
            vertex: CompilerVertexOptions::default(),
            swizzle_buffer_index: 30,
            indirect_params_buffer_index: 29,
            output_buffer_index: 28,
            patch_output_buffer_index: 27,
            tessellation_factor_buffer_index: 26,
            buffer_size_buffer_index: 25,
            enable_point_size_builtin: true,
            enable_rasterization: true,
            capture_output_to_buffer: false,
            swizzle_texture_samples: false,
            tessellation_domain_origin_lower_left: false,
            enable_argument_buffers: false,
            pad_fragment_output_components: false,
            resource_binding_overrides: Default::default(),
            vertex_attribute_overrides: Default::default(),
        }
    }
}

impl<'a> spirv::Parse<Target> for spirv::Ast<Target> {
    fn parse(module: &spirv::Module) -> Result<Self, ErrorCode> {
        let mut sc_compiler = ptr::null_mut();
        unsafe {
            check!(br::sc_internal_compiler_msl_new(
                &mut sc_compiler,
                module.words.as_ptr(),
                module.words.len(),
            ));
        }

        Ok(spirv::Ast {
            compiler: compiler::Compiler {
                sc_compiler,
                target_data: TargetData {
                    resource_binding_overrides: Vec::new(),
                    vertex_attribute_overrides: Vec::new(),
                },
                has_been_compiled: false,
            },
            target_type: PhantomData,
        })
    }
}

impl spirv::Compile<Target> for spirv::Ast<Target> {
    type CompilerOptions = CompilerOptions;

    /// Set MSL compiler specific compilation settings.
    fn set_compiler_options(&mut self, options: &CompilerOptions) -> Result<(), ErrorCode> {
        let raw_options = options.as_raw();
        unsafe {
            check!(br::sc_internal_compiler_msl_set_options(
                self.compiler.sc_compiler,
                &raw_options,
            ));
        }

        self.compiler.target_data.resource_binding_overrides.clear();
        self.compiler.target_data.resource_binding_overrides.extend(
            options.resource_binding_overrides.iter().map(|(loc, res)| {
                br::SPIRV_CROSS_NAMESPACE::MSLResourceBinding {
                    stage: loc.stage.as_raw(),
                    desc_set: loc.desc_set,
                    binding: loc.binding,
                    msl_buffer: res.buffer_id,
                    msl_texture: res.texture_id,
                    msl_sampler: res.sampler_id,
                }
            }),
        );

        self.compiler.target_data.vertex_attribute_overrides.clear();
        self.compiler.target_data.vertex_attribute_overrides.extend(
            options.vertex_attribute_overrides.iter().map(|(loc, vat)| {
                br::SPIRV_CROSS_NAMESPACE::MSLVertexAttr {
                    location: loc.0,
                    msl_buffer: vat.buffer_id,
                    msl_offset: vat.offset,
                    msl_stride: vat.stride,
                    per_instance: match vat.step {
                        spirv::VertexAttributeStep::Vertex => false,
                        spirv::VertexAttributeStep::Instance => true,
                    },
                    format: vat.format.as_raw(),
                    builtin: spirv::built_in_as_raw(vat.built_in),
                }
            }),
        );

        Ok(())
    }

    /// Generate MSL shader from the AST.
    fn compile(&mut self) -> Result<String, ErrorCode> {
        self.compile_internal()
    }
}

impl spirv::Ast<Target> {
    fn compile_internal(&self) -> Result<String, ErrorCode> {
        let vat_overrides = &self.compiler.target_data.vertex_attribute_overrides;
        let res_overrides = &self.compiler.target_data.resource_binding_overrides;
        unsafe {
            let mut shader_ptr = ptr::null();
            check!(br::sc_internal_compiler_msl_compile(
                self.compiler.sc_compiler,
                &mut shader_ptr,
                vat_overrides.as_ptr(),
                vat_overrides.len(),
                res_overrides.as_ptr(),
                res_overrides.len(),
            ));
            let shader = match CStr::from_ptr(shader_ptr).to_str() {
                Ok(v) => v.to_owned(),
                Err(_) => return Err(ErrorCode::Unhandled),
            };
            check!(br::sc_internal_free_pointer(
                shader_ptr as *mut std::os::raw::c_void
            ));
            Ok(shader)
        }
    }

    pub fn is_rasterization_enabled(&self) -> Result<bool, ErrorCode> {
        unsafe {
            let mut is_disabled = false;
            check!(br::sc_internal_compiler_msl_get_is_rasterization_disabled(
                self.compiler.sc_compiler,
                &mut is_disabled
            ));
            Ok(!is_disabled)
        }
    }
}
