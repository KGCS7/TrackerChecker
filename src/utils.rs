pub mod p_hashing {
    use crate::utils::{self, misc};
    use rayon::prelude::*;

    pub struct PHashData {
        pub original_path: std::path::PathBuf,
        pub temp_path: std::path::PathBuf,
        pub image_d_hash: fast_dhash::Dhash,
    }

    pub fn add_p_hash_for_media(p_hash_this: &std::path::Path) -> Option<PHashData> {
        if !p_hash_this.exists() {
            println!("Error: Path does not exist");
            return None;
        }

        let extension = p_hash_this.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        match extension {
            "png" | "jpg" | "jpeg" => {
                let temp_dir = std::path::Path::new("./tempDir/");
                if let Err(e) = std::fs::create_dir_all(temp_dir) {
                    println!("Error creating the temporary directory: {e}");
                    return None;
                }

                let file_name = match p_hash_this.file_name() {
                    Some(name) => name,
                    None => {
                        println!("Problem retrieving the filename");
                        return None;
                    }
                };

                let destination = temp_dir.join(file_name);
                match std::fs::read(p_hash_this) {
                    Ok(bytes) => {
                        if let Err(e) = std::fs::write(&destination, bytes) {
                            println!("Error in transferring file {:#?}: {e}", file_name);
                            return None;
                        }
                    }
                    Err(e) => {
                        println!("Error reading file {:#?}: {e}", file_name);
                        return None;
                    }
                }

                match image::open(&destination) {
                    Ok(img) => {
                        let hash = return_d_hash(img);
                        Some(PHashData {
                            original_path: p_hash_this.to_path_buf(),
                            temp_path: destination,
                            image_d_hash: hash,
                        })
                    }
                    Err(e) => {
                        println!("Error opening copied image: {e}");
                        None
                    }
                }
            }
            "mkv" | "mp4" => {
                let c_path = match std::ffi::CString::new(p_hash_this.to_str().expect("Error converting path to string")) {
                    Ok(c) => c,
                    Err(_) => {
                        println!("Null byte found in path string");
                        return None;
                    }
                };

                unsafe {
                    let mut fmt_ctx_ptr: *mut ffmpeg_sys_next::AVFormatContext = std::ptr::null_mut();
                    if ffmpeg_sys_next::avformat_open_input(
                        &mut fmt_ctx_ptr,
                        c_path.as_ptr(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    ) < 0
                    {
                        println!("Error opening video format context");
                        return None;
                    }

                    if ffmpeg_sys_next::avformat_find_stream_info(fmt_ctx_ptr, std::ptr::null_mut()) < 0 {
                        println!("Error finding stream info");
                        ffmpeg_sys_next::avformat_close_input(&mut fmt_ctx_ptr);
                        return None;
                    }

                    let stream_index = ffmpeg_sys_next::av_find_best_stream(
                        fmt_ctx_ptr,
                        ffmpeg_sys_next::AVMediaType::AVMEDIA_TYPE_VIDEO,
                        -1,
                        -1,
                        std::ptr::null_mut(),
                        0,
                    );

                    if stream_index < 0 {
                        println!("No video stream found.");
                        ffmpeg_sys_next::avformat_close_input(&mut fmt_ctx_ptr);
                        return None;
                    }

                    let stream = *(*fmt_ctx_ptr).streams.add(stream_index as usize);
                    let codec_id = (*(*stream).codecpar).codec_id;
                    let decoder = ffmpeg_sys_next::avcodec_find_decoder(codec_id);

                    if decoder.is_null() {
                        println!("Failed to find decoder.");
                        ffmpeg_sys_next::avformat_close_input(&mut fmt_ctx_ptr);
                        return None;
                    }

                    let decode_context = ffmpeg_sys_next::avcodec_alloc_context3(decoder);
                    if decode_context.is_null() {
                        println!("Failed to allocate decoder context.");
                        ffmpeg_sys_next::avformat_close_input(&mut fmt_ctx_ptr);
                        return None;
                    }

                    if ffmpeg_sys_next::avcodec_parameters_to_context(
                        decode_context,
                        (*stream).codecpar,
                    ) < 0
                    {
                        println!("Failed to apply codec parameters.");
                        ffmpeg_sys_next::avcodec_free_context(&mut (decode_context as *mut _));
                        ffmpeg_sys_next::avformat_close_input(&mut fmt_ctx_ptr);
                        return None;
                    }

                    if ffmpeg_sys_next::avcodec_open2(decode_context, decoder, std::ptr::null_mut()) < 0 {
                        println!("Failed to open decoder.");
                        ffmpeg_sys_next::avcodec_free_context(&mut (decode_context as *mut _));
                        ffmpeg_sys_next::avformat_close_input(&mut fmt_ctx_ptr);
                        return None;
                    }

                    let duration = (*fmt_ctx_ptr).duration;
                    let target_ts = if duration > 0 { duration / 4 } else { 0 };

                    let _ = ffmpeg_sys_next::av_seek_frame(
                        fmt_ctx_ptr,
                        -1,
                        target_ts,
                        ffmpeg_sys_next::AVSEEK_FLAG_BACKWARD as i32,
                    );

                    let packet = ffmpeg_sys_next::av_packet_alloc();
                    let frame = ffmpeg_sys_next::av_frame_alloc();
                    let mut result = None;

                    while ffmpeg_sys_next::av_read_frame(fmt_ctx_ptr, packet) >= 0 {
                        if (*packet).stream_index == stream_index {
                            if ffmpeg_sys_next::avcodec_send_packet(decode_context, packet) >= 0 {
                                if ffmpeg_sys_next::avcodec_receive_frame(decode_context, frame) == 0 {
                                    let width = (*frame).width;
                                    let height = (*frame).height;

                                    let mut dst_buf = vec![0u8; (width as usize) * (height as usize) * 3];
                                    let mut dst_data: [*mut u8; 8] = [std::ptr::null_mut(); 8];
                                    let mut dst_linesize: [i32; 8] = [0; 8];

                                    dst_data[0] = dst_buf.as_mut_ptr();
                                    dst_linesize[0] = 3 * width;

                                    let frame_format: ffmpeg_sys_next::AVPixelFormat;
                                    match frame.as_ref().expect("msg").format {
                                        num => {
                                            frame_format = std::mem::transmute(num);
                                        }
                                    }

                                    let sws_ctx = ffmpeg_sys_next::sws_getContext(
                                        width,
                                        height,
                                        frame_format,
                                        width,
                                        height,
                                        ffmpeg_sys_next::AVPixelFormat::AV_PIX_FMT_RGB24,
                                        ffmpeg_sys_next::SwsFlags::SWS_BILINEAR as i32,
                                        std::ptr::null_mut(),
                                        std::ptr::null_mut(),
                                        std::ptr::null_mut(),
                                    );

                                    if !sws_ctx.is_null() {
                                        let ret = ffmpeg_sys_next::sws_scale(
                                            sws_ctx,
                                            (*frame).data.as_ptr() as *const *const u8,
                                            (*frame).linesize.as_ptr(),
                                            0,
                                            height,
                                            dst_data.as_mut_ptr(),
                                            dst_linesize.as_ptr(),
                                        );

                                        ffmpeg_sys_next::sws_freeContext(sws_ctx);

                                        if ret > 0 {
                                            let temp_path = std::path::PathBuf::from(&misc::temp_path_for_p_hash(p_hash_this));
                                            misc::create_temp_dir(temp_path.clone());

                                            if image::save_buffer_with_format(
                                                &temp_path,
                                                &dst_buf,
                                                width as u32,
                                                height as u32,
                                                image::ColorType::Rgb8,
                                                image::ImageFormat::Png,
                                            ).is_ok() {
                                                if let Ok(img) = image::open(&temp_path) {
                                                    let hash = return_d_hash(img);
                                                    result = Some(PHashData {
                                                        original_path: p_hash_this.to_path_buf(),
                                                        temp_path,
                                                        image_d_hash: hash,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    ffmpeg_sys_next::av_packet_unref(packet);
                                    break;
                                }
                            }
                        }
                        ffmpeg_sys_next::av_packet_unref(packet);
                    }

                    ffmpeg_sys_next::av_frame_free(&mut (frame as *mut _));
                    ffmpeg_sys_next::av_packet_free(&mut (packet as *mut _));
                    ffmpeg_sys_next::avcodec_free_context(&mut (decode_context as *mut _));
                    ffmpeg_sys_next::avformat_close_input(&mut fmt_ctx_ptr);

                    if result.is_none() {
                        println!("Failed to decode frame after seeking.");
                    }

                    result
                }
            }
            _ => {
                println!("Unsupported file type");
                None
            }
        }
    }

    pub fn find_collisions() {
        let file_paths = collect_files_recursively(std::path::Path::new("./tempDir"));

        if file_paths.is_empty() {
            println!("No files found in ./tempDir");
            return;
        }

        let processed_images = process_entries_parallel(&file_paths);
        let store_dup = compare_hashes_parallel(&processed_images);

        if let Ok(r_d) = std::fs::read_dir(".") {
            for subdir in r_d.flatten() {
                if let Ok(ft) = subdir.file_type() {
                    if ft.is_dir() {
                        let temp_dir_name = subdir.file_name();
                        if temp_dir_name != "tempDir" {
                            if let Ok(s) = std::fs::read_dir(&temp_dir_name) {
                                remove_from_dir(s, &store_dup, "sdf");
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_files_recursively(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    files.extend(collect_files_recursively(&path));
                } else if path.is_file() {
                    files.push(path);
                }
            }
        }
        files
    }

    fn process_entries_parallel(file_paths: &[std::path::PathBuf]) -> Vec<PHashData> {
        file_paths
            .par_iter()
            .filter_map(|path| {
                if let Some(file_name) = path.file_name() {
                    println!("Current File we are processing: {:#?}", file_name);
                }
                match image::open(path) {
                    Ok(img_open) => {
                        let img_hash = utils::p_hashing::return_d_hash(img_open);
                        Some(PHashData {
                            original_path: path.clone(),
                            temp_path: std::path::PathBuf::from(misc::temp_path_for_p_hash(path)),
                            image_d_hash: img_hash,
                        })
                    }
                    Err(_) => None,
                }
            })
            .collect()
    }

    fn compare_hashes_parallel(images: &[PHashData]) -> Vec<std::path::PathBuf> {
        if images.is_empty() {
            return Vec::new();
        }

        let mut collected_dup: Vec<std::path::PathBuf> = images
            .par_iter()
            .enumerate()
            .flat_map(|(i, target)| {
                images[i + 1..].par_iter().filter_map(move |comp| {
                    let h_d = target.image_d_hash.hamming_distance(&comp.image_d_hash);
                    let similarity = 1.0 - (h_d as f64 / 64.0);
                    if similarity >= 0.90 {
                        println!(
                            "\tFound similarity comparing {:?} to {:?}",
                            comp.temp_path, target.temp_path
                        );
                        Some(target.original_path.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();
        collected_dup.dedup();
        collected_dup
    }

    fn remove_from_dir(
        passed_dir: std::fs::ReadDir,
        passed_dup: &[std::path::PathBuf],
        method: &str,
    ) {
        for f in passed_dir.flatten() {
            let file_name = f.file_name();
            let target_path = f.path();

            for p_d in passed_dup {
                let matches = p_d.file_name().map_or(false, |name| name == file_name);

                if !matches {
                    continue;
                }

                println!("Match found for: {file_name:?}");

                let mut should_delete = false;

                if method != "f" {
                    let options = eframe::NativeOptions {
                        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 450.0]),
                        ..Default::default()
                    };

                    let path_to_preview = target_path.clone();
                    let confirm_delete =
                        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                    let confirm_flag = confirm_delete.clone();

                    let mut preview_image = std::fs::read(&path_to_preview)
                        .ok()
                        .and_then(|bytes| image::load_from_memory(&bytes).ok())
                        .map(|image| {
                            let rgba = image.to_rgba8();
                            egui::ColorImage::from_rgba_unmultiplied(
                                [rgba.width() as usize, rgba.height() as usize],
                                rgba.as_raw(),
                            )
                        });
                    let mut preview_texture: Option<egui::TextureHandle> = None;

                    let _ = eframe::run_ui_native(
                        "Delete Confirmation",
                        options,
                        move |ctx, _frame| {
                            if preview_texture.is_none() {
                                if let Some(image) = preview_image.take() {
                                    preview_texture = Some(ctx.load_texture(
                                        "delete-preview",
                                        image,
                                        egui::TextureOptions::LINEAR,
                                    ));
                                }
                            }

                            egui::CentralPanel::default().show(ctx, |ui| {
                                ui.heading("Delete Photo Confirmation");
                                ui.label(format!("File: {}", path_to_preview.display()));
                                ui.add_space(10.0);

                                if let Some(texture) = preview_texture.as_ref() {
                                    ui.add(egui::Image::from_texture(texture).max_height(300.0));
                                } else {
                                    ui.label("Unable to load image preview.");
                                }

                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    if ui.button("Yes, Delete").clicked() {
                                        confirm_flag
                                            .store(true, std::sync::atomic::Ordering::Relaxed);
                                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                    if ui.button("Cancel").clicked() {
                                        confirm_flag
                                            .store(false, std::sync::atomic::Ordering::Relaxed);
                                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                });
                            });
                        },
                    );

                    should_delete = confirm_delete.load(std::sync::atomic::Ordering::Relaxed);
                } else {
                    #[cfg(target_os = "macos")]
                    {
                        if let Err(e) = std::process::Command::new("open").arg(&target_path).spawn()
                        {
                            println!("Error opening file for preview: {e}");
                        }
                    }
                    #[cfg(target_os = "windows")]
                    {
                        if let Err(e) = std::process::Command::new("cmd").args(["/C", "start", "", target_path.to_str().unwrap_or("")]).spawn()
                        {
                            println!("Error opening file for preview: {e}");
                        }
                    }
                    should_delete = true;
                }

                if should_delete {
                    match std::fs::remove_file(&target_path) {
                        Ok(_) => println!("Deleted: {file_name:?}"),
                        Err(e) => println!("Error deleting file: {e}"),
                    }
                }
            }
        }
    }

    pub fn return_d_hash(image: image::DynamicImage) -> fast_dhash::Dhash {
        let i_w = image.width();
        let i_h = image.height();
        let rgb_image = image.to_rgb8();
        fast_dhash::Dhash::new(&rgb_image.into_raw(), i_w, i_h, 3)
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
    use std::str::FromStr;

use crate::utils;

   pub fn create_temp_dir(extract_from_this_path: std::path::PathBuf) {
        if let Some(new_dir) = extract_from_this_path.parent() {
            if !new_dir.is_dir() {
                std::fs::create_dir_all(new_dir).expect("msg");
            }
        }
    }

    pub fn temp_path_for_p_hash(passed_path: &std::path::Path) -> String {
        let mut path_str = String::new();

        for component in passed_path.components() {
            if let std::path::Component::Normal(os_str) = component {
                if let Some(s) = os_str.to_str() {
                    if !path_str.is_empty() {
                        path_str.push('_');
                    }
                    path_str.push_str(s);
                }
            }
        }

        if let Some(stem) = passed_path.file_stem().and_then(|s| s.to_str()) {
            if let Some(idx) = path_str.rfind(stem) {
                path_str.truncate(idx + stem.len());
            }
        }

        #[cfg(target_os = "windows")]
        {
            path_str = path_str.replace('\\', "_");
        }

        #[cfg(target_os = "macos")]
        {
            path_str = path_str.replace('/', "_");
        }

        format!("./tempDir/{} copy.png", path_str)
    }
    
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
