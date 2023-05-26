# The IC Polyfill library

![Tests](https://github.com/wasm-forge/ic-wasi-polyfill/actions/workflows/rust.yml/badge.svg?event=push)

The project provides polyfill implementation of *wasi_unstable* and *wasi_snapshot_preview1* functions using IC System API.


## Usage

The intended use is to add this library as a dependency to your rust project. And then run `wasi2ic` on the produced Wasm binary.

In your project you would need to call the `init` function. It makes sure the linker does not remove the functions and can be used to initialize the random seed.

Example:
```rust
    init(&[12,3,54,1]);
```


## Supported WASI functions (wasi_unstable, wasi_snapshot_preview1)


| Status           | Description                                                  |
| ---------------- | ------------------------------------------------------------ |
| Supported        | Function is fully supported.                                 |
| No-op            | Empty implementation that does nothing but can be called without issues. |
| Not implemented  | Function is not yet implemented - calling it causes the application to panic. |
| Not supported    | Function is not planned to be implemented - calling it causes the application to panic. |


| WASI function               | Status          | 
| --------------------------- | --------------- |
| `args_get`                  | No-op           |
| `args_sizes_get`            | No-op           |
| `clock_res_get`             | Supported       |
| `clock_time_get`            | Supported       |
| `environ_get`               | No-op           |
| `environ_sizes_get`         | No-op           |
| `fd_advise`                 | No-op           |
| `fd_allocate`               | No-op           |
| `fd_close`                  | Supported       |
| `fd_datasync`               | No-op           |
| `fd_fdstat_get`             | Supported       |
| `fd_fdstat_set_flags`       | Supported       |
| `fd_fdstat_set_rights`      | Supported       |
| `fd_filestat_get`           | Supported       |
| `fd_filestat_set_size`      | No-op           |
| `fd_filestat_set_times`     | Supported       |
| `fd_pread`                  | Supported       |
| `fd_prestat_dir_name`       | Supported       |
| `fd_prestat_get`            | Supported       |
| `fd_pwrite`                 | Supported       |
| `fd_read`                   | Supported       |
| `fd_readdir`                | Supported       |
| `fd_renumber`               | Supported       |
| `fd_seek`                   | Supported       |
| `fd_sync`                   | Supported       |
| `fd_tell`                   | Supported       |
| `fd_write`                  | Supported       |
| `path_create_directory`     | Supported       |
| `path_filestat_get`         | Supported<sup>1</sup>       |
| `path_filestat_set_times`   | Supported<sup>1</sup>       |
| `path_link`                 | Supported<sup>1</sup>       |
| `path_open`                 | Supported<sup>1</sup>       |
| `path_readlink`             | Not implemented |
| `path_remove_directory`     | Supported       |
| `path_rename`               | Supported       |
| `path_symlink`              | Not implemented |
| `path_unlink_file`          | Supported       |
| `poll_oneoff`               | Not implemented |
| `proc_exit`                 | Supported       |
| `proc_raise`                | Not implemented |
| `random_get`                | Supported<sup>2</sup>       |
| `sched_yield`               | No-op           |
| `sock_accept`               | Not supported   |
| `sock_recv`                 | Not supported   |
| `sock_send`                 | Not supported   |
| `sock_shutdown`             | Not supported   |

*<sup>1</sup>* - Currently symlinks are not supported by the file system, this affects a few `path_` functions, the `flags` ("follow symlink") parameter is currently ignored.

*<sup>2</sup>* - The `random_get` function utilizes a synchronous pseudorandom number generator.


## Additional library functions


| Function                                      |  Description                  | 
| --------------------------------------------- | ----------------------------- |
| `init(seed: &[u8])`                           | Initialization call.          |
| `raw_init(seed: *const u8, len: usize)`       | Similar to `init`, but has simpler parameters for calling from C or C++. |
| `init_seed(seed: &[u8])`                      | Convenience method to explicitly re-initialize the random seed. |
| `raw_init_seed(seed: *const u8, len: usize)`  | Similar to `init_seed`, but has simpler parameters for calling from C or C++. |

