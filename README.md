# mods

## Modules
### Core modules

#### InstanceManager
Runs separate instances of the system. An instance have its own config and can run one copy of all modules
An instance can be active or inactive. The main UI is mainly a view on a selected instance.
This module also provides system information such as CPU and RAM usage.

#### MessageBus
All modules can communicate through this bus. Message types are defined by the MessageAPI module

Logging is also performed on this bus

#### HTTP server
Provides an HTTP endpoint based on the message/api module for the integrated UI and other external integrations

#### WebSocket server
Provides an WS endpoint based on the message/api module for the integrated UI and other external integrations.

Basically a way for external systems to send and receive on the internal MessageBus

#### Config
Defines the base config of the system and is extended by types from other modules

#### MessageAPI
Types for internal and external communication. Extended by API definition from the feature modules.

#### DeviceAggregator
Mixes devices such as PositionProviders to PositionSubscribers

#### LocationAggregator
Manages and defines collections of locations. A location can be defined as a algebraical combination from multiple PositionProviders. For example a device is inside a location if any one (PP1 OR PP2) of two PositionProviders says so.

#### ProcessController
Manages and defines process. A process requires some devices and how they report an OK result. Devices can be locked when outside of a location and unlocked when inside an location. Locations can trigger a program on a device for example. Logic for conditions 
that must be fullfilled before the instance shall proceed with the next steps. This module might be merged with the EventRouter

#### EventRouter
Customize behaviour based on events from the modules.

### Feature modules - PositionProviders or Inputs

PositionProviders will provide positions to any PositionSubscriber. 

#### MonoTagTracker (PositionProvider)
Single camera tag tracker module - provides position data about targets (collection of tags)

The MTT can track tags from some aruco and apriltag familes.

#### UWB (PositionProvider)

#### MQTT (PositionProvider)

#### SpiderAPI (PositionProvider)

#### ModbusTCP (PositionProvider)

#### TOF FingerPrinter (PositionProvider)

#### GenericResultProvider
Provides any result status that can be used in a process. Image validation, digital status signals etc.

### Feature modules - PositionSubscribers or Outputs
#### OpenProtcol
#### GenericToolAPI
#### ProjectorsAPI


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


## Build and run with camera sensor support (requires opencv and apriltag libs):
```
cargo run --release --features sensor
```

```
rustup component add llvm-tools
cargo install cargo-llvm-cov
cargo llvm-cov
cargo llvm-cov --html
```

```
npm install -g quicktype
```


docker run -d -p 5000:5000 --name registry registry:2.7

docker tag ubuntu localhost:6000/mods-runner

docker push localhost:6000/mods-runner

```
docker run -d --restart=always -p 8123:8123/tcp -p 8124:8124/tcp midnightair.local:6000/mods-runner
```

sudo vi /etc/docker/daemon.json
{
  "insecure-registries": ["midnightair.local:6000"]
}

sudo systemctl restart docker

## Linux
```
sudo apt install libclang-dev
sudo apt install libopencv-dev
sudo apt install libssl-dev
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

```
$env:APRILTAG_SYS_WINDOWS_PTHREAD_INCLUDE_DIR = "$env:VCPKG_ROOT\installed\x64-windows-static\include"
$env:APRILTAG_SYS_WINDOWS_PTHREAD_STATIC_LIB = "$env:VCPKG_ROOT\installed\x64-windows-static\lib\pthreadVC3.lib"
```

## mac

```
brew install opencv llvm
```
### env

```
export DYLD_FALLBACK_LIBRARY_PATH="$(xcode-select --print-path)/Toolchains/XcodeDefault.xctoolchain/usr/lib/"
export LDFLAGS=-L/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib
export LD_LIBRARY_PATH=${LD_LIBRARY_PATH}:/usr/local/lib  
```



### Develop and test

As a last step after cargo check passes run cargo fmt and then cargo check again. Errors AND warnings is not allowed and shall be fixed.

To run all tests use the following command:

```
cargo test --release
```

```
sudo ufw allow 8123
lsusb
sudo apt install v4l-utils
sudo v4l2-ctl --list-devices
sudo v4l2-ctl -d /dev/video0 --list-formats-ext
journalctl -fu mods
```

