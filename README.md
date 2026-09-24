As someone who constantly messing around with files, sometimes I accidentally download the same file twice with very small differences. So I wanted to make a program in which I can tell my computer to scan drive paths and tell me which files are most likely duplicates. 

# Download and Installation
There are two ways of passing arguments to the program to utilize its functionalities. The full `--help`

1. One can download the code either through Git `git clone https://github.com/KGCS7/TrackerChecker.git {optional directory argument to create new/save to folder}` or 
[via the download link to the zip](https://github.com/KGCS7/TrackerChecker/archive/refs/heads/main.zip)

2. Build the program via `cargo build {directory where Cargo.toml resides if not currently in the same one}` <br>

3. Run the program. Note that running the program for the first time might cause some warnings and errors due to configuration that must be done, particularly on Windows Machines. <br><br>
Plase refer to the [Dependencies](#dependencies) section for help configuring the requirements for the program and troubleshooting. Automatic configuration will be offered in the future. <br>

- Cargo Method 
    <ul style="list-style-type: '- ';">
    <li> Run the program via 
    <code>cargo run {directory where Cargo.toml resides if not currently in the same one}</code></li>
    
    <li> To pass an arguement(s) using the cargo method, use the format: 
    <code>cargo run -- {arg1} {arg2} ... {argN}</code> </li>

    </ul>
- Tradtional Terminal Method

    <ul style="list-style-type: '- ';">
    <li> Alternatively one could also run the program in the traditional terminal method of<code>{Path of executable created by step 2} </code> <br><br>
    Typically this is located at 
    <code>{Directory of Cargo.toml}/target/debug/tCheck.exe</code>
    </li>
    <br><br>
    </ul>
    
# Dependencies

## Rust
This program is written entirely in Rust using FFI binding for C header files. As such [Rust](https://rust-lang.org/tools/install/) is needed to build the program. One might get a message like if we don't have Rust properly installed. ![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/noRustUpError.png)

Once we have our Rust dependencies downloaded; simply run command `cargo build {directory of Cargo.toml if not within directory}`.  

## FFMpeg 
Currently the program is written using FFMpeg 9.0.1 FFI bindings. As such, the FFMpeg header (.h) and dynamic link libraries (.dll) must be present. 
### Mac

Thankfully Mac and Unix Like OS make it easier to install and link FFMpeg.
For Mac, using [homebrew package manager](https://brew.sh/) would be the simplest to download and use the FFMpeg function calls.

`brew install FFMpeg`
### Windows
FFMpeg Shared files are need for the program as we call functions themselves instead of the program. Shared files are not readily available for some architectures via package managers like Winget. Downloading and attempting to use FFMpeg bindings made for other architectures might get an error message like this: 
![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/archMismatchMessage.png)  [However, FFMpeg provides the libraries themselves for different architecture.](https://github.com/BtbN/FFMpeg-Builds/releases) Just download the zipped file ending in `*-shared-gpl.{extension}`. After downloading and extracting the files, we must link set the packages up to be able to be used for linking and building our program later. On Windows complete the following steps:

1. Hitting the Windows key and then typing 'environment variables' we select the option labeled  <code style="color:green"> Edit Environment Variables for your Account </code>
2. Next we have to set the environment variable `FFMPEG_DIR` to the location of the extracted FFMpeg files by clicking on <code style="color:green"> Add Variable</code> if the variable does not exist. 
3. If the variable is created but not properly set to the correct path, this would cause the program to not be able to build. Instead of adding a variable, we must <code style="color:green"> Edit Variable</code> to set the path of the folder containing the extracted FFMpeg files. If this step is not done/done incorrectly, one could expect an error message like this 
![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/FFMpegUnlink.png)
4. We will also add a new environment variable to point to the dynamic libraries (.dll) files to be used at execution time. Highlighting `Path` variable for User Variables (System Variables is also available if one Administrative Privileges) and clicking <code style="color:green">Edit</code>, we add a new value of the form '{path of FFMpeg folder}/bin'. If this step is not done/done incorrectly, one could expect an error message like this ![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/FFMpegUnlink.png)
5. Exit the terminal and restart.

Some systems might receive an error message like this ![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/hwcudaError.png) FRET NOT! This is due to a linking error for GPU acceleration capabilities that are not implemented for the program (YET). The fix is to simply delete the offending file {path of FFMpeg Library}/include/libavutil/hwcontext_cuda.h. Then restart terminal and rebuild.

# Linking and Building 
## Windows
To build a Rust program will require clang or another gcc compiler to build the project. If not already configured you might receive an error message like ![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/noVisualStudioLinker.png)
To properly build, [there's no other way other than downloading Microsoft Visual Studio](https://visualstudio.microsoft.com/downloads/) and enabling Desktop C++ Application development features. ![](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/desktopDevVS.png)

# LLVM
## Windows
Whether you download LLVM via [Chocolately](https://community.chocolatey.org/packages/llvm), [Winget](https://winstall.app/apps/LLVM.LLVM), or Visual Studio, to link and build the project from source, your computer must be able to access libclang.dll; otherwise you might get a message like:
![photo](https://raw.githubusercontent.com/KGCS7/TrackerChecker/RSS/errorPics/libclangError.png)
and the project will not run. Much like how we set the set a system and/or user PATH environment to access the FFMpeg dynamic linked libraries we must allow our program to access libclang.dll. The easiest way is to set this as a user/system wide environment variable.  

1. Hitting the Windows key and then typing 'environment variables' we select the option labeled <code style="color:green"> Edit Environment Variables for your Account </code>
2. Locate the location of `{path of LLVM}/LLVM/bin` and add to either User or System `PATH` environment variable (depending on privileges)
3. Clear previous build attempt with `cargo clean`, restart shell, and reattempt `cargo build`
