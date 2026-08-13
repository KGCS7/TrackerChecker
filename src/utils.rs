pub mod p_hashing {

    pub fn add_p_hash_for_media(
        p_hash_this: &std::path::Path,
        _check_here: &mut std::collections::BTreeMap<
            sha2::digest::array::Array<u8, typenum::U32>,
            String,
        >,
    ) {
        if !p_hash_this.exists() {
            std::println!("Error: Path does not exist");
            return;
        }

        let c_path = match std::ffi::CString::new(
            p_hash_this.to_str().expect("Error converting path to string"),
        ) {
            Ok(c) => c,
            Err(_) => {
                std::println!("Null byte found in path string");
                return;
            }
        };


        let mut format_context = match rsmpeg::avformat::AVFormatContextInput::open(c_path.as_c_str()) {
            Ok(c) => c,
            Err(e) => {
                std::println!("Error: {e}");
                return;
            }
        };


        let (stream_index, decoder) = match format_context.find_best_stream(rsmpeg::ffi::AVMEDIA_TYPE_VIDEO) {
            Ok(Some((idx, decoder))) => (idx, decoder),
            Ok(None) => {
                std::println!("No video stream found.");
                return;
            }
            Err(e) => {
                std::println!("Error finding video stream: {e}");
                return;
            }
        };


        let video_stream = &format_context.streams()[stream_index];
        let mut decode_context = rsmpeg::avcodec::AVCodecContext::new(&decoder);
        if let Err(e) = decode_context.apply_codecpar(&video_stream.codecpar()) {
            std::println!("Failed to apply codec parameters: {e}");
            return;
        }
        if let Err(e) = decode_context.open(None) {
            std::println!("Failed to open decoder: {e}");
            return;
        }


        let duration = format_context.duration;
        let target_ts = if duration > 0 { duration / 4 } else { 0 };

        if let Err(e) = format_context.seek(
            stream_index as i32,
            target_ts,
            rsmpeg::ffi::AVSEEK_FLAG_BACKWARD as i32,
        ) {
            std::println!("Unable to seek frame: {e}");
            return;
        }


        let mut frame_decoded = false;
        while let Ok(Some(packet)) = format_context.read_packet() {
            if packet.stream_index != stream_index as i32 {
                continue;
            }

            if let Err(e) = decode_context.send_packet(Some(&packet)) {
                std::println!("Error sending packet to decoder: {e}");
                break;
            }

            match decode_context.receive_frame() {
                Ok(frame) => {
                    std::println!(
                        "Successfully decoded frame! Dimensions: {}x{}",
                        frame.width,
                        frame.height
                    );
                    
                    // `frame` (AVFrame) contains raw image buffer data (e.g. YUV/RGB buffers in `frame.data`)
                    // Implement perceptual hashing algorithms (e.g., ImageHash / DCT) using the decoded frame data.
                    frame_decoded = true;
                    break;
                }
                Err(rsmpeg::error::RsmpegError::DecoderFlushedError)
                | Err(_) => {
                    continue;
                }
                Err(e) => {
                    std::println!("Error receiving frame from decoder: {e}");
                    break;
                }
            }
        }

        if !frame_decoded {
            std::println!("Failed to decode frame after seeking.");
            return;
        }

        std::println!("Finished processing file: {:?}", p_hash_this);
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
            std::println!(
                "Found Duplicate: {:#?} is a duplicate of {:#?}",
                file_name, dup
            );
        } else {
            std::println!("New entry {:#?} for {}", hash_to_check, file_name);
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
                            Err(e) => {std::println!("Error {}", e);}
                        }
                    }
                    Err(e) => {std::println!("Error {}", e);}
                }
            }
            Err(e) => {std::println!("Error {}", e);}
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
                                    Err(e) => {std::println!("Error {}", e);}
                                }
                            }
                        }
                        Err(e) => {std::println!("Error {}", e);}
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
                                std::println!(
                                    "Unable to match the method {:#?} with one of the options",
                                    method
                                );
                            }
                        }
                        Err(e) => {std::println!("Error in Computing Hash From Path{}", e);}
                    }
                }
            }
            Err(e) => {std::println!("Error Raised: {:#?}", e);}
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
                    std::println!("Limit has been properly set {:#?}", sys_mut_lim_struct)
                }
                -1 => {
                    std::println!(
                        "An Error has occurred. Try Debugging {:#?}",
                        std::io::Error::last_os_error().raw_os_error()
                    )
                }
                _ => {
                    std::println!("Something went horribly wrong!");
                }
            }

            // Attempts to get the Resource Limits of RAM
            let get_the_limit =
                unsafe { libc::getrlimit(libc::RLIMIT_DATA, &mut sys_mut_lim_struct) };
            match get_the_limit {
                0 => {
                    std::println!(
                        "Limit has been properly retrieved Cur:{:#?}\nMax: {:#?}",
                        sys_mut_lim_struct.rlim_cur, sys_mut_lim_struct.rlim_max
                    )
                }
                -1 => {
                    std::println!("An Error has occurred. Try Debugging ")
                }
                _ => {
                    std::println!("Something went horribly wrong!");
                }
            }
        }
    }
}
