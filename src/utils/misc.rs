use crate::utils;
const _GIG_FILE_SIZE: u64 = 0x40000000;

pub(crate) trait DirectoryHashMap {
    fn process_file(
        &mut self,
        path: std::path::PathBuf,
        method: &str,
        bitwise_paths: &mut Vec<std::path::PathBuf>,
    );
}

impl DirectoryHashMap
    for std::collections::BTreeMap<
        sha2::digest::array::Array<u8, typenum::U32>,
        utils::p_hashing::PHashData,
    >
{
    fn process_file(
        &mut self,
        path: std::path::PathBuf,
        method: &str,
        _bitwise_paths: &mut Vec<std::path::PathBuf>,
    ) {
        if method == "p" {
            utils::p_hashing::add_p_hash_for_media(&path);
        } else {
            std::println!(
                "Unable to match the method {:#?} with one of the options",
                method
            );
        }
    }
}

impl<'a> DirectoryHashMap
    for std::collections::BTreeMap<sha2::digest::array::Array<u8, typenum::U32>, &'a str>
{
    fn process_file(
        &mut self,
        path: std::path::PathBuf,
        method: &str,
        bitwise_paths: &mut Vec<std::path::PathBuf>,
    ) {
        if method == "b" {
            bitwise_paths.push(path);
        } else {
            std::println!(
                "Unable to match the method {:#?} with one of the options",
                method
            );
        }
    }
}

pub fn create_temp_dir(extract_from_this_path: &std::path::Path) {
    if let Some(new_dir) = extract_from_this_path.parent() {
        if !new_dir.is_dir() {
            std::fs::create_dir_all(new_dir).expect("msg");
        }
    }
}

pub fn truncated_screenshot_path(original_path: &std::path::Path) -> std::path::PathBuf {
    let current_dir = std::env::current_dir().expect("Unable to determine the current directory");
    let relative_path = original_path
        .strip_prefix(&current_dir)
        .unwrap_or(original_path);
    if !relative_path.as_os_str().is_empty() && relative_path != original_path {
        return relative_path.to_path_buf();
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        for (marker_index, component) in original_path.components().enumerate() {
            let is_user_directory = match component {
                std::path::Component::Normal(name) => name == "Users" || name == "home",
                _ => false,
            };
            let first_relative_component = marker_index + 2;
            if is_user_directory {
                let mut relative_path = std::path::PathBuf::new();
                for component in original_path.components().skip(first_relative_component) {
                    relative_path.push(component.as_os_str());
                }
                if !relative_path.as_os_str().is_empty() {
                    return relative_path;
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::path::Component;

        for (marker_index, component) in original_path.components().enumerate() {
            let is_user_directory = match component {
                Component::Normal(name) => name.eq_ignore_ascii_case(std::ffi::OsStr::new("Users")),
                _ => false,
            };
            let first_relative_component = marker_index + 2;
            if is_user_directory {
                let mut relative_path = std::path::PathBuf::new();
                for component in original_path.components().skip(first_relative_component) {
                    relative_path.push(component.as_os_str());
                }
                if !relative_path.as_os_str().is_empty() {
                    return relative_path;
                }
            }
        }
    }

    original_path.to_path_buf()
}

pub fn temp_path_for_p_hash(passed_path: &std::path::Path) -> String {
    #[cfg(target_os = "windows")]
    {
        use std::path::{Component, Prefix};

        let mut preview_path = std::path::PathBuf::from("./tempDir");
        let mut components = passed_path.components();
        match components.next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => {
                    preview_path.push(format!("drive_{}", char::from(drive)));
                }
                Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => {
                    preview_path.push("unc");
                    preview_path.push(server);
                    preview_path.push(share);
                }
                _ => preview_path.push("other"),
            },
            Some(component) => {
                preview_path.push("relative");
                match component {
                    Component::RootDir | Component::CurDir | Component::Prefix(_) => {}
                    Component::ParentDir => preview_path.push("_parent"),
                    Component::Normal(name) => preview_path.push(name),
                }
            }
            None => preview_path.push("relative"),
        }

        for component in components {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::Prefix(_) => unreachable!("path prefix must be the first component"),
                Component::ParentDir => preview_path.push("_parent"),
                Component::Normal(name) => preview_path.push(name),
            }
        }

        let Some(file_name) = preview_path.file_name() else {
            return String::new();
        };
        let mut preview_name = file_name.to_os_string();
        preview_name.push(" copy.png");
        preview_path.set_file_name(preview_name);
        return preview_path.to_string_lossy().into_owned();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut relative_path = truncated_screenshot_path(passed_path);
        if relative_path.is_absolute() {
            let mut fallback_path = std::path::PathBuf::new();
            if let Some(parent_name) = passed_path.parent().and_then(std::path::Path::file_name) {
                fallback_path.push(parent_name);
            }
            if let Some(file_name) = passed_path.file_name() {
                fallback_path.push(file_name);
            }
            relative_path = fallback_path;
        }

        let mut preview_path = std::path::PathBuf::from("./tempDir");
        preview_path.push(relative_path);
        let Some(file_name) = preview_path.file_name() else {
            return String::new();
        };
        let mut preview_name = file_name.to_os_string();
        preview_name.push(" copy.png");
        preview_path.set_file_name(preview_name);
        preview_path.to_string_lossy().into_owned()
    }
}

