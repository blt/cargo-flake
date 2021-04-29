# cargo-flake - a tool to detect flakey tests

This cargo plugin is intended to help you automatically discover flakey tests. A
flakey test is one that works _mostly_ but fails on occasion. `cargo-flake`
detects flakey tests by running your project's individual tests multiple times,
reporting on the failure frequency at the end of the run.
