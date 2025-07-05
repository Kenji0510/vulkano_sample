use std::{sync::Arc, time::Instant};

use vulkano::{
    Version, VulkanLibrary,
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage,
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
    },
    descriptor_set::{
        PersistentDescriptorSet, WriteDescriptorSet, allocator::StandardDescriptorSetAllocator,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, QueueFlags,
        physical::PhysicalDevice,
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
        compute::ComputePipelineCreateInfo,
        layout::{PipelineDescriptorSetLayoutCreateInfo, PipelineLayoutCreateInfo},
    },
    shader::ShaderModule,
    sync::{self, GpuFuture},
};

fn display_info(device: &PhysicalDevice) {
    println!("=== Physical Devices ===");
    println!("Name: {}", device.properties().device_name);
    println!("Type:            {:?}", device.properties().device_type);
    println!("API Version:     {}", device.api_version(),);
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

fn main() {
    let library = VulkanLibrary::new().expect("Failed to load vulkan library");
    let requireed_extensions = InstanceExtensions::empty();
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: requireed_extensions,
            // enumerate_portability: true,
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            max_api_version: Some(Version::V1_2),
            ..Default::default()
        },
    )
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

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: DeviceExtensions {
                khr_storage_buffer_storage_class: true,
                ..DeviceExtensions::empty()
            },
            ..Default::default()
        },
    )
    .expect("Failed to create device!");

    let queue = queues.next().unwrap();

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let data_iter = 0..6553600u32;
    let data_len = data_iter.len();
    let data_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        data_iter,
    )
    .expect("Failed to create buffer!");

    mod cs {
        vulkano_shaders::shader! {
            ty: "compute",
            src: "
                #version 460

                layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;

                layout(set = 0, binding = 0) buffer Data {
                    uint data[];
                } buf;

                void main() {
                    uint idx = gl_GlobalInvocationID.x;
                    buf.data[idx] *= 12;
                }
            "
        }
    }

    let shader = cs::load(device.clone()).expect("Failed to create shader!");

    let cs = shader.entry_point("main").unwrap();
    let stage = PipelineShaderStageCreateInfo::new(cs);
    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        None,
        ComputePipelineCreateInfo::stage_layout(stage, layout),
    )
    .expect("Failed to create compute pipeline!");

    let descriptor_set_allocator =
        StandardDescriptorSetAllocator::new(device.clone(), Default::default());

    let pipeline_layout = compute_pipeline.layout();
    let descriptor_set_layouts = pipeline_layout.set_layouts();
    let descriptor_set_layout_index = 0;
    let descriptor_set_layout = descriptor_set_layouts
        .get(descriptor_set_layout_index)
        .unwrap();

    let descriptor_set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        descriptor_set_layout.clone(),
        [WriteDescriptorSet::buffer(0, data_buffer.clone())],
        [],
    )
    .unwrap();

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    );

    let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    const LOCAL_SIZE: u32 = 256;
    let group_count_x = (data_len as u32 + LOCAL_SIZE - 1) / LOCAL_SIZE;

    let work_group_counts = [group_count_x, 1, 1];

    command_buffer_builder
        .bind_pipeline_compute(compute_pipeline.clone())
        .unwrap()
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_pipeline.layout().clone(),
            descriptor_set_layout_index as u32,
            descriptor_set,
        )
        .unwrap()
        .dispatch(work_group_counts)
        .unwrap();

    let command_buffer = command_buffer_builder.build().unwrap();

    let compute_start_time = Instant::now();
    let future = sync::now(device)
        .then_execute(queue, command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();
    let compute_end_time = compute_start_time.elapsed();
    println!("GPU computation time: {:?}", compute_end_time);

    // let content = data_buffer.read().unwrap();
    // for (n, val) in content.iter().enumerate() {
    //     // assert_eq!(*val, n as u32 * 12);
    //     println!("{}, {}", n, *val);
    // }

    println!("Succeeded!");
}
