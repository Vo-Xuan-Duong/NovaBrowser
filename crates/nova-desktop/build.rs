fn main() {
    slint_build::compile("ui/nova.slint")
        .expect("failed to compile the NovaBrowser desktop UI");
}
