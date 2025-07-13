use std::{sync::Arc, time::Instant};

use vulkano::{
    Version, VulkanLibrary,
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo,
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
    },
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, QueueFlags,
        physical::PhysicalDevice,
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo, compute::ComputePipelineCreateInfo,
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    sync::{self, GpuFuture},
};

use vulkano_sample::load_pcd::{
    PointForGBuffer, convert_to_point_for_gbuffer, load_pcd, load_pcd_xyz, save_pcd,
};

#[repr(C)]
#[derive(BufferContents)]
struct Uniform {
    min_coodination: [f32; 3],
    inv_vox: f32,
    scale: f32,
    inv_scale: f32,
    hash_mask: u32,
}

fn get_min_value(points: &[PointForGBuffer]) -> (f32, f32, f32) {
    let (mut min_x, mut min_y, mut min_z) = (f32::INFINITY, f32::INFINITY, f32::INFINITY);
    for p in points {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        min_z = min_z.min(p.z);
    }
    (min_x, min_y, min_z)
}

fn display_info(device: &PhysicalDevice) {
    println!("=== Physical Devices ===");
    println!("Name: {}", device.properties().device_name);
    println!("Type:            {:?}", device.properties().device_type);
    println!("API Version:     {}", device.api_version());
    println!("Driver Version:  {}", device.properties().driver_version);
    println!("UUID:            {:x?}", device.properties().device_uuid);

    let mem_props = device.memory_properties();
    for (i, heap) in mem_props.memory_heaps.iter().enumerate() {
        println!(
            "  Heap[{}]: size = {} MB, flags = {:?}",
            i,
            heap.size / (1024 * 1024),
            heap.flags
        );
    }

    let properties = device.properties();
    println!(
        "Max work_group_count:  {:?}",
        properties.max_compute_work_group_count
    );
    println!(
        "Max work_group_size:   {:?}",
        properties.max_compute_work_group_size
    );
    println!(
        "Max invocations:       {}",
        properties.max_compute_work_group_invocations
    );
    println!(
        "Max storage_buffer:    {}",
        properties.max_storage_buffer_range
    );
    println!(
        "Max uniform_buffer:    {}",
        properties.max_uniform_buffer_range
    );
    println!("Timestamp period (ns): {}", properties.timestamp_period);
    println!("-----------------------------");
}

const MAX_STORAGE_BUFFER_SIZE: u64 = 1024 * 1024 * 1024 * 4; // 4GB

