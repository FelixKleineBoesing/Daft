use std::borrow::Cow;
use std::io::Write;
use std::vec;

use image::{ColorType, DynamicImage, ImageBuffer};

use crate::datatypes::{
    logical::ImageArray, BinaryArray, DataType, Field, ImageFormat, ImageMode, StructArray,
};
use crate::error::{DaftError, DaftResult};
use image::{Luma, LumaA, Rgb, Rgba};

use super::as_arrow::AsArrow;
use num_traits::FromPrimitive;

use std::ops::Deref;

#[allow(clippy::upper_case_acronyms, dead_code)]
#[derive(Debug)]
pub enum DaftImageBuffer<'a> {
    L(ImageBuffer<Luma<u8>, Cow<'a, [u8]>>),
    LA(ImageBuffer<LumaA<u8>, Cow<'a, [u8]>>),
    RGB(ImageBuffer<Rgb<u8>, Cow<'a, [u8]>>),
    RGBA(ImageBuffer<Rgba<u8>, Cow<'a, [u8]>>),
    L16(ImageBuffer<Luma<u16>, Cow<'a, [u16]>>),
    LA16(ImageBuffer<LumaA<u16>, Cow<'a, [u16]>>),
    RGB16(ImageBuffer<Rgb<u16>, Cow<'a, [u16]>>),
    RGBA16(ImageBuffer<Rgba<u16>, Cow<'a, [u16]>>),
    RGB32F(ImageBuffer<Rgb<f32>, Cow<'a, [f32]>>),
    RGBA32F(ImageBuffer<Rgba<f32>, Cow<'a, [f32]>>),
}

macro_rules! with_method_on_image_buffer {
    (
    $key_type:expr, $method: ident
) => {{
        use DaftImageBuffer::*;

        match $key_type {
            L(img) => img.$method(),
            LA(img) => img.$method(),
            RGB(img) => img.$method(),
            RGBA(img) => img.$method(),
            L16(img) => img.$method(),
            LA16(img) => img.$method(),
            RGB16(img) => img.$method(),
            RGBA16(img) => img.$method(),
            RGB32F(img) => img.$method(),
            RGBA32F(img) => img.$method(),
        }
    }};
}

impl<'a> DaftImageBuffer<'a> {
    pub fn height(&self) -> u32 {
        with_method_on_image_buffer!(self, height)
    }

    pub fn width(&self) -> u32 {
        with_method_on_image_buffer!(self, width)
    }

    pub fn as_u8_slice(&'a self) -> &'a [u8] {
        use DaftImageBuffer::*;
        match self {
            L(img) => img.as_raw(),
            LA(img) => img.as_raw(),
            RGB(img) => img.as_raw(),
            RGBA(img) => img.as_raw(),
            _ => unimplemented!("unimplemented {self:?}"),
        }
    }

    pub fn color(&self) -> ColorType {
        use DaftImageBuffer::*;
        match self {
            L(..) => ColorType::L8,
            LA(..) => ColorType::La8,
            RGB(..) => ColorType::Rgb8,
            RGBA(..) => ColorType::Rgba8,
            L16(..) => ColorType::L16,
            LA16(..) => ColorType::La16,
            RGB16(..) => ColorType::Rgb16,
            RGBA16(..) => ColorType::Rgba16,
            RGB32F(..) => ColorType::Rgb32F,
            RGBA32F(..) => ColorType::Rgba32F,
        }
    }

    pub fn mode(&self) -> ImageMode {
        self.color().try_into().unwrap()
    }

    pub fn decode(bytes: &[u8]) -> DaftResult<Self> {
        image::load_from_memory(bytes)
            .map(|v| v.into())
            .map_err(|e| DaftError::ValueError(format!("Decoding image from bytes failed: {}", e)))
    }

    pub fn thumbnail(&self, w: u32, h: u32) -> Self {
        use DaftImageBuffer::*;
        match self {
            L(imgbuf) => {
                let result = image::imageops::thumbnail(imgbuf, w, h);
                DaftImageBuffer::L(image_buffer_vec_to_cow(result))
            }
            LA(imgbuf) => {
                let result = image::imageops::thumbnail(imgbuf, w, h);
                DaftImageBuffer::LA(image_buffer_vec_to_cow(result))
            }
            RGB(imgbuf) => {
                let result = image::imageops::thumbnail(imgbuf, w, h);
                DaftImageBuffer::RGB(image_buffer_vec_to_cow(result))
            }
            RGBA(imgbuf) => {
                let result = image::imageops::thumbnail(imgbuf, w, h);
                DaftImageBuffer::RGBA(image_buffer_vec_to_cow(result))
            }
            _ => unimplemented!("Mode {self:?} not implemented"),
        }
    }

    pub fn encode(&self, image_format: ImageFormat, out: &mut Vec<u8>) -> DaftResult<()> {
        let mut writer = std::io::BufWriter::new(std::io::Cursor::new(out));
        image::write_buffer_with_format(
            &mut writer,
            self.as_u8_slice(),
            self.width(),
            self.height(),
            self.color(),
            image::ImageFormat::from(image_format),
        )
        .map_err(|e| {
            DaftError::ValueError(format!(
                "Encoding image into file format {} failed: {}",
                image_format, e
            ))
        })?;
        writer.flush().map_err(|e| {
            DaftError::ValueError(format!(
                "Encoding image into file format {} failed: {}",
                image_format, e
            ))
        })
    }

    pub fn resize(&self, w: u32, h: u32) -> Self {
        use DaftImageBuffer::*;
        match self {
            L(imgbuf) => {
                let result =
                    image::imageops::resize(imgbuf, w, h, image::imageops::FilterType::Triangle);
                DaftImageBuffer::L(image_buffer_vec_to_cow(result))
            }
            LA(imgbuf) => {
                let result =
                    image::imageops::resize(imgbuf, w, h, image::imageops::FilterType::Triangle);
                DaftImageBuffer::LA(image_buffer_vec_to_cow(result))
            }
            RGB(imgbuf) => {
                let result =
                    image::imageops::resize(imgbuf, w, h, image::imageops::FilterType::Triangle);
                DaftImageBuffer::RGB(image_buffer_vec_to_cow(result))
            }
            RGBA(imgbuf) => {
                let result =
                    image::imageops::resize(imgbuf, w, h, image::imageops::FilterType::Triangle);
                DaftImageBuffer::RGBA(image_buffer_vec_to_cow(result))
            }
            _ => unimplemented!("Mode {self:?} not implemented"),
        }
    }
}

fn image_buffer_vec_to_cow<'a, P, T>(input: ImageBuffer<P, Vec<T>>) -> ImageBuffer<P, Cow<'a, [T]>>
where
    P: image::Pixel<Subpixel = T>,
    Vec<T>: Deref<Target = [P::Subpixel]>,
    T: ToOwned + std::clone::Clone,
    [T]: ToOwned,
{
    let h = input.height();
    let w = input.width();
    let owned: Cow<[T]> = input.into_raw().into();
    ImageBuffer::from_raw(w, h, owned).unwrap()
}

impl<'a> From<DynamicImage> for DaftImageBuffer<'a> {
    fn from(dyn_img: DynamicImage) -> Self {
        match dyn_img {
            DynamicImage::ImageLuma8(img_buf) => {
                DaftImageBuffer::<'a>::L(image_buffer_vec_to_cow(img_buf))
            }
            DynamicImage::ImageLumaA8(img_buf) => {
                DaftImageBuffer::<'a>::LA(image_buffer_vec_to_cow(img_buf))
            }
            DynamicImage::ImageRgb8(img_buf) => {
                DaftImageBuffer::<'a>::RGB(image_buffer_vec_to_cow(img_buf))
            }
            DynamicImage::ImageRgba8(img_buf) => {
                DaftImageBuffer::<'a>::RGBA(image_buffer_vec_to_cow(img_buf))
            }
            DynamicImage::ImageLuma16(img_buf) => {
                DaftImageBuffer::<'a>::L16(image_buffer_vec_to_cow(img_buf))
            }
            DynamicImage::ImageLumaA16(img_buf) => {
                DaftImageBuffer::<'a>::LA16(image_buffer_vec_to_cow(img_buf))
            }
            DynamicImage::ImageRgb16(img_buf) => {
                DaftImageBuffer::<'a>::RGB16(image_buffer_vec_to_cow(img_buf))
            }
            DynamicImage::ImageRgba16(img_buf) => {
                DaftImageBuffer::<'a>::RGBA16(image_buffer_vec_to_cow(img_buf))
            }
            DynamicImage::ImageRgb32F(img_buf) => {
                DaftImageBuffer::<'a>::RGB32F(image_buffer_vec_to_cow(img_buf))
            }
            DynamicImage::ImageRgba32F(img_buf) => {
                DaftImageBuffer::<'a>::RGBA32F(image_buffer_vec_to_cow(img_buf))
            }
            _ => unimplemented!("{dyn_img:?} not implemented"),
        }
    }
}

pub struct ImageArrayVecs<T> {
    pub data: Vec<T>,
    pub channels: Vec<u16>,
    pub heights: Vec<u32>,
    pub widths: Vec<u32>,
    pub modes: Vec<u8>,
    pub offsets: Vec<i64>,
    pub validity: Option<arrow2::bitmap::Bitmap>,
}

impl ImageArray {
    pub fn image_mode(&self) -> &Option<ImageMode> {
        match self.logical_type() {
            DataType::Image(_, mode) => mode,
            _ => panic!("Expected dtype to be Image"),
        }
    }

    pub fn data_array(&self) -> &arrow2::array::ListArray<i64> {
        let p = self.physical.as_arrow();
        const IMAGE_DATA_IDX: usize = 0;
        let array = p.values().get(IMAGE_DATA_IDX).unwrap();
        array.as_ref().as_any().downcast_ref().unwrap()
    }

    pub fn channel_array(&self) -> &arrow2::array::UInt16Array {
        let p = self.physical.as_arrow();
        const IMAGE_CHANNEL_IDX: usize = 1;
        let array = p.values().get(IMAGE_CHANNEL_IDX).unwrap();
        array.as_ref().as_any().downcast_ref().unwrap()
    }

    pub fn height_array(&self) -> &arrow2::array::UInt32Array {
        let p = self.physical.as_arrow();
        const IMAGE_HEIGHT_IDX: usize = 2;
        let array = p.values().get(IMAGE_HEIGHT_IDX).unwrap();
        array.as_ref().as_any().downcast_ref().unwrap()
    }

    pub fn width_array(&self) -> &arrow2::array::UInt32Array {
        let p = self.physical.as_arrow();
        const IMAGE_WIDTH_IDX: usize = 3;
        let array = p.values().get(IMAGE_WIDTH_IDX).unwrap();
        array.as_ref().as_any().downcast_ref().unwrap()
    }

    pub fn mode_array(&self) -> &arrow2::array::UInt8Array {
        let p = self.physical.as_arrow();
        const IMAGE_MODE_IDX: usize = 4;
        let array = p.values().get(IMAGE_MODE_IDX).unwrap();
        array.as_ref().as_any().downcast_ref().unwrap()
    }

    pub fn from_vecs<T: arrow2::types::NativeType>(
        name: &str,
        data_type: DataType,
        vecs: ImageArrayVecs<T>,
    ) -> DaftResult<Self> {
        if vecs.data.is_empty() {
            // Create an all-null array if the data array is empty.
            let physical_type = data_type.to_physical();
            let null_struct_array =
                arrow2::array::new_null_array(physical_type.to_arrow()?, vecs.channels.len());
            let daft_struct_array =
                StructArray::new(Field::new(name, physical_type).into(), null_struct_array)?;
            return Ok(ImageArray::new(
                Field::new(name, data_type),
                daft_struct_array,
            ));
        }
        let offsets = arrow2::offset::OffsetsBuffer::try_from(vecs.offsets)?;
        let arrow_dtype: arrow2::datatypes::DataType = T::PRIMITIVE.into();
        if let DataType::Image(inner_dtype, _) = &data_type {
            if inner_dtype.to_arrow()? != arrow_dtype {
                panic!("Inner value dtype of provided dtype {data_type:?} is inconsistent with inferred value dtype {arrow_dtype:?}");
            }
        }

        let list_datatype = arrow2::datatypes::DataType::LargeList(Box::new(
            arrow2::datatypes::Field::new("data", arrow_dtype, true),
        ));
        let data_array = Box::new(arrow2::array::ListArray::<i64>::new(
            list_datatype,
            offsets,
            Box::new(arrow2::array::PrimitiveArray::from_vec(vecs.data)),
            vecs.validity.clone(),
        ));

        let values: Vec<Box<dyn arrow2::array::Array>> = vec![
            data_array,
            Box::new(
                arrow2::array::UInt16Array::from_vec(vecs.channels)
                    .with_validity(vecs.validity.clone()),
            ),
            Box::new(
                arrow2::array::UInt32Array::from_vec(vecs.heights)
                    .with_validity(vecs.validity.clone()),
            ),
            Box::new(
                arrow2::array::UInt32Array::from_vec(vecs.widths)
                    .with_validity(vecs.validity.clone()),
            ),
            Box::new(
                arrow2::array::UInt8Array::from_vec(vecs.modes)
                    .with_validity(vecs.validity.clone()),
            ),
        ];
        let physical_type = data_type.to_physical();
        let struct_array = Box::new(arrow2::array::StructArray::new(
            physical_type.to_arrow()?,
            values,
            vecs.validity,
        ));

        let daft_struct_array = crate::datatypes::StructArray::new(
            Field::new(name, physical_type).into(),
            struct_array,
        )?;
        Ok(ImageArray::new(
            Field::new(name, data_type),
            daft_struct_array,
        ))
    }

    pub fn as_image_obj<'a>(&'a self, idx: usize) -> Option<DaftImageBuffer<'a>> {
        assert!(idx < self.len());
        if !self.physical.is_valid(idx) {
            return None;
        }

        let da = self.data_array();
        let ca = self.channel_array();
        let ha = self.height_array();
        let wa = self.width_array();
        let ma = self.mode_array();

        let offsets = da.offsets();

        let start = *offsets.get(idx).unwrap() as usize;
        let end = *offsets.get(idx + 1).unwrap() as usize;

        let values = da
            .values()
            .as_ref()
            .as_any()
            .downcast_ref::<arrow2::array::UInt8Array>()
            .unwrap();
        let slice_data = Cow::Borrowed(&values.values().as_slice()[start..end] as &'a [u8]);

        let c = ca.value(idx);
        let h = ha.value(idx);
        let w = wa.value(idx);
        let m: ImageMode = ImageMode::from_u8(ma.value(idx)).unwrap();
        assert_eq!(m.num_channels(), c);
        let result = match m {
            ImageMode::L => {
                DaftImageBuffer::<'a>::L(ImageBuffer::from_raw(w, h, slice_data).unwrap())
            }
            ImageMode::LA => {
                DaftImageBuffer::<'a>::LA(ImageBuffer::from_raw(w, h, slice_data).unwrap())
            }
            ImageMode::RGB => {
                DaftImageBuffer::<'a>::RGB(ImageBuffer::from_raw(w, h, slice_data).unwrap())
            }
            ImageMode::RGBA => {
                DaftImageBuffer::<'a>::RGBA(ImageBuffer::from_raw(w, h, slice_data).unwrap())
            }
            _ => unimplemented!("{m} is currently not implemented!"),
        };

        assert_eq!(result.height(), h);
        assert_eq!(result.width(), w);
        Some(result)
    }

    pub fn encode(&self, image_format: ImageFormat) -> DaftResult<BinaryArray> {
        let result = (0..self.len())
            .map(|i| self.as_image_obj(i))
            .map(|img| {
                img.map(|img| {
                    let mut buf = Vec::new();
                    img.encode(image_format, &mut buf)?;
                    Ok(buf)
                })
                .transpose()
            })
            .collect::<DaftResult<Vec<_>>>()?;
        let arrow_array = arrow2::array::BinaryArray::<i64>::from_iter(result.into_iter());
        BinaryArray::new(
            Field::new(self.name(), arrow_array.data_type().into()).into(),
            arrow_array.boxed(),
        )
    }

    pub fn resize(&self, w: u32, h: u32) -> DaftResult<Self> {
        let result = (0..self.len())
            .map(|i| self.as_image_obj(i))
            .map(|img| img.map(|img| img.resize(w, h)))
            .collect::<Vec<_>>();
        Self::from_daft_image_buffers(self.name(), result.as_slice(), self.image_mode())
    }

    pub fn from_daft_image_buffers(
        name: &str,
        inputs: &[Option<DaftImageBuffer<'_>>],
        image_mode: &Option<ImageMode>,
    ) -> DaftResult<Self> {
        use DaftImageBuffer::*;
        let is_all_u8 = inputs
            .iter()
            .filter_map(|b| b.as_ref())
            .all(|b| matches!(b, L(..) | LA(..) | RGB(..) | RGBA(..)));
        assert!(is_all_u8);

        let mut data_ref = Vec::with_capacity(inputs.len());
        let mut heights = Vec::with_capacity(inputs.len());
        let mut channels = Vec::with_capacity(inputs.len());
        let mut modes = Vec::with_capacity(inputs.len());
        let mut widths = Vec::with_capacity(inputs.len());
        let mut offsets = Vec::with_capacity(inputs.len() + 1);
        offsets.push(0i64);
        let mut validity = arrow2::bitmap::MutableBitmap::with_capacity(inputs.len());

        for ib in inputs {
            validity.push(ib.is_some());
            let (height, width, mode, buffer) = match ib {
                Some(ib) => (ib.height(), ib.width(), ib.mode(), ib.as_u8_slice()),
                None => (0u32, 0u32, ImageMode::L, &[] as &[u8]),
            };
            heights.push(height);
            widths.push(width);
            modes.push(mode as u8);
            channels.push(mode.num_channels());
            data_ref.push(buffer);
            offsets.push(offsets.last().unwrap() + buffer.len() as i64);
        }

        let data = data_ref.concat();
        let validity: Option<arrow2::bitmap::Bitmap> = match validity.unset_bits() {
            0 => None,
            _ => Some(validity.into()),
        };
        ImageArray::from_vecs(
            name,
            DataType::Image(Box::new(DataType::UInt8), *image_mode),
            ImageArrayVecs {
                data,
                channels,
                heights,
                widths,
                modes,
                offsets,
                validity,
            },
        )
    }
}

impl BinaryArray {
    pub fn image_decode(&self) -> DaftResult<ImageArray> {
        let arrow_array = self
            .data()
            .as_any()
            .downcast_ref::<arrow2::array::BinaryArray<i64>>()
            .unwrap();
        let mut img_bufs = Vec::<Option<DaftImageBuffer>>::with_capacity(arrow_array.len());
        let mut cached_dtype: Option<DataType> = None;
        // Load images from binary buffers.
        // Confirm that all images have the same value dtype.
        for row in arrow_array.iter() {
            let img_buf = row.map(DaftImageBuffer::decode).transpose()?;
            let dtype = img_buf.as_ref().map(|im| im.mode().get_dtype());
            match (dtype.as_ref(), cached_dtype.as_ref()) {
                (Some(t1), Some(t2)) => {
                    if t1 != t2 {
                        return Err(DaftError::ValueError(format!("All images in a column must have the same dtype, but got: {:?} and {:?}", t1, t2)));
                    }
                }
                (Some(t1), None) => {
                    cached_dtype = Some(t1.clone());
                }
                (None, _) => {}
            }
            img_bufs.push(img_buf);
        }
        // Fall back to UInt8 dtype if series is all nulls.
        let cached_dtype = cached_dtype.unwrap_or(DataType::UInt8);
        match cached_dtype {
            DataType::UInt8 => Ok(ImageArray::from_daft_image_buffers(self.name(), img_bufs.as_slice(), &None)?),
            _ => unimplemented!("Decoding images of dtype {cached_dtype:?} is not supported, only uint8 images are supported."),
        }
    }
}
