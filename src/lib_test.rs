use crate::{__ic_custom_environ_get, __ic_custom_environ_sizes_get, init, wasi};

#[test]
fn test_environ_get() {

    let init_env = [
        ("PATH", "/usr/bin"),
        ("UID", "1028"),
        ("HOME", "/home/user"),
    ];

    let mut lines = Vec::new();

    for pair in init_env.iter() {
        lines.push(String::from(pair.0) + "=" + pair.1 + "\0");
    }

    let lines = lines;

    unsafe {
        init(
            &[12, 3, 54, 1],
            &init_env,
        );
    }

    // get environment sizes
    let mut entry_count: wasi::Size = 0;
    let mut buffer_size: wasi::Size = 0;

    let ret = unsafe {
        __ic_custom_environ_sizes_get(
            (&mut entry_count) as *mut wasi::Size,
            (&mut buffer_size) as *mut wasi::Size,
        )
    };

    assert!(ret == 0);
    assert!(entry_count == lines.len());

    let mut expected_buffer_size = 0;
    for i in lines.iter().map(|x: &String| x.len()) {
        expected_buffer_size += i;
    }

    assert!(buffer_size == expected_buffer_size);


    let mut entry_table: Vec<wasi::Size> = Vec::with_capacity(entry_count);
    unsafe {entry_table.set_len(entry_count);}

    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_size);
    unsafe {buffer.set_len(buffer_size);}

    // get environment values
    let ret = unsafe {
        __ic_custom_environ_get(
            entry_table.as_mut_ptr() as *mut *mut u8,
            buffer.as_mut_ptr() as *mut u8,
        )
    };

    assert!(ret == 0);

    let computed_string = String::from_utf8(buffer).unwrap();

    let mut expected_string = String::new();
    for env_string in lines.iter() {
        expected_string += env_string;
    }
    
    assert!(computed_string == expected_string);

}