pub fn source_path_from_preview(preview_path: &std::path::Path) -> Option<std::path::PathBuf> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let relative_path = preview_path.strip_prefix("./tempDir").ok()?;
        let preview_name = relative_path.file_name()?.to_str()?;
        let source_name = preview_name.strip_suffix(" copy.png")?;
        let mut relative_source = relative_path.parent()?.to_path_buf();
        relative_source.push(source_name);
        let current_dir = std::env::current_dir().ok()?;
        let source_path = current_dir.join(&relative_source);
        if source_path.exists() {
            return Some(source_path);
        }

        let home_dir = std::env::var_os("HOME")?;
        let source_path = std::path::PathBuf::from(home_dir).join(relative_source);
        source_path.exists().then_some(source_path)
    }

    #[cfg(target_os = "windows")]
    {
        let normalized_path = preview_path.to_string_lossy().replace('\\', "/");
        let relative_path = normalized_path.strip_prefix("./tempDir/")?;
        let mut components: Vec<_> = relative_path.split('/').collect();
        let preview_name = components.pop()?;
        let source_name = preview_name.strip_suffix(" copy.png")?;
        let root = components.first().copied()?;

        let mut source_path = if let Some(drive) = root.strip_prefix("drive_") {
            components.remove(0);
            std::path::PathBuf::from(format!("{drive}:\\"))
        } else if root == "unc" {
            if components.len() < 3 {
                return None;
            }
            components.remove(0);
            let server = components.remove(0);
            let share = components.remove(0);
            std::path::PathBuf::from(format!("\\\\{server}\\{share}"))
        } else if root == "relative" {
            components.remove(0);
            std::path::PathBuf::new()
        } else {
            components.remove(0);
            std::path::PathBuf::from(root)
        };

        for component in components {
            source_path.push(component);
        }
        source_path.push(source_name);
        Some(source_path)
    }
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

