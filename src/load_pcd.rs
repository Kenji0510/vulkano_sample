// use std::fmt::Result;
use anyhow::Result;

use pcd_rs::{PcdDeserialize, PcdSerialize, Reader, WriterInit};
use vulkano::buffer::BufferContents;

#[derive(Clone, Copy, Debug, PcdDeserialize, PcdSerialize, BufferContents)]
#[repr(C)]
pub struct PointForGBuffer {
    // #[pcd(rename = "new_x")]
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// #[derive(Debug, PcdSerialize)]
#[derive(Clone, Copy, Debug, PcdDeserialize, PcdSerialize)]
#[repr(C)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub intensity: f32,
    pub normal_x: f32,
    pub normal_y: f32,
    pub normal_z: f32,
    pub curvature: f32,
}

pub fn load_pcd(file_path: &str) -> Result<Vec<Point>, Box<dyn std::error::Error>> {
    let reader = match Reader::open(file_path) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Error opening PCD file: {}", e);
            return Err(e.into());
        }
    };

    let points: Vec<Point> = match reader.collect() {
        Ok(points) => points,
        Err(e) => {
            eprintln!("Error reading PCD file: {}", e);
            return Err(e.into());
        }
    };

    Ok(points)
}

pub fn load_pcd_xyz(file_path: &str) -> Result<Vec<PointForGBuffer>, Box<dyn std::error::Error>> {
    let reader = match Reader::open(file_path) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Error opening PCD file: {}", e);
            return Err(e.into());
        }
    };

    let points: Vec<PointForGBuffer> = match reader.collect() {
        Ok(points) => points,
        Err(e) => {
            eprintln!("Error reading PCD file: {}", e);
            return Err(e.into());
        }
    };

    Ok(points)
}

pub fn save_pcd(
    file_path: &str,
    points: Vec<PointForGBuffer>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = WriterInit {
        width: 1,
        height: points.len() as u64,
        viewpoint: Default::default(),
        data_kind: pcd_rs::DataKind::Ascii,
        schema: None,
    }
    .create(
        "/home/kenji/workspace/Rust/vulkano_sample/data/export/export-vulkano-voxelization.pcd",
    )?;

    for point in points {
        writer.push(&point)?;
    }

    writer.finish()?;
    Ok(())
}

pub fn convert_to_point(data: &[f32], max_num: usize) -> Vec<PointForGBuffer> {
    let mut points = Vec::new();
    for i in (0..max_num).step_by(3) {
        if i + 2 < max_num {
            points.push(PointForGBuffer {
                x: data[i],
                y: data[i + 1],
                z: data[i + 2],
            });
        }
    }

    points
}

pub fn convert_to_point_for_gbuffer(data: &Vec<Point>, max_num: usize) -> Vec<PointForGBuffer> {
    let mut points = Vec::new();
    for i in 0..max_num {
        points.push(PointForGBuffer {
            x: data[i].x,
            y: data[i].y,
            z: data[i].z,
        });
    }

    return points;
}
