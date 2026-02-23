use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{
    extract::Query,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use rnb_engine::{open, BioView};
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

fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/engine/v1/artifact/summary", get(artifact_summary))
        .route("/engine/v1/artifact/bio/cells", get(bio_cells))
        .route("/engine/v1/artifact/bio/genes", get(bio_genes))
}

#[tokio::main]
async fn main() {
    let app = build_router();

    // For now bind to 0.0.0.0:8787 to match earlier design sketches.
    let addr: SocketAddr = "0.0.0.0:8787".parse().expect("valid socket addr");
    println!("rnb_engine_http listening on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr)
            .await
            .expect("bind http listener"),
        app,
    )
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
}
