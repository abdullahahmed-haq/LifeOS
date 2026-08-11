#[taurpc::ipc_type]
#[serde(rename_all = "camelCase")]
struct Area {
    id: String,
    title: String,
    revision: i32,
}

#[taurpc::ipc_type]
#[serde(rename_all = "camelCase")]
struct UpdateAreaRequest {
    id: String,
    title: String,
    expected_revision: i32,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, specta::Type)]
#[serde(
    tag = "code",
    content = "details",
    rename_all = "SCREAMING_SNAKE_CASE",
    rename_all_fields = "camelCase"
)]
enum ProbeError {
    NotFound { entity_id: String },
    ConflictRevision { expected: i32, actual: i32 },
}

#[taurpc::ipc_type]
#[serde(rename_all = "camelCase")]
struct FoundationProgress {
    phase: String,
    percent: u8,
}

#[taurpc::procedures(event_trigger = ProbeEventTrigger)]
trait ProbeApi {
    async fn get_area(id: String) -> Result<Area, ProbeError>;
    async fn update_area(request: UpdateAreaRequest) -> Result<Area, ProbeError>;

    #[taurpc(event)]
    async fn foundation_progress(progress: FoundationProgress);
}

#[derive(Clone)]
struct ProbeApiImpl;

#[taurpc::resolvers]
impl ProbeApi for ProbeApiImpl {
    async fn get_area(self, id: String) -> Result<Area, ProbeError> {
        if id.is_empty() {
            return Err(ProbeError::NotFound { entity_id: id });
        }
        Ok(Area {
            id,
            title: "Area".into(),
            revision: 7,
        })
    }

    async fn update_area(self, request: UpdateAreaRequest) -> Result<Area, ProbeError> {
        if request.expected_revision != 7 {
            return Err(ProbeError::ConflictRevision {
                expected: request.expected_revision,
                actual: 7,
            });
        }
        Ok(Area {
            id: request.id,
            title: request.title,
            revision: 8,
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handler = ProbeApiImpl.into_handler();
    let destination = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generated.ts");
    taurpc::Exporter::new()
        .error_handling(taurpc::ErrorHandlingMode::Result)
        .export(&handler, &destination)?;
    let generated = std::fs::read_to_string(&destination)?;
    std::fs::write(destination, format!("{}\n", generated.trim_end()))?;
    Ok(())
}
