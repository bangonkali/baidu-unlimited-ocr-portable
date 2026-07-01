#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_workbench_patch() -> Result<()> {
        let mut settings = WorkbenchUiSettings::default();
        apply_workbench_patch(
            &mut settings,
            crate::types::WorkbenchUiSettingsPatch {
                theme: Some("light".to_string()),
                overlay_visible: Some(false),
                ..Default::default()
            },
        )?;
        assert_eq!(settings.theme, "light");
        assert!(!settings.overlay_visible);
        Ok(())
    }
}
