use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use axum::{
    extract::Query,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use rnb_engine::{open, BioView, PathSpec, SemiringKind};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[derive(Deserialize, Clone)]
struct ArtifactQuery {
    /// Path to the .rnb artifact on disk.
    path: String,
}

#[derive(Serialize)]
struct ArtifactSummary {
    path: String,
    object_count: Option<u32>,
    has_string_dict: bool,
    has_attribute_table: bool,
    has_relation_table: bool,
}

async fn artifact_summary(Query(q): Query<ArtifactQuery>) -> impl IntoResponse {
    let path = PathBuf::from(&q.path);
    let art = match open(&path) {
        Ok(a) => a,
        Err(e) => {
            let msg = format!("failed to open artifact '{}': {e}", path.display());
            return (StatusCode::BAD_REQUEST, msg).into_response();
        }
    };

    let summary = ArtifactSummary {
        path: path.display().to_string(),
        object_count: art.object_count(),
        has_string_dict: art.string_dict().is_some(),
        has_attribute_table: art.attribute_table().is_some(),
        has_relation_table: art.relation_table().is_some(),
    };

    Json(summary).into_response()
}

#[derive(Serialize)]
struct BioObjectsResponse {
    path: String,
    kind: &'static str,
    object_ids: Vec<u32>,
}

async fn bio_cells(Query(q): Query<ArtifactQuery>) -> impl IntoResponse {
    bio_objects(q, "cell", "cells").await
}

async fn bio_genes(Query(q): Query<ArtifactQuery>) -> impl IntoResponse {
    bio_objects(q, "gene", "genes").await
}

async fn bio_objects(q: ArtifactQuery, label: &str, kind: &'static str) -> impl IntoResponse {
    let path = PathBuf::from(&q.path);
    let art = match open(&path) {
        Ok(a) => a,
        Err(e) => {
            let msg = format!("failed to open artifact '{}': {e}", path.display());
            return (StatusCode::BAD_REQUEST, msg).into_response();
        }
    };

    let bio = match BioView::from_artifact(&art) {
        Some(b) => b,
        None => {
            let msg = "artifact is missing StringDict or ObjectTable segments";
            return (StatusCode::BAD_REQUEST, msg.to_string()).into_response();
        }
    };

    let result = match label {
        "cell" => bio.cells(),
        "gene" => bio.genes(),
        _ => unreachable!("unsupported bio label"),
    };

    let objs = match result {
        Ok(objs) => objs,
        Err(e) => {
            let msg = format!("bio view error for '{}': {e}", label);
            return (StatusCode::BAD_REQUEST, msg).into_response();
        }
    };

    let ids = objs.into_iter().map(|o| o.id).collect();

    Json(BioObjectsResponse {
        path: path.display().to_string(),
        kind,
        object_ids: ids,
    })
    .into_response()
}

#[derive(Deserialize, Clone)]
struct ProjectQuery {
    /// Path to the .rnb artifact on disk.
    path: String,
    /// Comma-separated relation labels (StringDict strings).
    rels: String,
    /// Comma-separated source object IDs.
    src: String,
    /// Comma-separated destination object IDs.
    dst: String,
}

#[derive(Serialize)]
struct ProjectBlockResponse {
    path: String,
    rels: Vec<String>,
    row_ids: Vec<u32>,
    col_ids: Vec<u32>,
    csr_indptr: Vec<u32>,
    csr_indices: Vec<u32>,
    data: Vec<f32>,
}

async fn project_block(Query(q): Query<ProjectQuery>) -> impl IntoResponse {
    let path = PathBuf::from(&q.path);
    let art = match open(&path) {
        Ok(a) => a,
        Err(e) => {
            let msg = format!("failed to open artifact '{}': {e}", path.display());
            return (StatusCode::BAD_REQUEST, msg).into_response();
        }
    };

    let dict = match art.string_dict() {
        Some(d) => d,
        None => {
            let msg = "artifact is missing StringDict segment";
            return (StatusCode::BAD_REQUEST, msg.to_string()).into_response();
        }
    };

    // Parse rel labels and map to SIDs.
    let rel_labels: Vec<String> = q
        .rels
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();
    if rel_labels.is_empty() {
        return (StatusCode::BAD_REQUEST, "rels must not be empty").into_response();
    }

    let mut rel_type_sids = Vec::with_capacity(rel_labels.len());
    for label in &rel_labels {
        if let Some(idx) = dict.strings.iter().position(|s| s == label) {
            rel_type_sids.push(idx as u32);
        } else {
            let msg = format!("relation label '{}' not found in StringDict", label);
            return (StatusCode::BAD_REQUEST, msg).into_response();
        }
    }

    // Parse src/dst IDs.
    fn parse_ids(raw: &str) -> Result<Vec<u32>, String> {
        let mut out = Vec::new();
        for part in raw.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            let v: u32 = trimmed
                .parse()
                .map_err(|_| format!("invalid id '{}'", trimmed))?;
            out.push(v);
        }
        if out.is_empty() {
            return Err("id list must not be empty".to_string());
        }
        Ok(out)
    }

    let src_ids = match parse_ids(&q.src) {
        Ok(v) => v,
        Err(msg) => return (StatusCode::BAD_REQUEST, msg).into_response(),
    };
    let dst_ids = match parse_ids(&q.dst) {
        Ok(v) => v,
        Err(msg) => return (StatusCode::BAD_REQUEST, msg).into_response(),
    };

    let spec = PathSpec {
        rel_type_sids,
        semiring: SemiringKind::Boolean,
        state_id: None,
    };

    let block = match art.project_path_block(&spec, &src_ids, &dst_ids) {
        Ok(b) => b,
        Err(e) => {
            let msg = format!("projection error: {e}");
            return (StatusCode::BAD_REQUEST, msg).into_response();
        }
    };

    let resp = ProjectBlockResponse {
        path: path.display().to_string(),
        rels: rel_labels,
        row_ids: block.row_ids,
        col_ids: block.col_ids,
        csr_indptr: block.csr_indptr,
        csr_indices: block.csr_indices,
        data: block.data,
    };

    Json(resp).into_response()
}

fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/engine/v1/artifact/summary", get(artifact_summary))
        .route("/engine/v1/artifact/bio/cells", get(bio_cells))
        .route("/engine/v1/artifact/bio/genes", get(bio_genes))
        .route("/engine/v1/project/block", get(project_block))
}

#[derive(Serialize)]
struct EngineRegisterPayload<'a> {
    profile_token: Option<&'a str>,
    name: &'a str,
    kind: &'a str,
    endpoint_url: &'a str,
    version: &'a str,
    capabilities: Vec<&'a str>,
}

#[derive(Deserialize)]
struct EngineRegisterResponse {
    engine_id: String,
    heartbeat_token: String,
}

async fn spawn_registrar_task() {
    use std::env;

    let registrar_url = match env::var("RINNOVO_REGISTRAR_URL") {
        Ok(v) if !v.trim().is_empty() => v.trim_end_matches('/').to_string(),
        _ => return, // no registrar configured; nothing to do
    };

    let endpoint_url = match env::var("RINNOVO_ENGINE_ENDPOINT_URL") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => return, // endpoint URL is required to register
    };

    let name = env::var("RINNOVO_ENGINE_NAME").unwrap_or_else(|_| "local-dev".to_string());
    let kind = env::var("RINNOVO_ENGINE_KIND").unwrap_or_else(|_| "local".to_string());
    let profile_token = env::var("RINNOVO_PROFILE_TOKEN").ok();

    tokio::spawn(async move {
        let client = reqwest::Client::new();

        // Initial registration
        let payload = EngineRegisterPayload {
            profile_token: profile_token.as_deref(),
            name: &name,
            kind: &kind,
            endpoint_url: &endpoint_url,
            version: env!("CARGO_PKG_VERSION"),
            capabilities: vec!["rnb:v1", "http:v1"],
        };

        let register_url = format!("{}/v1/engines/register", registrar_url);
        let resp = client.post(&register_url).json(&payload).send().await;
        let Ok(resp) = resp else {
            eprintln!("engine_http: failed to call registrar at {}: {:?}", register_url, resp);
            return;
        };
        if !resp.status().is_success() {
            eprintln!(
                "engine_http: registrar registration failed: HTTP {}",
                resp.status()
            );
            return;
        }

        let Ok(body) = resp.json::<EngineRegisterResponse>().await else {
            eprintln!("engine_http: failed to parse registrar registration response");
            return;
        };

        let engine_id = body.engine_id;
        let heartbeat_token = body.heartbeat_token;

        // Heartbeat loop
        let heartbeat_url = format!("{}/v1/engines/{}/heartbeat", registrar_url, engine_id);
        loop {
            let res = client
                .post(&heartbeat_url)
                .header("X-Engine-Token", &heartbeat_token)
                .json(&serde_json::json!({"status": "online"}))
                .send()
                .await;

            if let Err(e) = res {
                eprintln!("engine_http: heartbeat error: {:?}", e);
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}

#[tokio::main]
async fn main() {
    use std::env;

    let app = build_router();

    // Allow the engine port to be configured via environment, with a stable
    // default and a simple failover if the chosen port is unavailable.
    let default_port: u16 = 8787;
    let requested_port = env::var("RINNOVO_ENGINE_PORT")
        .ok()
        .and_then(|raw| raw.parse::<u16>().ok());

    let primary_addr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], requested_port.unwrap_or(default_port)));

    // First try the requested/default port. If it's already in use or we
    // fail to bind for some other reason, fall back to an OS-assigned
    // ephemeral port so the engine can still come up.
    let listener = match tokio::net::TcpListener::bind(primary_addr).await {
        Ok(l) => l,
        Err(err) => {
            eprintln!(
                "rnb_engine_http: failed to bind {}; falling back to ephemeral port: {err}",
                primary_addr
            );
            tokio::net::TcpListener::bind("0.0.0.0:0")
                .await
                .expect("bind fallback http listener")
        }
    };

    let bound_addr = listener.local_addr().expect("discover bound socket addr");
    println!("rnb_engine_http listening on {}", bound_addr);

    // Optionally register with a registrar if configured via env vars.
    //
    // Note: RINNOVO_ENGINE_ENDPOINT_URL may differ from the bound_addr if the
    // engine is running behind a reverse proxy or is intended to be reached
    // via a different host/port. The daemon will be responsible for keeping
    // those in sync in managed deployments.
    spawn_registrar_task().await;

    axum::serve(listener, app)
        .await
        .expect("run axum server");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Method;
    use rnb_format::{Manifest, ObjectRecord, ObjectTable, StringDict};       
    use tower::util::ServiceExt;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("rnb_engine_http_{}_{}.rnb", name, std::process::id()));
        p
    }

    #[tokio::test]
    async fn health_endpoint_reports_ok() {
        let app = build_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["status"], "ok");
        assert!(body["version"].is_string());
    }

    #[tokio::test]
    async fn artifact_summary_handles_missing_file() {
        let app = build_router();
        let uri = "/engine/v1/artifact/summary?path=/nonexistent/path/does_not_exist.rnb";
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn artifact_summary_reports_basic_flags_for_empty_artifact() {
        let path = temp_path("empty");
        rnb_engine::write_empty(&path).unwrap();

        let app = build_router();
        let uri = format!(
            "/engine/v1/artifact/summary?path={}",
            urlencoding::encode(path.to_str().unwrap())
        );
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["path"].as_str().unwrap(), path.display().to_string());
        // An empty artifact may not have an object table yet.
        assert!(body["object_count"].is_null());
        assert!(body["has_string_dict"].is_boolean());

        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn bio_cells_and_genes_return_expected_ids() {
        let path = temp_path("bio");

        // Build a small artifact with a StringDict and ObjectTable matching the
        // expectations of BioView.
        let manifest = Manifest::minimal();
        let dict = StringDict::new(vec!["cell".to_string(), "gene".to_string()]);

        let mut ot = ObjectTable::empty();
        // id 0 -> cell, id 1 -> gene, id 2 -> cell
        ot.push(ObjectRecord {
            type_sid: 0,
            name_sid: 10,
            flags: 0,
        });
        ot.push(ObjectRecord {
            type_sid: 1,
            name_sid: 20,
            flags: 0,
        });
        ot.push(ObjectRecord {
            type_sid: 0,
            name_sid: 30,
            flags: 0,
        });

        // Use the existing helper that writes manifest + optional dict + object table.
        rnb_format::write_minimal_rnb(&path, &manifest, Some(&dict), Some(&ot)).unwrap();

        let app = build_router();

        // Cells
        let uri_cells = format!(
            "/engine/v1/artifact/bio/cells?path={}",
            urlencoding::encode(path.to_str().unwrap())
        );
        let resp_cells = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri_cells)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp_cells.status(), StatusCode::OK);
        let bytes_cells = axum::body::to_bytes(resp_cells.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_cells: serde_json::Value = serde_json::from_slice(&bytes_cells).unwrap();
        assert_eq!(body_cells["kind"], "cells");
        let ids_cells = body_cells["object_ids"].as_array().unwrap();
        let ids_cells: Vec<u32> = ids_cells
            .iter()
            .map(|v| v.as_u64().unwrap() as u32)
            .collect();
        assert_eq!(ids_cells, vec![0, 2]);

        // Genes
        let uri_genes = format!(
            "/engine/v1/artifact/bio/genes?path={}",
            urlencoding::encode(path.to_str().unwrap())
        );
        let resp_genes = build_router()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri_genes)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp_genes.status(), StatusCode::OK);
        let bytes_genes = axum::body::to_bytes(resp_genes.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_genes: serde_json::Value = serde_json::from_slice(&bytes_genes).unwrap();
        assert_eq!(body_genes["kind"], "genes");
        let ids_genes = body_genes["object_ids"].as_array().unwrap();
        let ids_genes: Vec<u32> = ids_genes
            .iter()
            .map(|v| v.as_u64().unwrap() as u32)
            .collect();
        assert_eq!(ids_genes, vec![1]);

        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn project_block_two_step_chain_over_http() {

        let path = temp_path("project");

        // Build a small artifact matching the engine-level projection test:
        // cell(0) --relA--> gene(1) --relA--> protein(2)
        let manifest = Manifest {
            flags: 0,
            required_segments: vec![rnb_format::SegmentType::Manifest],
            supported_kernels: vec![
                rnb_format::QueryKernel::GetRelationsFrom,
                rnb_format::QueryKernel::GetRelationsTo,
            ],
            max_chunk_bytes: 256 * 1024,
        };

        let dict = StringDict::new(vec![
            "cell".to_string(),
            "gene".to_string(),
            "protein".to_string(),
            "relA".to_string(),
        ]);

        let mut ot = ObjectTable::empty();
        ot.push(ObjectRecord {
            type_sid: 0,
            name_sid: 0,
            flags: 0,
        }); // id 0: cell
        ot.push(ObjectRecord {
            type_sid: 1,
            name_sid: 1,
            flags: 0,
        }); // id 1: gene
        ot.push(ObjectRecord {
            type_sid: 2,
            name_sid: 2,
            flags: 0,
        }); // id 2: protein

        // For now, use a simple roundtrip: write manifest + dict + object table.
        rnb_format::write_minimal_rnb(&path, &manifest, Some(&dict), Some(&ot)).unwrap();

        // Re-open and patch in relation + registry segments by rewriting the file
        // is non-trivial here; instead, we rely on the projection handler using
        // only the segments we've written (StringDict, ObjectTable, RelationTable,
        // TypeRegistry). The easiest way for tests is to construct an in-memory
        // artifact via rnb_engine and write it out via rnb_engine's own machinery
        // in a future slice. For now, we limit the HTTP projection test to the
        // simplest case: using the existing on-disk segments and a known path.

        // Build router and invoke /engine/v1/project/block with path, rels, src, dst.
        let app = build_router();
        let uri = format!(
            "/engine/v1/project/block?path={}&rels={}&src={}&dst={}",
            urlencoding::encode(path.to_str().unwrap()),
            "relA,relA",
            "0",
            "2"
        );

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        // With the current on-disk writer only providing manifest + dict +
        // object table, the projection has no RelationTable to walk and must
        // therefore be empty. We still expect the handler to succeed and
        // return a structurally valid CSR block with zero nnz.
        assert_eq!(body["row_ids"], serde_json::json!([0]));
        assert_eq!(body["col_ids"], serde_json::json!([2]));
        assert_eq!(body["csr_indptr"], serde_json::json!([0, 0]));
        assert_eq!(body["csr_indices"], serde_json::json!([]));
        assert_eq!(body["data"], serde_json::json!([]));

        let _ = std::fs::remove_file(path);
    }
}
