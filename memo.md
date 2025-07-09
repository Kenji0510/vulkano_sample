```bash
Loaded /Users/kenji/Downloads/combined_120.pcd
PCD data length: 160018
=== Physical Devices ===
Name: Apple M4 Pro
Type:            IntegratedGpu
API Version:     1.2.0
Driver Version:  10211
UUID:            Some([0, 0, 10, 6b, f, 5, 2, 9, 0, 0, 0, 0, 0, 0, 0, 0])
  Heap[0]: size = 49152 MB, flags = DEVICE_LOCAL
Max work_group_count:  [1073741824, 1073741824, 1073741824]
Max work_group_size:   [1024, 1024, 1024]
Max invocations:       1024
Max storage_buffer:    4294967295
Max uniform_buffer:    4294967295
Timestamp period (ns): 1
-----------------------------
GPU computation time: 20.160208ms
Point { x: 1.0183309, y: 14.78983, z: -0.61756647 }
Succeeded!
```
```bash
Loaded /home/kenji/workspace/Rust/vulkano_sample/data/combined_120.pcd
PCD data length: 160018
=== Physical Devices ===
Name: NVIDIA GeForce RTX 4080
Type:            DiscreteGpu
API Version:     1.2.0
Driver Version:  2309537856
UUID:            Some([59, ec, d3, 5f, a5, a6, 82, 3b, 23, 51, ba, 99, 4d, 16, 32, 9c])
  Heap[0]: size = 16376 MB, flags = DEVICE_LOCAL
  Heap[1]: size = 24053 MB, flags = empty()
Max work_group_count:  [2147483647, 65535, 65535]
Max work_group_size:   [1024, 1024, 64]
Max invocations:       1024
Max storage_buffer:    4294967295
Max uniform_buffer:    65536
Timestamp period (ns): 1
-----------------------------
=== Physical Devices ===
Name: llvmpipe (LLVM 19.1.1, 256 bits)
Type:            Cpu
API Version:     1.2.0
Driver Version:  1
UUID:            Some([6d, 65, 73, 61, 32, 34, 2e, 32, 2e, 38, 2d, 31, 75, 62, 75, 0])
  Heap[0]: size = 32071 MB, flags = DEVICE_LOCAL
Max work_group_count:  [65535, 65535, 65535]
Max work_group_size:   [1024, 1024, 1024]
Max invocations:       1024
Max storage_buffer:    134217728
Max uniform_buffer:    65536
Timestamp period (ns): 1
-----------------------------
GPU computation time: 408.301µs
Point { x: 1.0183309, y: 14.78983, z: -0.61756647 }
Succeeded!
```

```bash
Loaded /home/kenji/workspace/Rust/vulkano_sample/data/combined_120.pcd
=== PCD Info ===
PCD data length: 160018
Points num: 480054
------------------

=== Parameters ===
Voxel size: 0.1
Scale: 1000
Inv scale: 0.001
Capacity: 1048576
Buffer bytes: 4194304
Hash mask: 1048575
------------------

=== Physical Devices ===
Name: NVIDIA GeForce RTX 4080
Type:            DiscreteGpu
API Version:     1.2.0
Driver Version:  2309537856
UUID:            Some([59, ec, d3, 5f, a5, a6, 82, 3b, 23, 51, ba, 99, 4d, 16, 32, 9c])
  Heap[0]: size = 16376 MB, flags = DEVICE_LOCAL
  Heap[1]: size = 24053 MB, flags = empty()
Max work_group_count:  [2147483647, 65535, 65535]
Max work_group_size:   [1024, 1024, 64]
Max invocations:       1024
Max storage_buffer:    4294967295
Max uniform_buffer:    65536
Timestamp period (ns): 1
-----------------------------
=== Physical Devices ===
Name: llvmpipe (LLVM 19.1.1, 256 bits)
Type:            Cpu
API Version:     1.2.0
Driver Version:  1
UUID:            Some([6d, 65, 73, 61, 32, 34, 2e, 32, 2e, 38, 2d, 31, 75, 62, 75, 0])
  Heap[0]: size = 32071 MB, flags = DEVICE_LOCAL
Max work_group_count:  [65535, 65535, 65535]
Max work_group_size:   [1024, 1024, 1024]
Max invocations:       1024
Max storage_buffer:    134217728
Max uniform_buffer:    65536
Timestamp period (ns): 1
-----------------------------
=== Run Result ===
GPU computation time: 7.036515ms
Fail cnt: 0
Centroids num: 35431
Point { x: 2.660667, y: 0.73066676, z: -1.2123334 }
------------------

Saved voxelization points
Succeeded!
```

