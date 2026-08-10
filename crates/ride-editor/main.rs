mod filemng;

use filemng::FileMngClient;

fn main() {
    let file_mgr = FileMngClient::spawn();
    println!(
        "ride-editor: Zed GPUI GUI (crates/gui/gui-workbench) not wired up yet; current dir: {}",
        file_mgr.current_dir()
    );
}
