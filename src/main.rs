#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
mod utils;

fn main() {
    let m = clap::Command::new("TorrentChecker")
        .version("1.0")
        .about("Finds duplicates of media by using a SHA256 hash or perceptual hashing algorithms.")
        .arg(
            clap::arg!(
                -p --perceptual_hash ["Path1 Path2 ... PathN"] "perceptual_hash"
            )
            .id("pc")
            .num_args(1..),
        )
        .arg(
            clap::arg!(
                -b --bitwise_hash ["Path1 Path2 ... PathN"] "bitwise_hash"
            )
            .id("bc")
            .num_args(1..),
        )
        .arg(
            clap::arg!(
                -x --skip ["Path1 Path2 ... PathN"] "skip paths from hashing"
            )
            .id("skip")
            .num_args(1..),
        )
        .arg(
            clap::arg!(-f - -force)
                .id("f")
                .num_args(1)
                .value_parser(["true", "false"])
                .default_missing_value("false")
                .default_value("false"),
        )
        .get_matches();

    let excluded_paths = m
        .get_raw("skip")
        .into_iter()
        .flatten()
        .map(std::path::PathBuf::from)
        .collect::<Vec<_>>();

    utils::misc::flush_the_cache(true);

    if m.contains_id("pc") {
        let mut bitwise_paths = Vec::new();
        let mut file_hash: std::collections::BTreeMap<
            sha2::digest::array::Array<u8, typenum::U32>,
            utils::p_hashing::PHashData> = std::collections::BTreeMap::new();
        match m.get_raw("pc") {
            Some(i) => {
                for file in i {
                    utils::misc::go_thru_dir(
                        std::path::PathBuf::from(file),
                        &mut file_hash,
                        "p",
                        &mut bitwise_paths,
                        &excluded_paths,
                    );
                }
            }
            None => {println!("Error parsing path in P Hash implementation of CLI args");}
        };
        utils::p_hashing::find_collisions();
    } else if m.contains_id("bc") {
        let mut bitwise_paths = Vec::new();
        let mut zero_tolerance_hash: std::collections::BTreeMap<
            sha2::digest::array::Array<u8, typenum::U32>,
            &str,
        > = std::collections::BTreeMap::new();
        let temp_variable: Option<clap::parser::RawValues<'_>> = m.get_raw("bc");
        match temp_variable {
            Some(i) => {
                for file in i {
                    println!("Found path of: {:#?}", file);
                    utils::misc::go_thru_dir(
                        std::path::PathBuf::from(file),
                        &mut zero_tolerance_hash,
                        "b",
                        &mut bitwise_paths,
                        &excluded_paths,
                    );
                }
                for path in &bitwise_paths {
                    utils::s_hashing::add_hash_from_path(path, &mut zero_tolerance_hash);
                }
            }
            None => {
                println!("Error parsing path in P Hash implementation of CLI args");
            }
        };
    }
    utils::misc::flush_the_cache(false);
}
