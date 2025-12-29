mod create_image;
mod create_shader;
mod draw_triangles;
mod read_pixels;
mod write_pixels;

pub(super) use create_image::CreateImage;
pub(super) use create_shader::CreateShader;
pub(super) use draw_triangles::{DrawTriangleSourceArg, DrawTriangles, InternalDrawTriangles};
pub(super) use read_pixels::ReadPixels;
pub(super) use write_pixels::{InternalWritePixels, WritePixels, WritePixelsArg};

#[derive(Debug)]
pub(super) enum Command {
    CreateImage(CreateImage),
    DisposeImage(super::ImageID),
    DrawTriangles(DrawTriangles),
    WritexPixels(WritePixels),
    ReadPixels(ReadPixels),

    CreateShader(CreateShader),
    DisposeShader(super::ShaderID),

    InternalDrawTriangles(InternalDrawTriangles),
    InternalWritePixels(InternalWritePixels),
    Flush(bool),
    FlushWritePixels(super::ImageID),
}
