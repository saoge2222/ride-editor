slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    MainEditorWindow::new().unwrap().run().unwrap();
    Ok(())
}
