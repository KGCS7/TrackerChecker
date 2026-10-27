import os, sys, subprocess, pathlib, ctypes
if sys.platform == 'win32':
    import winreg

class FFmpegChecks:                
    ffmpeg_link:set[str] = set(["libavutil.a", "libswscale.a","libavcodec.a", "libavdevice.a","libavfilter.a"])
    match sys.platform:
        case 'win32':
            ffmpeg_link:set[str] = set(["libavutil.dll.a", "libswscale.dll.a","libavcodec.dll.a", "libavdevice.dll.a","libavfilter.dll.a"])
            check_here: str | None = os.environ.get("FFMPEG_DIR") 
            if check_here is None:
                check_here:set = {i for i in os.environ.get("PATH").split(';')}
                if len(check_here) == 0: print(f"THERE'S A PROBLEM WITH FINDING THE POSSIBLE PATHS TO LOOK FOR FFMPEG")
        case 'linux':
            check_here = "/usr/lib/"
        case 'darwin':
            check_here = "/opt/homebrew/lib"

    """ 
        FOR USE FOR WINDOWS MACHINES 
        Looks for FFMpeg libraries and creates a system-wide environment variable with the parent root as the value.
        If there is a problem setting the environment variable, an error will be thrown. 
    """  
    def create_FFmpeg_Dir_Path():
        try_where_exe = subprocess.run(["where.exe", "/R", "C:\\", "libavutil.dll.a"], capture_output=True, text=True, check=True)
        matches = try_where_exe.stdout.splitlines()
        if not matches:
            raise FileNotFoundError("where.exe found no libavutil.dll.a files")

        ffmpeg_dir = str(pathlib.Path(matches[0]).parent.parent)
        environment_key_path = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
        try:
            with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, environment_key_path, 0, winreg.KEY_SET_VALUE) as environment_key:
                winreg.SetValueEx(environment_key, "FFMPEG_DIR", 0, winreg.REG_SZ, ffmpeg_dir)
        except PermissionError as error:
            raise PermissionError("Setting machine-wide FFMPEG_DIR requires running the setup as Administrator") from error

        result = ctypes.c_size_t()
        send_message_timeout = ctypes.WinDLL("user32", use_last_error=True).SendMessageTimeoutW
        send_message_timeout.argtypes = [ 
            ctypes.c_void_p, ctypes.c_uint, ctypes.c_size_t, ctypes.c_wchar_p, 
            ctypes.c_uint, ctypes.c_uint, ctypes.POINTER(ctypes.c_size_t)]
        send_message_timeout.restype = ctypes.c_ssize_t
        if not send_message_timeout(0xFFFF, 0x001A, 0, "Environment", 0x0002, 5000, ctypes.byref(result)):
            print("Warning: Windows did not notify running applications about the environment change.")

        print(f"Successfully created FFMPEG_DIR that points to {ffmpeg_dir}")
        print("Restart any already-open terminals or IDEs before building.")
        return

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
            case 'win32':
                try:
                    FFmpegChecks.create_FFmpeg_Dir_Path()
                    return  
                except Exception as e:
                    print(f"Error {e}")
                    raise

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
                subprocess.run("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",shell=True)
            case 'linux':
                subprocess.run("sudo apt install rustup", shell=True)
            case 'win32':
                subprocess.run("winget install --Id Rustlang.Rustup --source winget", shell=True)
        return

    def check_for_cuda_compatibility() -> bool:
        try:
            if sys.platform == 'win32' and not None: 
                ffmpeg_dir = os.environ.get("FFMPEG_DIR") 
                attempt_for_delete = Misc.search_for_file_in_dir(file_name=ffmpeg_dir, look_for=RustCheck.hardware_acceleration, optional_delete=True) 
                if attempt_for_delete == True: return True
                elif set: return False
            else: return True
        except Exception as e:
            print(f"Error: {e}") 
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
    def delete_via_set(delete_set:set, target_dir:None|set = None) -> bool|set:
        failed_delete = set()
        for f in delete_set:
            if os.path.exists(f):                    
                match subprocess.run(f"del {f}",shell = True).check_returncode:
                    case 0:
                        print(f"subprocess.run(\"del {os.path.join(_, f)}\",\"shell\" = True)")
                    case _:
                        failed_delete.add(f)
        if len(failed_delete) != 0:
            for i in failed_delete: 
                try:
                    attempt_path = pathlib.Path(i)
                    if attempt_path.is_file():
                        subprocess.run(f"where.exe /R {attempt_path.name}")               
                except Exception as e:
                    print(f"{e}")
            
            print("Couldn't find the following files")
            return failed_delete       
        else: return True
    
    def check_headers(dependency):
        set_to_check = set()
        match dependency:
            case 'FFmpegChecks':
                set_to_check = FFmpegChecks.ffmpeg_link
                attempt_for_PATH = FFmpegChecks.check_here
            case 'RustCheck':
                set_to_check = RustCheck.rust_link
                attempt_for_PATH = RustCheck.check_here

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

        elif missing_headers == False: print(f"Error downloading the needed files: {set_to_check}")
        return

    def search_for_file_in_dir(file_name, look_for:set, optional_delete=False) -> bool | set:
        g_found_files:set[str] = set()
        match os.path.isdir(file_name):
            case True:
                for _, list_of_dir, list_of_files in os.walk(file_name): 
                    set_of_files = set(list_of_files)
                    list_of_found_files = set_of_files & look_for
                    list_of_missing_files =  look_for - set_of_files

                    if (len(list_of_missing_files) == 0) and (list_of_found_files == look_for): # Early Exit
                        print(f"Files: {list_of_found_files} seem to present"); 
                        # While we are here we shall delete the files if the optional_delete is TRUE
                        if optional_delete:
                            Misc.delete_via_set({(f"{_}\\{i}") for i in list_of_found_files})
                        return True
                    else:     
                        match len(list_of_found_files) > 0:
                            case True:
                                if len(list_of_found_files) > 0 and optional_delete: 
                                    Misc.delete_via_set({os.join(f"{_}\\{i}") for i in list_of_found_files})
                                elif (len(list_of_found_files) > 0) and (not optional_delete) and (len(list_of_missing_files) > 0):
                                    for i in list_of_found_files: g_found_files.add(i)
                                    look_for = list_of_missing_files
                            case _:
                                continue
            case False: 
                if file_name in look_for:
                    match optional_delete:
                        case True:
                            Misc.delete_via_set(set(file_name))
                            os.path.abspath(file_name)
                        case False:
                            return True         
                else:
                    return False
        return list_of_missing_files


if __name__ == "__main__":
    Misc.check_headers(FFmpegChecks.__name__)
    Misc.check_headers(RustCheck.__name__)
    RustCheck.check_for_cuda_compatibility()