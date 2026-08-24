
When coding make sure to write short comments about the intent of the code. Use doc comments for public functions and structs.

When running any cargo commands on Windows make sure to set the following environment variables to make sure the code can compile with all features enabled:

PowerShell:
```
$env:APRILTAG_SYS_WINDOWS_PTHREAD_INCLUDE_DIR = "$env:VCPKG_ROOT\installed\x64-windows-static\include"
$env:APRILTAG_SYS_WINDOWS_PTHREAD_STATIC_LIB = "$env:VCPKG_ROOT\installed\x64-windows-static\lib\pthreadVC3.lib"
```

bash:
```
export APRILTAG_SYS_WINDOWS_PTHREAD_INCLUDE_DIR="$VCPKG_ROOT/installed/x64-windows-static/include"
export APRILTAG_SYS_WINDOWS_PTHREAD_STATIC_LIB="$VCPKG_ROOT/installed/x64-windows-static/lib/pthreadVC3.lib"
```

Never use unsafe code! No exceptions. If you need to use unsafe code, please discuss it with the team first.

It is forbidden to use unwrap() and expect() in the code. Use proper error handling instead.

As a last step after cargo check passes run ```cargo fmt```.
Errors and related warnings are not allowed and shall be fixed.
To run all tests use the following command:
```
cargo test
```
