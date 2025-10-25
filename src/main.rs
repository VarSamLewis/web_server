// Import Axum web framework components:
// - Json: for handling JSON request/response bodies
// - Router: for defining HTTP routes and handlers
// - get, post: for specifying HTTP methods
use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
};
// Import Serde for JSON serialization/deserialization
use serde::{Deserialize, Serialize};
// Import SocketAddr for defining network addresses
use std::net::SocketAddr;

// Import tracing for logging and tracing
use tracing::{debug, error, info, warn};
use tracing_subscriber;

//Define error struct
#[allow(dead_code)]
enum AppError {
    ValidationError(String),
    NotFound(String),
    InternalError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
        };
        let body = Json(serde_json::json!({
            "error": message
        }));
        (status, body).into_response()
    }
}

// #[tokio::main] macro transforms this async main function into a synchronous entry point
// and sets up the Tokio async runtime for handling concurrent async operations
#[tokio::main]
async fn main() {
    // Build the application router with three routes:
    // 1. "/" - GET request returns a welcome message
    // 2. "/hello/:name" - GET request with a dynamic path parameter
    // 3. "/json" - POST request that accepts and returns JSON

    // Initialise tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/hello/:name", get(hello))
        .route("/create", post(handle_post))
        .route("/update", put(handle_put))
        .route("/update", patch(handle_patch))
        .route("/delete/:id", delete(handle_delete));

    // Create a socket address binding to localhost (127.0.0.1) on port 3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server running at http://{}", addr);

    // Start the HTTP server:
    // - bind(&addr): bind to the specified address
    // - serve(): start serving requests using the router
    // - await: asynchronously wait for the server to run (runs indefinitely)
    match axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        Ok(_) => info!("Server shut down gracefully"),
        Err(e) => {
            error!("Server error: {}", e);
            std::process::exit(1);
        }
    }
}
// Handler for the root route ("/")
// Returns a static string slice that lives for the entire program duration
async fn root() -> &'static str {
    "Welcome to the Rust Web Server!"
}

// Handler for the dynamic route ("/hello/:name")
// Extracts the "name" path parameter using Axum's Path extractor
// Example: GET /hello/Alice -> returns "Hello, Alice!"
async fn hello(axum::extract::Path(name): axum::extract::Path<String>) -> Result<String, AppError> {
    // Log incoming request with structured data
    info!(name = %name, "Processing hello request");

    if name.trim().is_empty() {
        warn!("Empty name provided");
        return Err(AppError::ValidationError(
            "Name cannot be empty".to_string(),
        ));
    }

    if name.len() > 50 {
        warn!(name_length = name.len(), "Name too long");
        return Err(AppError::ValidationError(
            "Name is too long (max 50 characters)".to_string(),
        ));
    }

    debug!(name = %name, "Validation passed");
    Ok(format!("Hello, {}!", name))
}

// Struct representing the expected JSON input from POST requests
// Deserialize trait allows automatic parsing from JSON to this struct
#[derive(Deserialize)]
struct InputData {
    name: String,
    age: u8, // u8 means unsigned 8-bit integer (0-255)
}

// Struct for PATCH requests - allows partial updates with optional fields
#[derive(Deserialize)]
struct PatchData {
    name: Option<String>,
    age: Option<u8>,
}

// Struct representing the JSON response to send back
// Serialize trait allows automatic conversion from this struct to JSON
#[derive(Serialize)]
struct ResponseData {
    message: String,
}

