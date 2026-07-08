use utoipa::Modify;

pub(super) struct BinaryImageResponse;

impl Modify for BinaryImageResponse {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        apply_openapi_modifiers(openapi);
    }
}

pub(super) fn apply_openapi_modifiers(openapi: &mut utoipa::openapi::OpenApi) {
    add_annotation_identity_schema_fields(openapi);
    add_ocr_geometry_schema_fields(openapi);
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

fn add_ocr_geometry_schema_fields(openapi: &mut utoipa::openapi::OpenApi) {
    let Some(components) = openapi.components.as_mut() else {
        return;
    };
    components
        .schemas
        .insert("OcrGeometryPoint".to_string(), geometry_point_schema());
    components
        .schemas
        .insert("OcrGeometryBounds".to_string(), geometry_bounds_schema());
    components
        .schemas
        .insert("OcrGeometry".to_string(), geometry_schema());

    let Some(utoipa::openapi::RefOr::T(utoipa::openapi::schema::Schema::Object(schema))) =
        components.schemas.get_mut("OverlayBox")
    else {
        return;
    };
    schema.properties.insert(
        "geometry".to_string(),
        utoipa::openapi::RefOr::Ref(utoipa::openapi::schema::Ref::from_schema_name(
            "OcrGeometry",
        )),
    );
}

fn geometry_point_schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    use utoipa::openapi::schema::{ObjectBuilder, Schema, Type};
    utoipa::openapi::RefOr::T(Schema::Object(
        ObjectBuilder::new()
            .schema_type(Type::Object)
            .property("x", number_schema())
            .property("y", number_schema())
            .required("x")
            .required("y")
            .build(),
    ))
}

fn geometry_bounds_schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    use utoipa::openapi::schema::{ObjectBuilder, Schema, Type};
    utoipa::openapi::RefOr::T(Schema::Object(
        ObjectBuilder::new()
            .schema_type(Type::Object)
            .property("left", number_schema())
            .property("top", number_schema())
            .property("width", number_schema())
            .property("height", number_schema())
            .required("left")
            .required("top")
            .required("width")
            .required("height")
            .build(),
    ))
}

fn geometry_schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    use utoipa::openapi::schema::{ArrayBuilder, ObjectBuilder, Ref, Schema, Type};
    utoipa::openapi::RefOr::T(Schema::Object(
        ObjectBuilder::new()
            .schema_type(Type::Object)
            .property("kind", string_schema())
            .property(
                "points",
                ArrayBuilder::new().items(Ref::from_schema_name("OcrGeometryPoint")),
            )
            .property("rotation_degrees", number_schema())
            .property("layer_id", string_schema())
            .property("coordinate_space", string_schema())
            .property(
                "bounds",
                utoipa::openapi::RefOr::Ref(Ref::from_schema_name("OcrGeometryBounds")),
            )
            .required("kind")
            .required("points")
            .required("layer_id")
            .required("coordinate_space")
            .required("bounds")
            .build(),
    ))
}

fn string_schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    use utoipa::openapi::schema::{ObjectBuilder, Schema, Type};
    utoipa::openapi::RefOr::T(Schema::Object(
        ObjectBuilder::new().schema_type(Type::String).build(),
    ))
}

fn number_schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
    use utoipa::openapi::schema::{ObjectBuilder, Schema, Type};
    utoipa::openapi::RefOr::T(Schema::Object(
        ObjectBuilder::new().schema_type(Type::Number).build(),
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
