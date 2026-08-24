pub mod p_hashing {
    use crate::utils;
    use rayon::prelude::*;
    //Struct for p hash data
    struct PHashData {
        image_path: std::path::PathBuf,
        image_d_hash: fast_dhash::Dhash,
    }

    pub fn add_p_hash_for_media(p_hash_this: &std::path::Path) {
        if !p_hash_this.exists() { std::println!("Error: Path does not exist"); return; }

        let temp_dir: &std::path::Path = std::path::Path::new("./tempDir/");
        let c_path = match std::ffi::CString::new(p_hash_this.to_str().expect("Error converting path to string"),) {
            Ok(c) => c,
            Err(_) => { std::println!("Null byte found in path string"); return; }
        };

        match p_hash_this
            .extension()
            .expect("Error extracting the file extension")
            .to_str()
            .expect("Error converting the file extension into a string")
        {
            "png" | "jpg" | "jpeg" => {
                match std::fs::copy(
                    p_hash_this,
                    temp_dir.join(p_hash_this.file_name().expect("msg")),
                ) {
                    Ok(transfered_file) => {
                        if !transfered_file.ge(&0) {
                            std::println!("No bytes transferred");
                        }
                    }
                    Err(e) => {
                        std::println!(
                            "Error in transferring the file {:#?}: {e}",
                            &p_hash_this.file_name()
                        );
                    }
                };
            }
            "mkv" | "mp4" => {
                // Start of the Video Decoding
                let mut format_context =
                    match rsmpeg::avformat::AVFormatContextInput::open(c_path.as_c_str()) {
                        Ok(c) => c,
                        Err(e) => {
                            std::println!("Error: {e}");
                            return;
                        }
                    };

                let (stream_index, decoder) =
                    match format_context.find_best_stream(rsmpeg::ffi::AVMEDIA_TYPE_VIDEO) {
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
                            // Convert the decoded frame to RGB24 using libswscale via rsmpeg::ffi
                            let width = frame.width as i32;
                            let height = frame.height as i32;

                            // prepare destination buffer (RGB24)
                            let mut dst_buf: Vec<u8> =
                                vec![0u8; (width as usize) * (height as usize) * 3];
                            let mut dst_data: [*mut u8; 8] = [std::ptr::null_mut(); 8];
                            let mut dst_linesize: [i32; 8] = [0; 8];
                            dst_data[0] = dst_buf.as_mut_ptr();
                            dst_linesize[0] = (3 * width) as i32;

                            // source pointers and linesizes from the decoded frame
                            let src_data: [*mut u8; 8] = frame.data;
                            let src_linesize: [i32; 8] = frame.linesize;

                            // determine source pixel format from frame.format (i32)
                            let src_pix_fmt = unsafe {
                                std::mem::transmute::<i32, rsmpeg::ffi::AVPixelFormat>(frame.format)
                            };

                            // create sws context
                            let sws_ctx = unsafe {
                                rsmpeg::ffi::sws_getContext(
                                    width,
                                    height,
                                    src_pix_fmt,
                                    width,
                                    height,
                                    rsmpeg::ffi::AV_PIX_FMT_RGB24,
                                    rsmpeg::ffi::SWS_BILINEAR as i32,
                                    std::ptr::null_mut(),
                                    std::ptr::null_mut(),
                                    std::ptr::null_mut(),
                                )
                            };

                            if sws_ctx.is_null() {
                                std::println!("Failed to create sws context");
                                return;
                            }

                            // perform conversion
                            let ret = unsafe {
                                rsmpeg::ffi::sws_scale(
                                    sws_ctx,
                                    src_data.as_ptr() as *const *const u8,
                                    src_linesize.as_ptr(),
                                    0,
                                    height,
                                    dst_data.as_mut_ptr() as *mut *mut u8,
                                    dst_linesize.as_ptr(),
                                )
                            };

                            // free sws context
                            unsafe { rsmpeg::ffi::sws_freeContext(sws_ctx) };

                            if ret <= 0 {
                                std::println!("sws_scale failed or produced no output: {}", ret);
                                continue;
                            }

                            // ensure output directory exists
                            if let Err(e) = std::fs::create_dir_all("./tempDir") {
                                std::println!("Failed to create ./tempDir: {e}");
                                continue;
                            }

                            // build output path: ./tempDir/{originalName}_copy.png
                            let stem = match p_hash_this
                                .file_stem()
                                .and_then(|s| s.to_str().map(|s| s.to_owned()))
                            {
                                Some(s) => s,
                                None => {
                                    std::println!("Error extracting file stem");
                                    continue;
                                }
                            };

                            let out_path = format!("./tempDir/{}_copy.png", stem);

                            // save buffer as PNG (RGB8)
                            match image::save_buffer_with_format(
                                &out_path,
                                &dst_buf,
                                width as u32,
                                height as u32,
                                image::ColorType::Rgb8,
                                image::ImageFormat::Png,
                            ) {
                                Ok(saved_image) => saved_image,
                                Err(e) => {
                                    std::println!("Error saving image: {e}");
                                    continue;
                                }
                            }
                            // `frame` (AVFrame) contains raw image buffer data (e.g. YUV/RGB buffers in `frame.data`)
                            // Implement perceptual hashing algorithms (e.g., ImageHash / DCT) using the decoded frame data.
                            frame_decoded = true;
                            break;
                        }
                        Err(rsmpeg::error::RsmpegError::DecoderFlushedError) | Err(_) => {continue;}
                        Err(e) => { std::println!("Error receiving frame from decoder: {e}"); continue; }
                    }
                }

                if !frame_decoded {
                    std::println!("Failed to decode frame after seeking.");
                    return;
                }
            }
            _ => {
                std::println!("Unsupported file type")
            }
        }
        return;
    }

    pub fn find_collisions() {
        // Collect directory entries into owned paths to avoid holding locks during parallel execution
        let entries: std::vec::Vec<std::fs::DirEntry> = std::fs::read_dir("./tempDir")
            .expect("Wasn't able to find or read the ./tempDir")
            .map(|res| res.expect("Failed to read directory entry"))
            .collect();

        // Process all images to compute hashes in parallel using join
        let processed_images = process_entries_parallel(&entries[..]);

        // Perform pairwise comparisons in parallel using join
        let _store_dup = compare_hashes_parallel(&processed_images[..]);

        match std::fs::read_dir(".") {
            Ok(r_d) => {
                for subdir in r_d.into_iter() {
                    let dir_entry = subdir.expect("Error returning the DirEntry");
                    if dir_entry.file_type().expect("msg").is_dir() {
                        let temp_dir_name = dir_entry.file_name();
                        if !temp_dir_name.eq("tempDir") {
                            std::fs::read_dir(temp_dir_name).expect("msg");
                        }
                    } else { //File type
                    };
                }
            }
            Err(e) => {std::println!("Error: {e}")}
        }

        return;
    }

    fn process_entries_parallel(entries: &[std::fs::DirEntry]) -> Vec<PHashData> {
        if entries.is_empty() {
            return Vec::new();
        }
        if entries.len() == 1 {
            let entry = &entries[0];
            let path = entry.path();
            println!("Current File we are processing: {:#?}", entry.file_name());

            let img_open = image::open(&path).expect("Failed to open image");
            let img_hash = utils::p_hashing::return_d_hash(img_open);
            return vec![PHashData {image_path: path,image_d_hash: img_hash}];
        }

        let mid = entries.len() / 2;
        let (left, right) = entries.split_at(mid);

        let (mut left_res, right_res) = rayon::join(
            || process_entries_parallel(left),
            || process_entries_parallel(right),
        );

        left_res.extend(right_res);
        left_res
    }

   fn compare_hashes_parallel(images: &[PHashData]) -> Vec<std::path::PathBuf> {
    if images.is_empty() {return Vec::new(); }
    
    let mut collected_dup:std::vec::Vec<std::path::PathBuf> = images
        .par_iter()
        .enumerate()
        .flat_map(|(i, target)| {
            images[i + 1..].par_iter().filter_map(move |comp| {
                let h_d = target.image_d_hash.hamming_distance(&comp.image_d_hash);
                let similarity = 1.0 - (h_d as f64 / 64.0);
                if similarity >= 0.90 {
                    println!(
                        "\tFound similarity comparing {:?} to {:?}",
                        comp.image_path, target.image_path
                    );
                    Some(target.image_path.clone())
                } else {
                    None
                }
            })
        })
        .collect();
        collected_dup.dedup();
        collected_dup
    
    }

    pub fn return_d_hash(image: image::DynamicImage) -> fast_dhash::Dhash {
        let i_w = image.width(); let i_h = image.height(); let i_c = image.color().channel_count();
        return fast_dhash::Dhash::new(&image.into_bytes(), i_w, i_h, i_c);
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
                file_name,
                dup
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
                            Err(e) => {
                                std::println!("Error {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        std::println!("Error {}", e);
                    }
                }
            }
            Err(e) => {
                std::println!("Error {}", e);
            }
        }
    }
}

