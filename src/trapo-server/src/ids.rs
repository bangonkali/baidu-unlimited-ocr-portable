pub(crate) fn new_persistence_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

pub(crate) fn is_uuid_v7(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|id| id.get_version_num() == 7)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistence_ids_are_uuid_v7() {
        let id = new_persistence_id();

        assert!(is_uuid_v7(&id));
        assert_eq!(id.len(), 36);
    }
}
