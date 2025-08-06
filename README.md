### FrAPI - Free API client/server tester.

![General_01](./docs/img/general_01.png)

## Description

This application made purely with Rust lang. Aiming to fast API requests mostly.  
At same time analog of `Postman`, but more lightweight, without any telemetry and have less functions.  
Excelent if you need to test something fast here and now with limited resources.
Also this application planned as mocking API server for test purpose, supporting complex data models constructor and some basic operations with data.

### !! This application under heavy development !!

Pros:

-   lightweight
-   easy to deploy - only one file
-   no telemetry, accounts, clouts, etc...
-   faster
-   can be build under any OS, where Rust libs used available (OSX, Windows, Linux, etc...)

Cons:

-   less functions, compared with `Postman`
-   no shared data for teams
-   ui abit clumsy (yet)
-   limited protocols available

### Build

Iam using combination of batch scripts and `build.rs`, deppending on target OS.

### Windows:

1. Install `Rust 1.88.0`, if not done yet.
2. Navigate to project directory and execute from terminal:
   `cargo build --release`

This is esiest way to do it.

### OSx

1. Install `Rust 1.88.0` if not done yet.
2. Navigate to project directory and execute from terminal:
   `cargo build --release`
3. After build is done you will notice addition folder to appear `builds/macos`
4. copy bin file `frapi` from Rust default target folder to `builds/<frapi version>/macos/frapi.app/MacOS`

**NOTICE:** For this build i use custom simple batch script to automate this copy.  
All additional folders for MacOS application made inside `build.rs` before compilaiton of main application.

### Roadmap:

1. ~~Create working build with basic client request abilities~~ - Done
2. Add authorizations - `Basic` and `Bearer`
3. Add self hosting servers with models builders for mocking data for HTTP
4. Add self hosting servers with models builders for mocking data for WS
5. Add enviroment variables
6. Add `mqtt` protocol for requests
7. Create `Settings` page with some base settings
