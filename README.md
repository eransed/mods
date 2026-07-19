# mods

## Required build dependencies
- rust
- git
- nodejs
- npm
- docker
- quicktype
- opencv
- llvm
- vcpkg?
- cmake?

```
npm install -g quicktype
```

## windows dependencies
https://github.com/llvm/llvm-project

https://github.com/opencv/opencv

### env
#### Examples
- OpenCV_DIR=C:\Users\root\repos\opencv\build
- OPENCV_INCLUDE_PATHS=C:\Users\root\repos\opencv\build\install\include
- OPENCV_LINK_PATHS=C:\Users\root\repos\opencv\build\install\x64\vc18\lib

Additionally:
- PATH shall include something like /c/Users/root/repos/opencv/build/install/x64/vc18/bin:/c/Program Files/LLVM/bin
- opencv_version should work and print something
- clang++ --version should work and print something
- cmake --version should work and print something

```
export OPENCV_LINK_LIBS="opencv_calib3d4150,opencv_core4150,opencv_dnn4150,opencv_features2d4150,opencv_flann4150,opencv_gapi4150,opencv_highgui4150,opencv_imgcodecs4150,opencv_imgproc4150,opencv_ml4150,opencv_objdetect4150,opencv_photo4150,opencv_stitching4150,opencv_video4150,opencv_videoio4150"
```

Or

```
export OPENCV_LINK_LIBS="opencv_calib3d,opencv_core,opencv_dnn,opencv_features2d,opencv_flann,opencv_gapi,opencv_highgui,opencv_imgcodecs,opencv_imgproc,opencv_ml,opencv_objdetect,opencv_photo,opencv_stitching,opencv_video,opencv_videoio"
```

```
vcpkg install pthread:x64-windows-static
```

```
export APRILTAG_SYS_WINDOWS_PTHREAD_INCLUDE_DIR="$VCPKG_ROOT/installed/x64-windows-static/include"
export APRILTAG_SYS_WINDOWS_PTHREAD_STATIC_LIB="$VCPKG_ROOT/installed/x64-windows-static/lib/pthreadVC3.lib"
```
