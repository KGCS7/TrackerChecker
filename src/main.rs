use typenum::U32;
use std::{collections::BTreeMap, io::prelude::*, vec};
use sha2::{Digest, Sha256, digest::{generic_array::GenericArray}};
use std::{env,fs::File, path::{Path, PathBuf}};




pub fn find_file(path_name:&Path) -> Result<&str, &str>{
    let j = Path::is_file(&path_name);
    match j {
        true =>{
            let t = Path::to_str(&path_name);
            match t{
                Some(s)=>{return Ok(s);}
                None =>{return Err("Could not convert the path to a string");}
            }
        }   
        false =>{return Err("File does not exist!");}
    }
}


pub fn compare_hash_library(hash_to_check:&GenericArray<u8, U32>, file_name:String, check_here:&mut BTreeMap::<GenericArray<u8, U32>, Vec<String>>){
    let m = check_here.get(hash_to_check);
    match m {
        Some(dup)=>{println!("Found Duplicate: {:#?}", dup);}
        None=>{
            println!("New entry");
            // let key = *hash_to_check;
            check_here.insert(hash_to_check.clone(), vec![file_name]);

        }
    }
}



pub fn compute_hash_from_path(path_name:&str, add_here:&mut BTreeMap<GenericArray<u8, U32>,Vec<String>>){
    println!("Now computing the hash for file: {}", path_name);
    let fd = File::open(path_name);
    match fd {
        Ok(mut open_file) =>{
            let mut byte_buff = Vec::new(); 
            let read_file_to_bytes = open_file.read_to_end(&mut byte_buff);
            match read_file_to_bytes{
                Ok(_)=>{
                    let d:GenericArray<u8, U32> = Sha256::digest(byte_buff);
                    println!("Sha256 Hash from {}, is {:#?}", path_name, d);
                    compare_hash_library(&d, path_name.to_string(), add_here);
                }
                Err(e ) =>{println!("Error {}", e);}
            }
        }
        Err(e) => {
            println!("Error in Computing Hash From Path{}", e);
        }
    }
}

fn main() {
    let a:Vec<String> = env::args().collect();
    let a_len = a.len();
    
    // If arguments were passed when we start the program, then we automatically assume that they are paths to check
    if a_len.gt(&1){
        let mut file_hash:BTreeMap<GenericArray<u8, U32>, Vec<String>> = BTreeMap::new();
        for path in 1 .. a_len{
            let string_path = &a[path];
            let p_n = Path::new(string_path);
            let f = Path::is_file(p_n);
            
            match f {
                true => {compute_hash_from_path(string_path, &mut file_hash);}

                false => {
                    let d = Path::is_dir(p_n);
                    match d {
                        true =>{
                            let  p = std::fs::read_dir(p_n);
                            match p{
                                Ok(walk_dir) => {
                                    for file in walk_dir{
                                        match file {
                                            Ok(f)=>{
                                                let f_full:PathBuf = f.path();
                                                // let f_n = f.file_name();
                                                let f_n_s = f_full.to_str();
                                                match f_n_s {
                                                    Some(true_string_name)=>{
                                                        println!("File {} has hash of", true_string_name);
                                                        compute_hash_from_path(true_string_name, &mut file_hash);}
                                                    None=>{
                                                        println!("There was a problem computing the hashes the files");
                                                    }
                                                }
                                            }
                                            Err(e)=>{println!("Error: {}", e)}
                                        }
                                    }
                                }
                                Err(e)=>{println!("Error: {}", e)}
                            }

                        }
                        false =>{println!("Nether file nor directory! Skipping {}", string_path);}
                    }
                }
            }
        }
    }
    else if a_len.eq(&1) {
        println!("Need A Path To Get Started");
    }

}
