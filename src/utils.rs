pub mod p_hashing {
    use std::println;

    pub fn add_p_hash_for_media(
        p_hash_this: &std::path::Path,
        _check_here: &mut std::collections::BTreeMap<
            sha2::digest::array::Array<u8, typenum::U32>,
            String,
        >,
    ) {
        if !p_hash_this.exists() {println!("Error: Path does not exist"); return;}

        let path_str = match p_hash_this.to_str() {
            Some(s) => s,
            None => {println!("Error converting path to string"); return;}
        };

        let c_path = match std::ffi::CString::new(path_str) {
            Ok(c) => c,
            Err(_) => {println!("Null byte found in path string"); return;}
        };

        // Open input stream directly passing the &CString as &CStr via rsmpeg::ffi compatibility
        unsafe {
            match rsmpeg::avformat::AVFormatContextInput::open(c_path.as_c_str()) {
                Ok(mut con) => {
                    match rsmpeg::avformat::AVFormatContextInput::find_best_stream(
                        &con,
                        rsmpeg::ffi::AVMEDIA_TYPE_VIDEO,
                    ) {
                        Ok(_) => {
                            // Locate the first video stream index and corresponding decoder parameters
                            let s: Option<usize> = con
                                .streams
                                .as_mut()
                                .iter()
                                .position(|stream: &&mut *mut rsmpeg::ffi::AVStream| {
                                    stream
                                        .as_mut()
                                        .expect("sfds")
                                        .codecpar
                                        .as_mut()
                                        .expect("sdfsd")
                                        .codec_type
                                        == rsmpeg::ffi::AVMEDIA_TYPE_VIDEO
                                });

                            let mut s_i: i32 = -1;
                            let duration = con.duration;
                            let target_ts = if duration > 0 { duration / 4 } else { 0 };

                            match s {
                                Some(stream_index) => {s_i = stream_index as i32;}
                                None => {println!("Error returning the stream and codec!");}
                            }

                            match rsmpeg::avformat::AVFormatContextInput::seek(
                                &mut con,
                                s_i,
                                target_ts,
                                rsmpeg::ffi::AVSEEK_FLAG_BACKWARD as i32,
                            ) {
                                Ok(found_frame) => {println!("Found a frame {:#?}", found_frame);}
                                Err(e) => {println!("Unable to seek a frame {}", e);}
                            }
                        }

                        Err(e) => {println!("Error returning the stream and codec!");}
                    }
                }
                Err(e) => {println!("Error creating the AVFormatContext: {:?}", e); return;}
            };
        }
        println!("Finished the process for the file {:#?}", p_hash_this);
    }
}

pub mod s_hashing {
    use sha2::{Digest, digest::array::Array};

    // Given given SHA256 digest, path name (String), and BTreeMap with (fixed Sha256 digest as key and file name with String)
    pub fn compare_hash_library(
        hash_to_check: Array<u8, typenum::U32>,
        file_name: &str,
        check_here: &mut std::collections::BTreeMap<Array<u8, typenum::U32>, String>,
    ) {
        if let Some(dup) = check_here.get(&hash_to_check) {
            println!(
                "Found Duplicate: {:#?} is a duplicate of {:#?}",
                file_name, dup
            );
        } else {
            println!("New entry {:#?} for {}", hash_to_check, file_name);
            check_here.insert(hash_to_check, file_name.to_string());
        }
    }

    // SHA256 Hashing given path
    pub fn add_hash_from_path(
        path_name: &std::path::Path,
        add_here: &mut std::collections::BTreeMap<Array<u8, typenum::U32>, String>,
    ) {
        let fd = std::fs::File::open(path_name);
        match fd {
            Ok(mut f_buff) => {
                let mut byte_buff: Vec<u8> = std::vec::Vec::new();
                let f_n = f_buff.metadata();
                match f_n {
                    Ok(_) => {
                        let fb = std::io::Read::read_to_end(&mut f_buff, &mut byte_buff);
                        match fb {
                            Ok(_) => {
                                let i = sha2::Sha256::digest(byte_buff);
                                compare_hash_library(
                                    i,
                                    path_name.to_str().expect("Couldn't deterimine the path"),
                                    add_here,
                                );
                            }
                            Err(e) => {println!("Error {}", e);}
                        }
                    }
                    Err(e) => {println!("Error {}", e);}
                }
            }
            Err(e) => {println!("Error {}", e);}
        }
    }
}

pub mod misc {
    use crate::utils::{self, p_hashing};

    pub fn go_thru_dir(
        walk_thru: &mut std::path::Path,
        add_here: &mut std::collections::BTreeMap<
            sha2::digest::array::Array<u8, typenum::U32>,
            String,
        >,
        method: &str,
    ) {
        let get_metadata = std::fs::metadata(&mut *walk_thru);
        match get_metadata {
            Ok(file_descriptor_info) => {
                if file_descriptor_info.is_dir() {
                    let dir = std::fs::read_dir(walk_thru);
                    match dir {
                        Ok(go_thru) => {
                            for file in go_thru {
                                match file {
                                    Ok(try_this) => {
                                        go_thru_dir(&mut try_this.path(), add_here, method);
                                    }
                                    Err(e) => {println!("Error {}", e);}
                                }
                            }
                        }
                        Err(e) => {println!("Error {}", e);}
                    }
                } else if file_descriptor_info.is_file() {
                    let fd = std::fs::File::open(&walk_thru);
                    match fd {
                        Ok(_) => {
                            if method.eq("p") {
                                p_hashing::add_p_hash_for_media(&walk_thru, add_here);
                            } else if method.eq("b") {
                                utils::s_hashing::add_hash_from_path(&walk_thru, add_here);
                            } else {
                                println!(
                                    "Unable to match the method {:#?} with one of the options",
                                    method
                                );
                            }
                        }
                        Err(e) => {println!("Error in Computing Hash From Path{}", e);}
                    }
                }
            }
            Err(e) => {println!("Error Raised: {:#?}", e);}
        }
    }

    const _GIG_FILE_SIZE: u64 = 0x40000000;
    pub fn _find_and_set_ram_limit() {
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            let mut sys_mut_lim_struct: libc::rlimit = libc::rlimit {
                rlim_cur: _GIG_FILE_SIZE,
                rlim_max: _GIG_FILE_SIZE * 4,
            };
            let set_the_limit =
                unsafe { libc::setrlimit(libc::RLIMIT_DATA, &mut sys_mut_lim_struct) };
            match set_the_limit {
                0 => {
                    println!("Limit has been properly set {:#?}", sys_mut_lim_struct)
                }
                -1 => {
                    println!(
                        "An Error has occurred. Try Debugging {:#?}",
                        std::io::Error::last_os_error().raw_os_error()
                    )
                }
                _ => {
                    println!("Something went horribly wrong!");
                }
            }

            // Attempts to get the Resource Limits of RAM
            let get_the_limit =
                unsafe { libc::getrlimit(libc::RLIMIT_DATA, &mut sys_mut_lim_struct) };
            match get_the_limit {
                0 => {
                    println!(
                        "Limit has been properly retrieved Cur:{:#?}\nMax: {:#?}",
                        sys_mut_lim_struct.rlim_cur, sys_mut_lim_struct.rlim_max
                    )
                }
                -1 => {
                    println!("An Error has occurred. Try Debugging ")
                }
                _ => {
                    println!("Something went horribly wrong!");
                }
            }
        }
    }
}
