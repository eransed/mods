
When coding make sure to write short comments about the intent of the code. Use doc comments for public functions and structs.

It is forbidden to use unwrap() and expect() in the code. Use proper error handling instead.

As a last step after cargo check passes run ```cargo fmt```.
Errors and related warnings are not allowed and shall be fixed.
To run all tests use the following command:
```
cargo test
```
