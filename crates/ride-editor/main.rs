mod filemng;

use filemng::FileMngClient;

fn main() {
    let file_mgr = FileMngClient::spawn();
    println!(
        "ride-editor: Vulkan GUI (crates/gui-workbench) not wired up yet; current dir: {}",
        file_mgr.current_dir()
    );
}
