# CANPi Layout Library

The functionality is distributed as a Rust crate that is accessed from GitHub using a tag or a branch (or the latest on the default branch).
e.g.

```cargo
[dependencies]
canpi-layout = { git = "https://github.com/emthornber/canpi-layout.git", tag = "v0.1.0" }
```

or

```cargo
[dependencies]
canpi-layout = { git = "https://github.com/emthornber/canpi-layout.git", branch = "lgtrunk" }
```

or

```cargo
[dependencies]
canpi-layout = { git = "https://github.com/emthornber/canpi-layout.git" }
```

## Compiling

The source code will be compiled as part of the build of the calling executable along with all the other dependant crates.
