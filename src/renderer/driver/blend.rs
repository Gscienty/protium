#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlendFactor {
    Zero,
    One,
    SourceColor,
    OneMinusSourceColor,
    SourceAlpha,
    OneMinusSourceAlpha,
    DestinationColor,
    OneMinusDestinationColor,
    DestinationAlpha,
    OneMinusDestinationAlpha,
    SourceAlphaSaturated,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlendOperation {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

impl From<BlendFactor> for u32 {
    fn from(val: BlendFactor) -> Self {
        match val {
            BlendFactor::Zero => web_sys::WebGl2RenderingContext::ZERO,
            BlendFactor::One => web_sys::WebGl2RenderingContext::ONE,
            BlendFactor::SourceColor => web_sys::WebGl2RenderingContext::SRC_COLOR,
            BlendFactor::OneMinusSourceColor => {
                web_sys::WebGl2RenderingContext::ONE_MINUS_SRC_COLOR
            }
            BlendFactor::SourceAlpha => web_sys::WebGl2RenderingContext::SRC_ALPHA,
            BlendFactor::OneMinusSourceAlpha => {
                web_sys::WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA
            }
            BlendFactor::DestinationColor => web_sys::WebGl2RenderingContext::DST_COLOR,
            BlendFactor::OneMinusDestinationColor => {
                web_sys::WebGl2RenderingContext::ONE_MINUS_DST_COLOR
            }
            BlendFactor::DestinationAlpha => web_sys::WebGl2RenderingContext::DST_ALPHA,
            BlendFactor::OneMinusDestinationAlpha => {
                web_sys::WebGl2RenderingContext::ONE_MINUS_DST_ALPHA
            }
            BlendFactor::SourceAlphaSaturated => {
                web_sys::WebGl2RenderingContext::SRC_ALPHA_SATURATE
            }
        }
    }
}

impl From<BlendOperation> for u32 {
    fn from(val: BlendOperation) -> Self {
        match val {
            BlendOperation::Add => web_sys::WebGl2RenderingContext::FUNC_ADD,
            BlendOperation::Subtract => web_sys::WebGl2RenderingContext::FUNC_SUBTRACT,
            BlendOperation::ReverseSubtract => {
                web_sys::WebGl2RenderingContext::FUNC_REVERSE_SUBTRACT
            }
            BlendOperation::Min => web_sys::WebGl2RenderingContext::MIN,
            BlendOperation::Max => web_sys::WebGl2RenderingContext::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blend {
    pub(super) factor_source_rgb: BlendFactor,
    pub(super) factor_source_alpha: BlendFactor,
    pub(super) factor_destination_rgb: BlendFactor,
    pub(super) factor_destination_alpha: BlendFactor,
    pub(super) op_rgb: BlendOperation,
    pub(super) op_alpha: BlendOperation,
}

pub const BLEND_SOURCE_OVER: Blend = Blend {
    factor_source_rgb: BlendFactor::One,
    factor_source_alpha: BlendFactor::One,
    factor_destination_rgb: BlendFactor::OneMinusDestinationAlpha,
    factor_destination_alpha: BlendFactor::OneMinusDestinationAlpha,
    op_rgb: BlendOperation::Add,
    op_alpha: BlendOperation::Add,
};

pub const BLEND_CLEAR: Blend = Blend {
    factor_source_rgb: BlendFactor::Zero,
    factor_source_alpha: BlendFactor::Zero,
    factor_destination_rgb: BlendFactor::Zero,
    factor_destination_alpha: BlendFactor::Zero,
    op_rgb: BlendOperation::Add,
    op_alpha: BlendOperation::Add,
};

pub const BLEND_COPY: Blend = Blend {
    factor_source_rgb: BlendFactor::One,
    factor_source_alpha: BlendFactor::One,
    factor_destination_rgb: BlendFactor::Zero,
    factor_destination_alpha: BlendFactor::Zero,
    op_rgb: BlendOperation::Add,
    op_alpha: BlendOperation::Add,
};

pub const BLEND_DEFAULT: Blend = Blend {
    factor_source_rgb: BlendFactor::One,
    factor_source_alpha: BlendFactor::One,
    factor_destination_rgb: BlendFactor::OneMinusSourceAlpha,
    factor_destination_alpha: BlendFactor::OneMinusSourceAlpha,
    op_rgb: BlendOperation::Add,
    op_alpha: BlendOperation::Max,
};
