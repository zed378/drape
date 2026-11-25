#![forbid(unsafe_code)]

fn main() {
    slint_build::compile("ui/probe.slint").expect("the handle probe component must compile");
}
