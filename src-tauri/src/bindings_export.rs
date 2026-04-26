#[cfg(test)]
mod tests {
    #[test]
    fn export_typescript_bindings() {
        tauri_specta::ts::export(
            tauri_specta::collect_commands![
                crate::compute_delta_e,
                crate::compute_xyy,
                crate::ipc::commands::get_app_state,
                crate::ipc::commands::connect_meter,
                crate::ipc::commands::disconnect_meter,
                crate::ipc::commands::connect_display,
                crate::ipc::commands::disconnect_display,
                crate::ipc::commands::get_device_inventory,
            ],
            "../../src/bindings.ts",
        )
        .unwrap();
    }
}