```bash
Loaded /home/kenji/workspace/Rust/vulkano_sample/data/combined_120.pcd
=== PCD Info ===
PCD data length: 160018
Points num: 480054
------------------

=== Parameters ===
Voxel size: 0.05
Scale: 1000
Inv scale: 0.001
Capacity: 1048576
Buffer bytes: 4194304
Hash mask: 1048575
------------------

=== Physical Devices ===
Name: NVIDIA GeForce RTX 4080
Type:            DiscreteGpu
API Version:     1.2.0
Driver Version:  2309537856
UUID:            Some([59, ec, d3, 5f, a5, a6, 82, 3b, 23, 51, ba, 99, 4d, 16, 32, 9c])
  Heap[0]: size = 16376 MB, flags = DEVICE_LOCAL
  Heap[1]: size = 24053 MB, flags = empty()
Max work_group_count:  [2147483647, 65535, 65535]
Max work_group_size:   [1024, 1024, 64]
Max invocations:       1024
Max storage_buffer:    4294967295
Max uniform_buffer:    65536
Timestamp period (ns): 1
-----------------------------
=== Physical Devices ===
Name: llvmpipe (LLVM 19.1.1, 256 bits)
Type:            Cpu
API Version:     1.2.0
Driver Version:  1
UUID:            Some([6d, 65, 73, 61, 32, 34, 2e, 32, 2e, 38, 2d, 31, 75, 62, 75, 0])
  Heap[0]: size = 32071 MB, flags = DEVICE_LOCAL
Max work_group_count:  [65535, 65535, 65535]
Max work_group_size:   [1024, 1024, 1024]
Max invocations:       1024
Max storage_buffer:    134217728
Max uniform_buffer:    65536
Timestamp period (ns): 1
-----------------------------
=== Run Result ===
GPU computation time: 8.70099ms
Fail cnt: 0
Centroids num: 81880
Point { x: -0.22950001, y: 5.8155003, z: -1.6925001 }
------------------

Saved voxelization points
Succeeded!
```

```bash
Loaded /home/kenji/workspace/Rust/vulkano_sample/data/combined_120.pcd
=== PCD Info ===
PCD data length: 160018
Points num: 480054
------------------

=== Parameters ===
Voxel size: 0.05
Scale: 1000
Inv scale: 0.001
Capacity: 1048576
Buffer bytes: 4194304
Hash mask: 1048575
------------------

=== Physical Devices ===
Name: NVIDIA GeForce RTX 4080
Type:            DiscreteGpu
API Version:     1.2.0
Driver Version:  2309537856
UUID:            Some([59, ec, d3, 5f, a5, a6, 82, 3b, 23, 51, ba, 99, 4d, 16, 32, 9c])
  Heap[0]: size = 16376 MB, flags = DEVICE_LOCAL
  Heap[1]: size = 24053 MB, flags = empty()
Max work_group_count:  [2147483647, 65535, 65535]
Max work_group_size:   [1024, 1024, 64]
Max invocations:       1024
Max storage_buffer:    4294967295
Max uniform_buffer:    65536
Timestamp period (ns): 1
-----------------------------
=== Physical Devices ===
Name: llvmpipe (LLVM 19.1.1, 256 bits)
Type:            Cpu
API Version:     1.2.0
Driver Version:  1
UUID:            Some([6d, 65, 73, 61, 32, 34, 2e, 32, 2e, 38, 2d, 31, 75, 62, 75, 0])
  Heap[0]: size = 32071 MB, flags = DEVICE_LOCAL
Max work_group_count:  [65535, 65535, 65535]
Max work_group_size:   [1024, 1024, 1024]
Max invocations:       1024
Max storage_buffer:    134217728
Max uniform_buffer:    65536
Timestamp period (ns): 1
-----------------------------
=== Run Result ===
GPU computation time: 477.901µs
Fail cnt: 0
Centroids num: 81880
Point { x: -0.05, y: 5.1980004, z: -0.975 }
------------------

Saved voxelization points
Succeeded!
```