pub fn go_thru_dir<T: DirectoryHashMap>(
    walk_thru: std::path::PathBuf,
    add_here: &mut T,
    method: &str,
    bitwise_paths: &mut Vec<std::path::PathBuf>,
) {
    match std::fs::metadata(&walk_thru) {
        Ok(file_descriptor_info) => {
            if file_descriptor_info.is_dir() {
                match std::fs::read_dir(&walk_thru) {
                    Ok(go_thru) => {
                        for file in go_thru {
                            match file {
                                Ok(try_this) => {
                                    go_thru_dir(try_this.path(), add_here, method, bitwise_paths);
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
                        add_here.process_file(walk_thru, method, bitwise_paths);
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

pub fn _find_and_set_ram_limit() {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let mut sys_mut_lim_struct: libc::rlimit = libc::rlimit {
            rlim_cur: _GIG_FILE_SIZE,
            rlim_max: _GIG_FILE_SIZE * 4,
        };
        let set_the_limit = unsafe { libc::setrlimit(libc::RLIMIT_DATA, &mut sys_mut_lim_struct) };
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

        let get_the_limit = unsafe { libc::getrlimit(libc::RLIMIT_DATA, &mut sys_mut_lim_struct) };
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

pub mod delete_dup_preview {
    struct DeleteConfirmationApp {
        screenshot_path: std::path::PathBuf,
        confirm_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
        preview_image: Option<egui::ColorImage>,
        preview_texture: Option<egui::TextureHandle>,
    }

    impl eframe::App for DeleteConfirmationApp {
        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            if self.preview_texture.is_none() {
                if let Some(image) = self.preview_image.take() {
                    self.preview_texture = Some(ui.ctx().load_texture(
                        "delete-preview",
                        image,
                        egui::TextureOptions::LINEAR,
                    ));
                }
            }

            egui::CentralPanel::default().show(ui, |ui| {
                ui.heading("Delete source file?");
                ui.label(format!("File: {}", self.screenshot_path.display()));
                ui.add_space(10.0);

                if let Some(texture) = self.preview_texture.as_ref() {
                    ui.add(
                        egui::Image::from_texture(texture)
                            .max_width(360.0)
                            .max_height(300.0),
                    );
                } else {
                    ui.label("Unable to load image preview.");
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Yes, Delete").clicked() {
                        self.confirm_flag
                            .store(true, std::sync::atomic::Ordering::Relaxed);
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Cancel").clicked() {
                        self.confirm_flag
                            .store(false, std::sync::atomic::Ordering::Relaxed);
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        }
    }

    pub fn ask_to_delete(
        path_to_preview: &std::path::Path,
        screenshot_path: &std::path::Path,
    ) -> bool {
        let confirm_delete = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let confirm_flag = std::sync::Arc::clone(&confirm_delete);

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 450.0]),
            #[cfg(target_os = "linux")]
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        };

        let preview_image = match image::open(&path_to_preview) {
                Ok(image) => Some(image),
                Err(preview_error) => match image::open(path_to_preview) {
                    Ok(image) => Some(image),
                    Err(source_error) => {
                        eprintln!(
                            "Unable to load preview from ({preview_error}) or source {} ({source_error})",
                            path_to_preview.display()
                        );
                        None
                    }
                }
            }
            .map(|image| {
                let rgba = image.to_rgba8();
                egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as usize, rgba.height() as usize],
                    rgba.as_raw(),
                )
            });

        let app = DeleteConfirmationApp {
            screenshot_path: screenshot_path.to_path_buf(),
            confirm_flag,
            preview_image,
            preview_texture: None,
        };

        if let Err(error) = eframe::run_native(
            "Delete Confirmation",
            options,
            Box::new(|_cc| Ok(Box::new(app) as Box<dyn eframe::App>)),
        ) {
            eprintln!("Unable to open delete confirmation window: {error}");
        }

        confirm_delete.load(std::sync::atomic::Ordering::Relaxed)
    }
}
pub mod suppress_ffmpeg_messages {
    static FFMPEG_LOG_LEVEL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    pub struct FfmpegLogLevelGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        previous_level: i32,
    }

    impl FfmpegLogLevelGuard {
        pub fn suppress_messages() -> Self {
            let lock = FFMPEG_LOG_LEVEL_LOCK
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let previous_level = unsafe { ffmpeg_sys_next::av_log_get_level() };
            unsafe {
                ffmpeg_sys_next::av_log_set_level(ffmpeg_sys_next::AV_LOG_QUIET);
            }

            Self {
                _lock: lock,
                previous_level,
            }
        }
    }

    impl Drop for FfmpegLogLevelGuard {
        fn drop(&mut self) {
            unsafe {
                ffmpeg_sys_next::av_log_set_level(self.previous_level);
            }
        }
    }
}
