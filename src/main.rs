//! Minimal binary entrypoint; delegates lifecycle and platform setup to `rustcraft::platform`.

fn main() {
    rustcraft::platform::run_app();
}