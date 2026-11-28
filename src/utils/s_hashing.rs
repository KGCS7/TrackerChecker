
use sha2::{Digest, digest::array::Array};
pub fn compare_hash_library<'a>(
    hash_to_check: Array<u8, typenum::U32>,
    file_name: &'a str,
    check_here: &mut std::collections::BTreeMap<Array<u8, typenum::U32>, &'a str>,
) {
    if let Some(dup) = check_here.get(&hash_to_check) {
        std::println!(
            "Found Duplicate: {:#?} is a duplicate of {:#?}",
            file_name,
            dup
        );
    } else {
        std::println!("New entry {:#?} for {}", hash_to_check, file_name);
        check_here.insert(hash_to_check, file_name);
    }
}

pub fn add_hash_from_path<'a>(
    path_name: &'a std::path::Path,
    add_here: &mut std::collections::BTreeMap<Array<u8, typenum::U32>, &'a str>,
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
                                path_name.to_str().expect("Couldn't determine the path"),
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
