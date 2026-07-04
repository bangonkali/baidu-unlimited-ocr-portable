impl AppState {
    async fn publish_status_changed(&self) {
        let payload = serde_json::to_value(self.status().await).unwrap_or_else(|_| json!({}));
        self.inner.hub.publish("status.changed", payload);
    }

    fn log_info(&self, component: &str, message: impl AsRef<str>) {
        let record = self.inner.logger.info(component, message);
        self.inner.hub.publish(
            "log.appended",
            serde_json::to_value(record).unwrap_or_else(|_| json!({})),
        );
    }

    fn log_warn(&self, component: &str, message: impl AsRef<str>) {
        let record = self.inner.logger.warn(component, message);
        self.inner.hub.publish(
            "log.appended",
            serde_json::to_value(record).unwrap_or_else(|_| json!({})),
        );
    }

    fn log_error(&self, component: &str, message: impl AsRef<str>) {
        let record = self.inner.logger.error(component, message);
        self.inner.hub.publish(
            "log.appended",
            serde_json::to_value(record).unwrap_or_else(|_| json!({})),
        );
    }
}
