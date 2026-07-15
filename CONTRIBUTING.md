# Contributing

Keep changes scoped and preserve data safety. Never write codec output directly over a user file, ignore codec exit status, or decide destination names inside a parallel worker.

Before submitting a change, run the frontend and Rust commands listed in the README. Add one focused test for any new branch, parser, collision policy, or filesystem safety behavior. Do not commit generated `dist`, `.output`, `.nuxt`, or `src-tauri/target` files.

Use Conventional Commit messages. Version and release changes must keep `package.json`, `Cargo.toml`, `tauri.conf.json`, and the `vX.Y.Z` tag synchronized.