// Handler for the JSON POST route ("/json")
// Accepts JSON in the request body and returns JSON in the response
// Returns 201 CREATED status code for successful resource creation
// Example input: {"name": "Alice", "age": 30}
// Example output: {"message": "Hello, Alice! You are 30 years old."}
async fn handle_post(
    Json(payload): Json<InputData>,
) -> Result<(StatusCode, Json<ResponseData>), AppError> {
    // Structured logging with multiple fields
    info!(
        name = %payload.name,
        age = payload.age,
        "Processing JSON request"
    );

    if payload.name.trim().is_empty() {
        warn!("Empty name in JSON payload");
        return Err(AppError::ValidationError(
            "Name cannot be empty".to_string(),
        ));
    }

    if payload.name.len() > 50 {
        return Err(AppError::ValidationError(
            "Name is too long (max 50 characters)".to_string(),
        ));
    }

    if payload.age == 0 || payload.age > 120 {
        warn!(age = payload.age, "Invalid age provided");
        return Err(AppError::ValidationError("Invalid age".to_string()));
    }

    Ok((
        StatusCode::CREATED,
        Json(ResponseData {
            message: format!(
                "Hello, {}! You are {} years old.",
                payload.name, payload.age
            ),
        }),
    ))
}

async fn handle_put(
    Json(payload): Json<InputData>,
) -> Result<(StatusCode, Json<ResponseData>), AppError> {
    // Structured logging with multiple fields
    info!(
        name = %payload.name,
        age = payload.age,
        "Processing JSON request"
    );

    if payload.name.trim().is_empty() {
        warn!("Empty name in JSON payload");
        return Err(AppError::ValidationError(
            "Name cannot be empty".to_string(),
        ));
    }

    if payload.name.len() > 50 {
        return Err(AppError::ValidationError(
            "Name is too long (max 50 characters)".to_string(),
        ));
    }

    if payload.age == 0 || payload.age > 120 {
        warn!(age = payload.age, "Invalid age provided");
        return Err(AppError::ValidationError("Invalid age".to_string()));
    }

    Ok((
        StatusCode::OK,
        Json(ResponseData {
            message: format!(
                "Hello, {}! You are {} years old.",
                payload.name, payload.age
            ),
        }),
    ))
}

// Handler for PATCH requests - allows partial updates
// Only the fields provided will be updated
// Returns 200 OK status code for successful partial updates
// Example input: {"name": "Bob"} or {"age": 25} or both
async fn handle_patch(
    Json(payload): Json<PatchData>,
) -> Result<(StatusCode, Json<ResponseData>), AppError> {
    info!("Processing PATCH request");

    // Validate name if provided
    if let Some(ref name) = payload.name {
        if name.trim().is_empty() {
            warn!("Empty name in PATCH payload");
            return Err(AppError::ValidationError(
                "Name cannot be empty".to_string(),
            ));
        }
        if name.len() > 50 {
            return Err(AppError::ValidationError(
                "Name is too long (max 50 characters)".to_string(),
            ));
        }
        info!(name = %name, "Updating name");
    }

    // Validate age if provided
    if let Some(age) = payload.age {
        if age == 0 || age > 120 {
            warn!(age = age, "Invalid age in PATCH payload");
            return Err(AppError::ValidationError("Invalid age".to_string()));
        }
        info!(age = age, "Updating age");
    }

    // Build response message based on what was provided
    let message = match (payload.name, payload.age) {
        (Some(name), Some(age)) => format!("Updated: {}! You are {} years old.", name, age),
        (Some(name), None) => format!("Updated name to: {}", name),
        (None, Some(age)) => format!("Updated age to: {}", age),
        (None, None) => {
            warn!("No fields provided in PATCH request");
            return Err(AppError::ValidationError(
                "At least one field must be provided".to_string(),
            ));
        }
    };

    Ok((StatusCode::OK, Json(ResponseData { message })))
}

// Handler for DELETE requests - deletes a resource by ID
// Returns 200 OK status code with confirmation message
// Example: DELETE /delete/123
async fn handle_delete(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<(StatusCode, Json<ResponseData>), AppError> {
    info!(id = %id, "Processing DELETE request");

    // Validate ID is not empty
    if id.trim().is_empty() {
        warn!("Empty ID provided for deletion");
        return Err(AppError::ValidationError("ID cannot be empty".to_string()));
    }

    // In a real application, you would delete the resource from a database here
    // For this example, we'll just return a success message
    info!(id = %id, "Resource deleted successfully");

    Ok((
        StatusCode::OK,
        Json(ResponseData {
            message: format!("Resource with ID '{}' has been deleted", id),
        }),
    ))
}