pub mod misc {
    use crate::utils;

    // Takes a boolean as an argument such that if true then we delete "./tempDir/" and then attempt to create it again
    // if false, then we simply try to delete "./tempDir/""
    pub fn flush_the_cache(mode: bool) {
        let t_b: &std::path::Path = std::path::Path::new("./tempDir/");
        if mode.eq(&true) {
            if std::path::Path::exists(t_b) {
                match std::fs::remove_dir_all(std::path::Path::new("./tempDir/")) {
                    Ok(_) => {
                        std::println!("Removed the cached.");
                    }
                    Err(e) => {
                        std::println!("Error deleting the cached files {e}.");
                    }
                }
            };
            match std::fs::DirBuilder::new()
                .recursive(true)
                .create(std::path::Path::new("./tempDir/"))
            {
                Ok(d) => d,
                Err(e) => {
                    std::println!("Error in creating the tempDir: {e}");
                    return;
                }
            };
        } else {
            match std::fs::remove_dir_all(std::path::Path::new("./tempDir/")) {
                Ok(_) => {
                    std::println!("Removed the cached.");
                }
                Err(e) => {
                    std::println!("Error deleting the cached files {e}.");
                }
            }
        }
    }

    pub fn go_thru_dir(
        walk_thru: &mut std::path::Path,
        add_here: &mut std::collections::BTreeMap<
            sha2::digest::array::Array<u8, typenum::U32>,
            String,
        >,
        method: &str,
    ) {
        match std::fs::metadata(&mut *walk_thru) {
            Ok(file_descriptor_info) => {
                if file_descriptor_info.is_dir() {
                    match std::fs::read_dir(walk_thru) {
                        Ok(go_thru) => {
                            for file in go_thru {
                                match file {
                                    Ok(try_this) => {
                                        go_thru_dir(&mut try_this.path(), add_here, method);
                                    }
                                    Err(e) => {
                                        std::println!("Error {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            std::println!("Error {}", e);
                        }
                    }
                } else if file_descriptor_info.is_file() {
                    match std::fs::File::open(&walk_thru) {
                        Ok(_) => {
                            if method.eq("p") {
                                utils::p_hashing::add_p_hash_for_media(&walk_thru);
                            } else if method.eq("b") {
                                utils::s_hashing::add_hash_from_path(&walk_thru, add_here);
                            } else {
                                std::println!(
                                    "Unable to match the method {:#?} with one of the options",
                                    method
                                );
                            }
                        }
                        Err(e) => {
                            std::println!("Error in Computing Hash From Path{}", e);
                        }
                    }
                }
            }
            Err(e) => {
                std::println!("Error Raised: {:#?}", e);
            }
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
                        sys_mut_lim_struct.rlim_cur,
                        sys_mut_lim_struct.rlim_max
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
