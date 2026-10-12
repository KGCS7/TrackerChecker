import os, sys, subprocess
class FFmpegChecks:
    ffmpeg_link:set[str] = set(["libavutil.a", "libswscale.a","libavcodec.a", "libavdevice.a","libavfilter.a"])
    def download_missing_headers(lib_List:set):
        match sys.platform:
            case 'linux':
                for lib in lib_List:
                    pkg_dev = lib.rsplit(".a")
                    subprocess.run(f"sudo apt install {pkg_dev[0]}-dev", shell=True)
            case 'darwin':
                if len(lib_List) > 0:
                    try:  
                        subprocess.run("brew install ffmpeg", shell=True)
                    except:
                        print(f"Error installing FFmpeg via Homebrew")
        return


class RustCheck:
    hardware_accelleration = set(["hwcontext_cuda.h"])
    if sys.platform == 'win32': rust_link = set(["cargo.exe"]); check_here = os.path.expanduser("~\\.cargo\\")
    else: rust_link = set(["cargo"]); check_here = os.path.expanduser("~/.cargo/")
    def download_rust():
        match sys.platform:
            case 'darwin':
                print("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
            case 'linux':
                return
            case 'win32':
                subprocess.run("winget install --Id Rustlang.Rustup --source winget", shell=True)
                # print("winget install --Id Rustlang.Rustup --source winget")
        return

    def check_for_cuda_compatibiility() -> bool:
        try:
            ffmpeg_dir = os.environ.get("FFMPEG_DIR") 
            searchFile = search_for_file_in_dir(file_name=ffmpeg_dir, look_for=RustCheck.hardware_accelleration, optional_compare=True)
            match type(searchFile):
                case set:
                    for i in searchFile:
                        match subprocess.run(f"del {i}",shell = True).check_returncode:
                            case 0:
                                # print(f"subprocess.run(\"del {i}\",\"shell\" = True)")
                                return True
                            case _:
                                print(f"Couldn't delete {RustCheck.hardware_accelleration} try manually deleteing at {i}")
                                return False
        except:
            # print(f"{os.environ}")
            return False       
        return

class ClangCheck:
    def download_libclang():
        match sys.platform:
            case 'darwin'|'linux':
                return
            case 'win32':
                subprocess.run("winget install -Id Microsoft.VisualStudio.Community --source winget")
                return

def check_headers(dependency):
    set_to_check = set()
    match dependency:
        case 'FFmpegChecks':
            set_to_check = FFmpegChecks.ffmpeg_link
            match sys.platform:
                    case 'linux':
                        attempt_for_PATH = "/usr/lib/"
                    case 'darwin':
                        attempt_for_PATH = "/opt/homebrew/lib"
                    case 'win32':
                        attempt_for_PATH: str|list[str]|None = os.environ.get("PATH").split(':')
            
        case 'RustCheck':
            set_to_check = RustCheck.rust_link
            attempt_for_PATH = RustCheck.check_here
            
    # Matching based on OS
    if isinstance(attempt_for_PATH, str): missing_headers = search_for_file_in_dir(attempt_for_PATH, set_to_check)
    else:
        for fd in attempt_for_PATH:
            missing_headers = search_for_file_in_dir(fd, set_to_check)

    if type(missing_headers) == set: 
        match dependency:
            case 'FFmpegChecks':
                FFmpegChecks.download_missing_headers(missing_headers)
            case 'RustCheck':
                RustCheck.download_rust()
    elif missing_headers == False: print(f"Error downloading the needed files: {missing_headers}")
    return

def search_for_file_in_dir(file_name, look_for:set, optional_compare=False) -> bool | list | set:
    list_of_found_files = []
    list_of_missing_files = []
    match os.path.isdir(file_name):
        case True:
            for _, list_of_dir, list_of_files in os.walk(file_name): 
                set_of_files = set(list_of_files)
                list_of_found_files = set_of_files & look_for
                list_of_missing_files =  look_for - set_of_files

                if (len(list_of_missing_files) == 0) and (list_of_found_files == look_for): # Early Exit
                    print(f"Files: {list_of_found_files} seem to present"); 
                    if optional_compare:
                        return {os.path.join(_, f) for f in list_of_found_files}
                    else:
                        return True
                else:     
                    for subDir in list_of_dir:
                        search_for_file_in_dir(subDir, look_for, optional_compare=optional_compare)
        case False:
            match optional_compare:
                case True:
                    if file_name in look_for:
                        return os.path.abspath(file_name)
                    else:
                        return False
                case False:
                    return         
    
    if (len(list_of_missing_files) == 0) and (list_of_found_files == look_for):
        print("All files seem to present"); return True
    else:     
        return list_of_missing_files

check_headers(FFmpegChecks.__name__)
check_headers(RustCheck.__name__)
RustCheck.check_for_cuda_compatibiility()