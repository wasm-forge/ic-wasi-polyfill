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

The "Noop" functions do not have any effect on the execution. Calling the "Not supported" functions cause the application to panic.


| WASI function               | Status        | 
| -------------               | -------       |
| `args_get`                  | Noop          |
| `args_sizes_get`            | Noop          |
| `clock_res_get`             | Supported     |
| `clock_time_get`            | Supported     |
| `environ_get`               | Noop          |
| `environ_sizes_get`         | Noop          |
| `fd_advise`                 | Supported     |
| `fd_allocate`               | Noop          |
| `fd_close`                  | Supported     |
| `fd_datasync`               | Noop          |
| `fd_fdstat_get`             | Supported     |
| `fd_fdstat_set_flags`       | Supported     |
| `fd_fdstat_set_rights`      | Supported     |
| `fd_filestat_get`           | Supported     |
| `fd_filestat_set_size`      | Noop          |
| `fd_filestat_set_times`     | Supported     |
| `fd_pread`                  | Supported     |
| `fd_prestat_dir_name`       | Supported     |
| `fd_prestat_get`            | Supported     |
| `fd_pwrite`                 | Supported     |
| `fd_read`                   | Supported     |
| `fd_readdir`                | Supported     |
| `fd_renumber`               | Supported     |
| `fd_seek`                   | Supported     |
| `fd_sync`                   | Supported     |
| `fd_tell`                   | Supported     |
| `fd_write`                  | Supported     |
| `path_create_directory`     | Supported     |
| `path_filestat_get`         | Supported     |
| `path_filestat_set_times`   | Not supported |
| `path_link`                 | Not supported |
| `path_open`                 | Supported     |
| `path_readlink`             | Not supported |
| `path_remove_directory`     | Supported |
| `path_rename`               | Not supported |
| `path_symlink`              | Not supported |
| `path_unlink_file`          | Supported     |
| `poll_oneoff`               | Not supported |
| `proc_exit`                 | Supported     |
| `proc_raise`                | Not supported |
| `random_get`                | Supported<sup>1</sup>   |
| `sched_yield`               | Noop          |
| `sock_accept`               | Not supported |
| `sock_recv`                 | Not supported |
| `sock_send`                 | Not supported |
| `sock_shutdown`             | Not supported |

*<sup>1</sup>* - The `random_get` function utilizes a synchronous pseudorandom number generator.


## Additional library functions


| Function                                      |  Description                  | 
| -------------                                 | -------                       |
| `init(seed: &[u8])`                           | Initialization call.           |
| `raw_init(seed: *const u8, len: usize)`       | Similar to `init`, but has simpler parameters for calling from C or C++. |
| `init_seed(seed: &[u8])`                      | Convenience method to explicitly re-initialize the random seed. |
| `raw_init_seed(seed: *const u8, len: usize)`  | Similar to `init_seed`, but has simpler parameters for calling from C or C++. |