fn main() {
    // let pcd_file_path = "/home/kenji/workspace/Rust/vulkano_sample/data/combined_120.pcd";
    let pcd_file_path =
        "/home/kenji/workspace/Rust/vulkano_sample/data/source/export-street-001.pcd";

    // let pcd_data = match load_pcd(pcd_file_path) {
    let pcd_data_for_gbuffer = match load_pcd_xyz(pcd_file_path) {
        Ok(points) => {
            println!("Loaded {}", pcd_file_path);
            points
        }
        Err(e) => {
            eprintln!("Error loading PCD file: {}", e);
            return;
        }
    };

    // let pcd_data_for_gbuffer = convert_to_point_for_gbuffer(&pcd_data, pcd_data.len());

    println!("=== PCD Info ===");
    println!("PCD data length: {}", pcd_data_for_gbuffer.len());
    println!("xyz num: {}", pcd_data_for_gbuffer.len() * 3);
    println!("------------------\n");

    let (min_x, min_y, min_z) = get_min_value(&pcd_data_for_gbuffer);

    let voxel_size = 0.05;
    let scale = 1000.0;
    // let capasity = (pcd_data_for_gbuffer.len() * 3 * 4).next_power_of_two();
    let capasity = (pcd_data_for_gbuffer.len() * 2).next_power_of_two();
    let buf_bytes = capasity;

    println!("=== Parameters ===");
    println!("Voxel size: {}", &voxel_size);
    println!("Capacity: {}", &capasity);
    println!("Buffer bytes: {}", &buf_bytes);

    if (capasity * std::mem::size_of::<u32>() >= MAX_STORAGE_BUFFER_SIZE as usize)
        || (buf_bytes * std::mem::size_of::<u32>() >= MAX_STORAGE_BUFFER_SIZE as usize)
    {
        eprintln!(
            "Error: Buffer size exceeds maximum allowed size of {} bytes.",
            MAX_STORAGE_BUFFER_SIZE
        );
        return;
    }

    let uniform = Uniform {
        min_coodination: [min_x, min_y, min_z],
        inv_vox: 1.0 / voxel_size,
        scale: scale,
        inv_scale: 1.0 / scale,
        hash_mask: (capasity - 1) as u32,
    };

    println!("Scale: {}", &uniform.scale);
    println!("Inv scale: {}", &uniform.inv_scale);
    println!("Hash mask: {}", &uniform.hash_mask);
    println!("-------------------\n");

    let library = VulkanLibrary::new().expect("Failed to load vulkan library");
    let requireed_extensions = InstanceExtensions::empty();
    let instance = Instance::new(library, InstanceCreateInfo {
        enabled_extensions: requireed_extensions,
        // enumerate_portability: true,
        flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
        max_api_version: Some(Version::V1_2),
        ..Default::default()
    })
    .expect("Faied to create instance");

    for device_extension in instance.enumerate_physical_devices().unwrap() {
        display_info(&device_extension);
    }

    let physical_device = instance
        .enumerate_physical_devices()
        .expect("Could not enumerate devices!")
        .next()
        .expect("No devices available!");

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_, queue_family_properties)| {
            queue_family_properties
                .queue_flags
                .contains(QueueFlags::COMPUTE)
        })
        .expect("Could not find a compute queue family!") as u32;

    let (device, mut queues) = Device::new(physical_device, DeviceCreateInfo {
        queue_create_infos: vec![QueueCreateInfo {
            queue_family_index,
            ..Default::default()
        }],
        enabled_extensions: DeviceExtensions {
            khr_storage_buffer_storage_class: true,
            khr_shader_non_semantic_info: true,
            ..DeviceExtensions::empty()
        },
        ..Default::default()
    })
    .expect("Failed to create device!");

    let queue = queues.next().unwrap();

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    // let data_iter = 0..6553600u32;
    let staging_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        pcd_data_for_gbuffer.clone(),
    )
    .expect("Failed to create staging buffer!");

    let data_len = pcd_data_for_gbuffer.len();
    let input_data_buffer = Buffer::new_slice::<PointForGBuffer>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
        data_len as u64,
    )
    .expect("Failed to create buffer!");

    let table_key_buffer = Buffer::new_slice::<u32>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
        capasity as u64,
    )
    .expect("Failed to create table key buffer!");

    let sum_x_buffer = Buffer::new_slice::<u32>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
        capasity as u64,
    )
    .expect("Failed to create sum x buffer!");

    let sum_y_buffer = Buffer::new_slice::<u32>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
        capasity as u64,
    )
    .expect("Failed to create sum y buffer!");

    let sum_z_buffer = Buffer::new_slice::<u32>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
        capasity as u64,
    )
    .expect("Failed to create sum z buffer!");

    let table_cnt_buffer = Buffer::new_slice::<u32>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
        buf_bytes as u64,
    )
    .expect("Failed to create table cnt buffer!");

    let fail_cnt_buffer = Buffer::new_sized::<u32>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    )
    .expect("Failed to create fail cnt buffer!");

    let uniform_buffer = Buffer::from_data(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        uniform,
    )
    .expect("Failed to create uniform buffer!");

    let centroids_num_buffer = Buffer::new_sized::<u32>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    )
    .expect("Failed to create centroids num buffer!");

    let output_data_buffer = Buffer::new_slice::<PointForGBuffer>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
        data_len as u64,
    )
    .expect("Failed to create output buffer!");

    let readback_buf = Buffer::new_slice::<PointForGBuffer>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        data_len as u64,
    )
    .expect("Failed to create readback buffer!");

    let staging_fail_cnt = Buffer::new_sized::<u32>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
    )
    .expect("Failed to create staging fail cnt buffer!");

    let staging_centroids_num = Buffer::new_sized::<u32>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
    )
    .expect("Failed to create staging centroids num buffer!");

    mod cs_01 {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "src/shaders/calc_voxel_pos.comp"
        }
    }

    mod cs_02 {
        vulkano_shaders::shader! {
            ty: "compute",
            path: "src/shaders/calc_centroids.comp"
        }
    }

    // let shader = cs::load(device.clone()).expect("Failed to create shader!");
    let shader_01 = cs_01::load(device.clone()).expect("Failed to create shader!");
    let shader_02 = cs_02::load(device.clone()).expect("Failed to create shader!");

    let cs_01 = shader_01.entry_point("main").unwrap();
    let cs_02 = shader_02.entry_point("main").unwrap();
    let stage = PipelineShaderStageCreateInfo::new(cs_01);
    let stage_centroid = PipelineShaderStageCreateInfo::new(cs_02);
    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage, &stage_centroid])
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    let compute_pipeline_01 = ComputePipeline::new(
        device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage, layout.clone()),
    )
    .expect("Failed to create compute pipeline!");

    let compute_pipeline_02 = ComputePipeline::new(
        device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage_centroid, layout.clone()),
    )
    .expect("Failed to create compute pipeline!");

    let descriptor_set_allocator =
        StandardDescriptorSetAllocator::new(device.clone(), Default::default());

    let pipeline_layout_01 = compute_pipeline_01.layout();
    let descriptor_set_layouts_01 = pipeline_layout_01.set_layouts();
    let descriptor_set_layout_index = 0;
    let descriptor_set_layout = descriptor_set_layouts_01
        .get(descriptor_set_layout_index)
        .unwrap();

    let descriptor_set_01 = DescriptorSet::new(
        Arc::new(descriptor_set_allocator),
        descriptor_set_layout.clone(),
        [
            WriteDescriptorSet::buffer(0, input_data_buffer.clone()),
            WriteDescriptorSet::buffer(1, table_key_buffer.clone()),
            WriteDescriptorSet::buffer(2, sum_x_buffer.clone()),
            WriteDescriptorSet::buffer(3, sum_y_buffer.clone()),
            WriteDescriptorSet::buffer(4, sum_z_buffer.clone()),
            WriteDescriptorSet::buffer(5, table_cnt_buffer.clone()),
            WriteDescriptorSet::buffer(6, fail_cnt_buffer.clone()),
            WriteDescriptorSet::buffer(7, uniform_buffer.clone()),
            WriteDescriptorSet::buffer(8, centroids_num_buffer.clone()),
            WriteDescriptorSet::buffer(9, output_data_buffer.clone()),
        ],
        [],
    )
    .unwrap();

    // let descriptor_set_02 = PersistentDescriptorSet::new(
    //     &descriptor_set_allocator,
    //     descriptor_set_layout.clone(),
    //     [
    //         WriteDescriptorSet::buffer(0, input_data_buffer.clone()),
    //         WriteDescriptorSet::buffer(1, table_key_buffer.clone()),
    //         WriteDescriptorSet::buffer(2, sum_x_buffer.clone()),
    //         WriteDescriptorSet::buffer(3, sum_y_buffer.clone()),
    //         WriteDescriptorSet::buffer(4, sum_z_buffer.clone()),
    //         WriteDescriptorSet::buffer(5, table_cnt_buffer.clone()),
    //         WriteDescriptorSet::buffer(6, fail_cnt_buffer.clone()),
    //         WriteDescriptorSet::buffer(7, uniform_buffer.clone()),
    //         WriteDescriptorSet::buffer(8, centroids_num_buffer.clone()),
    //         WriteDescriptorSet::buffer(9, output_data_buffer.clone()),
    //     ],
    //     [],
    // )
    // .unwrap();

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    );

    // let transfer_to_compute = DependencyInfo {
    //     memory_barriers: {
    //         let mut barriers = SmallVec::new();
    //         barriers.push(MemoryBarrier {
    //             src_stages: PipelineStages::ALL_TRANSFER,
    //             dst_stages: PipelineStages::COMPUTE_SHADER,
    //             src_access: AccessFlags::TRANSFER_WRITE,
    //             dst_access: AccessFlags::SHADER_READ | AccessFlags::SHADER_WRITE,
    //             ..Default::default()
    //         });
    //         barriers
    //     },
    //     ..Default::default()
    // };

    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        Arc::new(command_buffer_allocator),
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    const LOCAL_SIZE: u32 = 256;
    let group_count_x = (data_len as u32 + LOCAL_SIZE - 1) / LOCAL_SIZE;

    let work_group_counts = [group_count_x, 1, 1];

    unsafe {
        command_buffer_builder
            .copy_buffer(CopyBufferInfo::buffers(
                staging_buffer.clone(),
                input_data_buffer.clone(),
            ))
            .unwrap()
            .fill_buffer(table_key_buffer.clone(), 0u32)
            .unwrap()
            .fill_buffer(sum_x_buffer.clone(), 0u32)
            .unwrap()
            .fill_buffer(sum_y_buffer.clone(), 0u32)
            .unwrap()
            .fill_buffer(sum_z_buffer.clone(), 0u32)
            .unwrap()
            .fill_buffer(table_cnt_buffer.clone(), 0u32)
            .unwrap()
            .bind_pipeline_compute(compute_pipeline_01.clone()) // The first compute pipeline
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                compute_pipeline_01.layout().clone(),
                descriptor_set_layout_index as u32,
                descriptor_set_01.clone(),
            )
            .unwrap()
            .dispatch(work_group_counts)
            .unwrap()
            .bind_pipeline_compute(compute_pipeline_02.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                compute_pipeline_02.layout().clone(),
                descriptor_set_layout_index as u32,
                descriptor_set_01.clone(),
            )
            .unwrap()
            .dispatch([capasity as u32 / LOCAL_SIZE, 1, 1])
            .unwrap()
            .copy_buffer(CopyBufferInfo::buffers(
                output_data_buffer.clone(),
                readback_buf.clone(),
            ))
            .unwrap()
            .copy_buffer(CopyBufferInfo::buffers(
                fail_cnt_buffer.clone(),
                staging_fail_cnt.clone(),
            ))
            .unwrap()
            .copy_buffer(CopyBufferInfo::buffers(
                centroids_num_buffer.clone(),
                staging_centroids_num.clone(),
            ))
            .unwrap();
    }

    let command_buffer = command_buffer_builder.build().unwrap();

    let compute_start_time = Instant::now();
    let future = sync::now(device)
        .then_execute(queue, command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();
    let compute_end_time = compute_start_time.elapsed();

    let fail_cnt = staging_fail_cnt.read().unwrap();

    let centroids_num = staging_centroids_num.read().unwrap();

    let voxelization_points = readback_buf.read().unwrap();
    let voxelization_points_vec = voxelization_points[..*centroids_num as usize].to_vec();

    println!("=== Run Result ===");
    println!("GPU computation time: {:?}", compute_end_time);
    println!("Fail cnt: {}", *fail_cnt);
    println!("Centroids num: {}", *centroids_num);
    println!("{:?}", voxelization_points[0]);
    println!("------------------\n");

    match save_pcd(
        "/home/kenji/workspace/Rust/vulkano_sample/data/export-vulkano-voxelization.pcd",
        voxelization_points_vec,
    ) {
        Ok(_) => println!("Saved voxelization points"),
        Err(e) => eprintln!("Error saving voxelization points: {}", e),
    };

    println!("Succeeded!");
}
