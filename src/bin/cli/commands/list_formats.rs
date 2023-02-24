use flpak::Registry;

pub fn list_formats() {
    let registry = Registry::new();

    for format_desc in registry.list() {
        let mut capabilities = Vec::new();

        if format_desc.make_reader_fn.is_some() {
            capabilities.push("extract");
        }

        if format_desc.writer_fn.is_some() {
            capabilities.push("create");
        }

        let capabilities = capabilities.join(", ");

        println!(
            "{:<8}{:<48}{}",
            format_desc.name, format_desc.description, capabilities
        );
    }
}
