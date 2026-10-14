import os, sys, subprocess
class FFmpegChecks:                
    ffmpeg_link:set[str] = set(["libavutil.a", "libswscale.a","libavcodec.a", "libavdevice.a","libavfilter.a"])
    match sys.platform:
        case 'win32':
            ffmpeg_link:set[str] = set(["libavutil.dll.a", "libswscale.dll.a","libavcodec.dll.a", "libavdevice.dll.a","libavfilter.dll.a"])
            check_here: str | None = os.environ.get("FFMPEG_DIR") 
            if check_here is None:
                check_here:set = {os.environ.get("PATH").split(';')}
                if len(check_here) == 0: print(f"THERE'S A PROBLEM WITH FINDING THE POSSIBLE PATHS TO LOOK FOR FFMPEG")
        case 'linux':
            check_here = "/usr/lib/"
        case 'darwin':
            check_here = "/opt/homebrew/lib"

    
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
    rust_link = set(["cargo"]); 
    check_here = os.path.expanduser("~\\.cargo\\")
    hardware_acceleration = set(["hwcontext_cuda.h"])
    
    match sys.platform:
        case 'win32': rust_link = set(["cargo.exe"]); 
        case 'linux':  check_here = "/usr/bin/"

    def download_rust():
        match sys.platform:
            case 'darwin':
                print("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
            case 'linux':
                return
            case 'win32':
                subprocess.run("winget install --Id Rustlang.Rustup --source winget", shell=True)
        return

    def check_for_cuda_compatibility() -> bool:
        try:
            if sys.platform == 'win32' and not None: 
                ffmpeg_dir = os.environ.get("FFMPEG_DIR") 
                if Misc.search_for_file_in_dir(file_name=ffmpeg_dir, look_for=RustCheck.hardware_acceleration, optional_delete=True) == True:
                    return True
                elif set:
                    print(f"I couldn't find hwcontext_cuda.h in {ffmpeg_dir}")
                    return False
            else: return True
        except: 
            return False       
        
class ClangCheck:
    def download_libclang():
        match sys.platform:
            case 'darwin'|'linux':
                return
            case 'win32':
                subprocess.run("winget install -Id Microsoft.VisualStudio.Community --source winget")
                return

class Misc:
    def delete_via_set(parse_from:set) -> bool|set:
        for f in parse_from:
            if os.path.exists(f):                    
                match subprocess.run(f"del {f}",shell = True).check_returncode:
                    case 0:
                        print(f"subprocess.run(\"del {os.path.join(_, f)}\",\"shell\" = True)")
                        return True
                    case _:
                        print(f"Couldn't delete {f} try manually deleteing at {f}")
                        return False
        return
    
    def check_headers(dependency):
        set_to_check = set()
        match dependency:
            case 'FFmpegChecks':
                set_to_check = FFmpegChecks.ffmpeg_link
                attempt_for_PATH = FFmpegChecks.check_here
            case 'RustCheck':
                set_to_check = RustCheck.rust_link
                attempt_for_PATH = RustCheck.check_here
                
        # Matching based on OS
        if isinstance(attempt_for_PATH, str): missing_headers = Misc.search_for_file_in_dir(attempt_for_PATH, set_to_check)
        else:
            for fd in attempt_for_PATH:
                missing_headers = Misc.search_for_file_in_dir(fd, set_to_check)
                if missing_headers == True: return

        if type(missing_headers) == set: 
            match dependency:
                case 'FFmpegChecks':
                    FFmpegChecks.download_missing_headers(missing_headers)
                case 'RustCheck':
                    RustCheck.download_rust()

        elif missing_headers == False: print(f"Error downloading the needed files: {missing_headers}")
        return

    def search_for_file_in_dir(file_name, look_for:set, optional_delete=False) -> bool | set:
        match os.path.isdir(file_name):
            case True:
                for _, list_of_dir, list_of_files in os.walk(file_name): 
                    set_of_files = set(list_of_files)
                    list_of_found_files = set_of_files & look_for
                    list_of_missing_files =  look_for - set_of_files

                    if (len(list_of_missing_files) == 0) and (list_of_found_files == look_for): # Early Exit
                        print(f"Files: {list_of_found_files} seem to present"); return True
                    else:     
                        for subDir in list_of_dir:
                            if Misc.search_for_file_in_dir(f"{_}\\{subDir}", look_for, optional_delete=optional_delete) == True: 
                                return True
                            
                    if len(list_of_found_files) > 0 and optional_delete: 
                        Misc.delete_via_set({os.join(f"{_}\\{i}") for i in list_of_found_files})
                    elif len(list_of_found_files) > 0 and not optional_delete:
                        return True
            case False: 
                match optional_delete:
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

Misc.check_headers(FFmpegChecks.__name__)
Misc.check_headers(RustCheck.__name__)
RustCheck.check_for_cuda_compatibility()