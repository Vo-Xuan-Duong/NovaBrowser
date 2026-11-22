fn main() {
    slint_build::compile("ui/nova.slint")
        .expect("Failed to compile NovaBrowser's Slint UI");
}
