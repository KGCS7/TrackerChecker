#[cfg(target_os = "macos")]
mod utils;

fn main() {
    let m =clap::Command::new("TorrentChecker").version("1.0")
    .about("Finds duplicates of media by using a SHA256 hash or perceptual hashing algorithms.")
    .arg(
        clap::arg!(
            -p --perceptual_hash ["Path1 Path2 ... PathN"] "perceptual_hash"
        ).id("pc")
    ).arg(
        clap::arg!(
            -b --bitwise_hash ["Path1 Path2 ... PathN"] "bitwise_hash"
        ).id("bc")
    )
    .get_matches();
    
    let mut file_hash:std::collections::BTreeMap<sha2::digest::array::Array<u8, typenum::U32>, String> = std::collections::BTreeMap::new();

    if m.contains_id("pc"){
        let temp_variable: Option<clap::parser::RawValues<'_>> = m.get_raw("pc");
        match temp_variable {
            Some(i)=>{
                for file in i{
                    println!("Found path of: {:#?}", file);
                    utils::misc::go_thru_dir(&mut std::path::PathBuf::from(file), &mut file_hash, "p");
                }
            }
            None=>{println!("Error parsing path in P Hash implementation of CLI args");}
        };
    }else if m.contains_id("bc") {
        let temp_variable: Option<clap::parser::RawValues<'_>> = m.get_raw("bc");
         match temp_variable {
            Some(i)=>{
                for file in i{
                    println!("Found path of: {:#?}", file);
                    utils::misc::go_thru_dir(&mut std::path::PathBuf::from(file), &mut file_hash, "b");
                }
            }
            None=>{println!("Error parsing path in P Hash implementation of CLI args");}
        };
    }
}
