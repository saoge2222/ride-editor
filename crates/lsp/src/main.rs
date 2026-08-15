fn main() {
    let mut server = lsp::server::LspServer::new();
    let code = server.run();
    std::process::exit(code);
}
