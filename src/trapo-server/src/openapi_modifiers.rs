use utoipa::Modify;

pub(super) struct BinaryImageResponse;

impl Modify for BinaryImageResponse {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        add_annotation_identity_schema_fields(openapi);
        for (path_key, content_types) in [
            (
                "/api/documents/{file_hash}/preview-images/{variant}/{page_no}",
                &["image/png", "image/jpeg"][..],
            ),
            (
                "/api/documents/{file_hash}/regions/{region_id}/snippet",
                &["image/png"][..],
            ),
        ] {
            let Some(path) = openapi.paths.paths.get_mut(path_key) else {
                continue;
            };
            let Some(operation) = path.get.as_mut() else {
                continue;
            };
            let Some(utoipa::openapi::RefOr::T(response)) =
                operation.responses.responses.get_mut("200")
            else {
                continue;
            };
            for content_type in content_types {
                if let Some(content) = response.content.get_mut(*content_type) {
                    content.schema = Some(binary_schema());
                }
            }
        }
    }
}

fn add_annotation_identity_schema_fields(openapi: &mut utoipa::openapi::OpenApi) {
    let Some(components) = openapi.components.as_mut() else {
        return;
    };
    for schema_name in ["OverlayBox", "TextRegionSpan"] {
        let Some(utoipa::openapi::RefOr::T(utoipa::openapi::schema::Schema::Object(schema))) =
            components.schemas.get_mut(schema_name)
        else {
            continue;
        };
        schema
            .properties
            .insert("annotation_id".to_string(), string_schema());
        if !schema.required.iter().any(|item| item == "annotation_id") {
            schema.required.insert(0, "annotation_id".to_string());
        }
    }
}

fn string_schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    use utoipa::openapi::schema::{ObjectBuilder, Schema, Type};
    utoipa::openapi::RefOr::T(Schema::Object(
        ObjectBuilder::new().schema_type(Type::String).build(),
    ))
}

fn binary_schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    use utoipa::openapi::schema::{KnownFormat, ObjectBuilder, Schema, SchemaFormat, Type};
    utoipa::openapi::RefOr::T(Schema::Object(
        ObjectBuilder::new()
            .schema_type(Type::String)
            .format(Some(SchemaFormat::KnownFormat(KnownFormat::Binary)))
            .build(),
    ))
}
