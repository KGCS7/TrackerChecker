import os, sys, subprocess
ffmpeg_link = set(["libavutil.a", "libswscale.a","libavcodec.a", "libavdevice.a","libavfilter.a"])

def search_for_file_in_dir(file_name, optional_compare=False) -> bool | list:
    list_of_found_files = []
    list_of_missing_files = []
    match os.path.isdir(file_name):
        case True:
            for _, list_of_dir, list_of_files in os.walk(file_name): 
                set_of_files = set(list_of_files)
                list_of_found_files = set_of_files & ffmpeg_link
                list_of_missing_files =  ffmpeg_link - set_of_files

                if (len(list_of_missing_files) == 0) and (list_of_found_files == ffmpeg_link): # Early Exit
                    print("All files seem to present"); return True
                else:     
                    for subDir in list_of_dir:
                        search_for_file_in_dir(subDir)
        case False:
            match optional_compare:
                case True:
                    if file_name in ffmpeg_link:
                        return True
                    else:
                        return False
                case False:
                    return         
    
    if (len(list_of_missing_files) == 0) and (list_of_found_files == ffmpeg_link):
        print("All files seem to present"); return True
    else:     
        return list_of_missing_files


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

def check_ffmpeg_headers():
    match sys.platform:
        case 'linux':
            attempt_for_PATH = "/usr/lib/"
        case 'win32':
            attempt_for_PATH: str|list[str]|None = os.environ.get("PATH").split(':')
        case 'darwin':
            attempt_for_PATH = '/opt/homebrew/'


   
    # Matching based on OS
    if isinstance(attempt_for_PATH, str): missing_headers = search_for_file_in_dir(attempt_for_PATH)
    else:
        for fd in attempt_for_PATH:
            missing_headers = search_for_file_in_dir(fd)

    if type(missing_headers) == set: download_missing_headers(missing_headers)
    else: print(f"Error downloading the needed files: {missing_headers}")
    return


# Start Point of code 
check_ffmpeg_headers()