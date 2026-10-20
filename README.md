As someone who constantly messing around with files, sometimes I accidentally download the same file twice with very small differences. So I wanted to make a program in which I can tell my computer to scan drive paths and tell me which files are most likely duplicates. 

# Download and Installation
There are two ways of passing arguments to the program to utilize its functionalities. The full list of functionalities and how to use them can be accessed via the [--help](#how-to-pass-the-paths-to-search) functionalities

1. One can download the code either through Git `git clone https://github.com/KGCS7/TrackerChecker.git {optional directory argument to create new/save to folder}` or 
[via the download link to the zip](https://github.com/KGCS7/TrackerChecker/archive/refs/heads/main.zip)

2. Build the program via `cargo build {directory where Cargo.toml resides if not currently in the same one}` <br>

3. Run the program. Note that running the program for the first time might cause some warnings and errors due to configuration that must be done, particularly on Windows Machines. <br><br>
Please refer to the [Dependencies](#dependencies) section for help configuring the requirements for the program and troubleshooting.<br><br>
- Cargo Method 
    <ul style="list-style-type: '- ';">
    <li> Run the program via 
    <code>cargo run {directory where Cargo.toml resides if not currently in the same one}</code></li>
    
    <li> To pass an argument(s) using the cargo method, use the format: 
    <code>cargo run -- {arg1} {arg2} ... {argN}</code> </li>
    </ul>
- Traditional Terminal Method
    <ul style="list-style-type: '- ';">
    <li> Alternatively one could also run the program in the traditional terminal method of<code>{Path of executable created by 'cargo build' process} </code> <br><br>
    Typically this is located at 
    <code>{Directory of Cargo.toml}/target/debug/tCheck.exe</code>
    </li>
    <br><br>
    </ul>

# How to pass the paths to search
If the program is ran without any paths to compare, then the program deletes any photos that was generated 

To get the full list of arguments, one can pass one of the stand alone argument; as they are all equivalent and all bring up the same help menu.
<ul>
<li> <code> -h </code> </li>
<li> <code> --h </code> </li>
<li> <code> -help </code> </li>
<li> <code> --help </code> </li>
</ul>

That is to say that `cargo run -- -help` will bring the same menu as `tCheck.exe -h`

### Perceptual Hashing
Denoted by <code>-p</code> or <code>-pc</code> followed by <code>First Path</code> <code>Second Path </code>... <code>Last Path</code>, the program will go through each file in all paths supplied and attempt to get a photo. For all photos that were able to be retrieved, a comparison of how visually similar they are is preformed. If deemed to be similar, the program will return with an alert.

### Bitwise Hashing
Denoted by <code>-b</code> or <code>-bw</code> followed by <code>First Path</code> <code>Second Path </code>... <code>Last Path</code>, the program will go through each file in all paths supplied create a hash based on the contents of the file. If a single bit is off, then the program will flag the files as different. <br><br>    

# Configure.py
A python file [./installScripts/Configure.py](./installScripts/Configure.py) is provided that could be ran before the first use of the program. [./installScripts/Configure.py](./installScripts/Configure.py) scans PATH environments and known default download locations for dependencies needed for the program to run properly. If the program detects any missing files, it will attempt to download them. 

# Dependencies
[A python script is supplied with the source code, ./installScripts/Configure.py](./installScripts/Configure.py) that when ran attempts to download missing dependencies.

- Linux and MacOS machines will both use [curl](https://curl.se/download.html) with [sh](https://www.gnu.org/software/bash/) for downloading and configuring [Rust](#rust)
- Linux will use the [apt manager](https://wiki.debian.org/AptCLI) to handle all other dependency downloads
- MacOS will use [homebrew](https://brew.sh/) to handle all other dependency downloads
- Windows machines downloads will be handled by [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/).

## Rust
This program is written entirely in Rust using FFI binding for C header files. As such [Rust](https://rust-lang.org/tools/install/) is needed to build the program. One might get a message like if we don't have Rust properly installed. ![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/noRustUpError.png)

Once we have our Rust dependencies downloaded; simply run command `cargo build {directory of Cargo.toml if not within directory}`.  

## FFMpeg 
Currently the program is written using FFMpeg 9.0.x FFI bindings. As such, the FFMpeg header (.h) and dynamic link libraries (.dll) must be present. 
### Mac
Thankfully Mac and Unix Like OS make it easier to install and link FFMpeg.
For Mac, using [homebrew package manager](https://brew.sh/) would be the simplest to download and use the FFMpeg function calls.

`brew install FFMpeg`

### Windows
FFMpeg Shared files are need for the program as we call functions themselves instead of the program. Shared files are not readily available for some architectures via package managers like Winget. Downloading and attempting to use FFMpeg bindings made for other architectures might get an error message like this:<br> 
![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/archMismatchMessage.png)<br>[However, FFMpeg provides the libraries themselves for different architecture.](https://github.com/BtbN/FFMpeg-Builds/releases) Just download the zipped file ending in `-shared-gpl.{extension}`. After downloading and extracting the files, we must link set the packages up to be able to be used for linking and building our program later. On Windows complete the following steps:

1. Hitting the Windows key and then typing 'environment variables' we select the option labeled  <code style="color:green"> Edit Environment Variables for your Account </code>
2. Next we have to set the environment variable `FFMPEG_DIR` to the location of the extracted FFMpeg files by clicking on <code style="color:green"> Add Variable</code> if the variable does not exist. 
3. If the variable is created but not properly set to the correct path, this would cause the program to not be able to build. Instead of adding a variable, we must <code style="color:green"> Edit Variable</code> to set the path of the folder containing the extracted FFMpeg files. If this step is not done/done incorrectly, one could expect an error message like this 
![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/FFMpegUnlink.png)
4. We will also add a new environment variable to point to the dynamic libraries (.dll) files to be used at execution time. Highlighting `Path` variable for User Variables (System Variables is also available if one Administrative Privileges) and clicking <code style="color:green">Edit</code>, we add a new value of the form '{path of FFMpeg folder}/bin'. If this step is not done/done incorrectly, one could expect an error message like this ![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/ffmpegUnlink.png)
5. Exit the terminal and restart.

Some systems might receive an error message like this ![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/hwcudaError.png)<br> FRET NOT! This is due to a linking error for GPU acceleration capabilities that are not implemented for the program (YET). The fix is to simply delete the offending file {path of FFMpeg Library}/include/libavutil/hwcontext_cuda.h. Then restart terminal and rebuild.

## Linux (Debian 13 tested)
One is able to download the files from FFMpeg needed by using <code> sudo apt install ffmpeg</code>

One might need to install the shared libraries used via `sudo apt install {package}` such as 
<ul>
<li> libavutil-dev </li>
<li> libswscale-dev </li>
<li> libavcodec-dev </li>
<li> libavdevice-dev </li>
<li> libavfilter-dev </li>
</ul>

# Linking and Building 
To build a Rust program will require clang or another gcc compiler to build the project. If not already configured you might receive the following error message depending on your operating system:

## Windows
![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/noVisualStudioLinker.png)<br>To properly build, [there's no other way other than downloading Microsoft Visual Studio](https://visualstudio.microsoft.com/downloads/) and enabling Desktop C++ Application development features.<br><br> <b>Note that if Microsoft Visual Studio was downloaded via the [Configure.py](./installScripts/Configure.py) provided, then you must launch the application and enable the C++ Application Development</b>
![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/desktopDevVS.png)

## Linux (Tested on Debian 13)
![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/debMissingCLinker.png)<br>
To alleviate this message, one simply runs <code>sudo apt install clang</code>

# LLVM
## Windows
Whether you download LLVM via [Winget](https://winstall.app/apps/LLVM.LLVM), [Chocolately](https://community.chocolatey.org/packages/llvm), or Visual Studio, to link and build the project from source, your computer must be able to access libclang.dll; otherwise you might get a message like:
![Windows libclang.dll error example](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/libclangError.png) <br>
Much like how we set the set a system and/or user PATH environment to access the FFMpeg dynamic linked libraries we must allow our program to access libclang.dll. The easiest way is to set this as a user/system wide environment variable.  

1. Hitting the Windows key and then typing 'environment variables' we select the option labeled <code style="color:green"> Edit Environment Variables for your Account </code>
2. Locate the location of `{path of LLVM}/LLVM/bin` and add to either User or System `PATH` environment variable (depending on privileges)
3. Clear previous build attempt with `cargo clean`, restart shell, and reattempt `cargo build`