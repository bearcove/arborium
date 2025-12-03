//! Showcase of miette-arborium syntax highlighting across various languages.
//!
//! Run with: cargo run --example miette_showcase -p miette-arborium --features all-languages

use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme, NamedSource, SourceSpan};
use miette_arborium::ArboriumHighlighter;
use std::error::Error;
use std::fmt;

/// A simple diagnostic error for demonstration
#[derive(Debug)]
struct CodeError {
    #[allow(dead_code)]
    message: String,
    src: NamedSource<String>,
    span: SourceSpan,
    label: String,
    help: Option<String>,
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl Error for CodeError {}

impl Diagnostic for CodeError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        Some(Box::new(std::iter::once(miette::LabeledSpan::at(
            self.span,
            &self.label,
        ))))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.help
            .as_ref()
            .map(|h| Box::new(h.as_str()) as Box<dyn fmt::Display>)
    }
}

fn print_diagnostic(error: &CodeError, handler: &GraphicalReportHandler) {
    let mut output = String::new();
    handler.render_report(&mut output, error).unwrap();
    println!("{output}");
}

fn main() {
    // Create the handler with arborium highlighting
    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode())
        .with_syntax_highlighting(ArboriumHighlighter::new())
        .with_context_lines(3);

    println!("\n🎨 miette-arborium Syntax Highlighting Showcase");
    println!("Theme: Monokai");
    println!("================================================\n");

    // Rust example
    let rust_code = r#"use std::collections::HashMap;

fn process_data(items: &[Item]) -> Result<Summary, Error> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for item in items {
        let key = item.category.clone();
        *counts.entry(key).or_insert(0) += 1;
    }

    Ok(Summary { counts, total: items.len() })
}

struct Summary {
    counts: HashMap<String, usize>,
    total: usize,
}"#;

    let rust_error = CodeError {
        message: "Rust type error".into(),
        src: NamedSource::new("processor.rs", rust_code.to_string()).with_language("rust"),
        span: (246, 9).into(), // "or_insert"
        label: "expected `&mut usize`, found `usize`".into(),
        help: Some("consider using `entry(...).or_insert(0)` pattern correctly".into()),
    };
    println!("📦 Rust\n");
    print_diagnostic(&rust_error, &handler);

    // Python example
    let python_code = r#"import asyncio
from dataclasses import dataclass
from typing import List, Optional

@dataclass
class User:
    name: str
    email: str
    age: Optional[int] = None

async def fetch_users(api_url: str) -> List[User]:
    """Fetch users from the API endpoint."""
    async with aiohttp.ClientSession() as session:
        async with session.get(api_url) as response:
            data = await response.json()
            return [User(**item) for item in data]

if __name__ == "__main__":
    users = asyncio.run(fetch_users("https://api.example.com/users"))
    print(f"Found {len(users)} users")"#;

    let python_error = CodeError {
        message: "Python import error".into(),
        src: NamedSource::new("fetch_users.py", python_code.to_string()).with_language("python"),
        span: (278, 7).into(), // "aiohttp"
        label: "ModuleNotFoundError: No module named 'aiohttp'".into(),
        help: Some("try: pip install aiohttp".into()),
    };
    println!("\n🐍 Python\n");
    print_diagnostic(&python_error, &handler);

    // TypeScript example
    let typescript_code = r#"interface ApiResponse<T> {
  data: T;
  status: number;
  message?: string;
}

type UserRole = 'admin' | 'user' | 'guest';

interface User {
  id: number;
  name: string;
  email: string;
  role: UserRole;
}

async function getUser(id: number): Promise<ApiResponse<User>> {
  const response = await fetch(`/api/users/${id}`);
  const data: User = await response.json();
  return { data, status: response.status };
}

const handleUser = (user: User): void => {
  console.log(`Welcome, ${user.name}!`);
};"#;

    let ts_error = CodeError {
        message: "TypeScript type error".into(),
        src: NamedSource::new("api.ts", typescript_code.to_string()).with_language("typescript"),
        span: (395, 15).into(), // "response.status"
        label: "Property 'status' does not exist on type 'Response'".into(),
        help: Some("Did you mean to use response.ok or response.statusText?".into()),
    };
    println!("\n📘 TypeScript\n");
    print_diagnostic(&ts_error, &handler);

    // Go example
    let go_code = r#"package main

import (
    "context"
    "encoding/json"
    "fmt"
    "net/http"
    "time"
)

type Server struct {
    addr    string
    timeout time.Duration
}

func (s *Server) HandleRequest(ctx context.Context, w http.ResponseWriter, r *http.Request) error {
    ctx, cancel := context.WithTimeout(ctx, s.timeout)
    defer cancel()

    data := map[string]interface{}{
        "status": "ok",
        "time":   time.Now().Unix(),
    }

    return json.NewEncoder(w).Encode(data)
}

func main() {
    srv := &Server{addr: ":8080", timeout: 30 * time.Second}
    fmt.Printf("Server starting on %s\n", srv.addr)
}"#;

    let go_error = CodeError {
        message: "Go error".into(),
        src: NamedSource::new("server.go", go_code.to_string()).with_language("go"),
        span: (330, 6).into(), // "cancel"
        label: "cancel declared and not used".into(),
        help: Some("consider calling cancel() or removing the declaration".into()),
    };
    println!("\n🔵 Go\n");
    print_diagnostic(&go_error, &handler);

    // JSON example
    let json_code = r#"{
  "name": "miette-arborium",
  "version": "0.1.0",
  "dependencies": {
    "arborium": "^0.1.0",
    "miette": "^7.0.0",
    "owo-colors": "^4.0.0"
  },
  "features": {
    "default": ["all-languages"],
    "all-languages": true
  },
  "repository": {
    "type": "git",
    "url": "https://github.com/bearcove/arborium"
  }
}"#;

    let json_error = CodeError {
        message: "JSON parse error".into(),
        src: NamedSource::new("package.json", json_code.to_string()).with_language("json"),
        span: (226, 4).into(), // "true"
        label: "expected array, found boolean".into(),
        help: Some("feature values should be arrays of feature names".into()),
    };
    println!("\n📋 JSON\n");
    print_diagnostic(&json_error, &handler);

    // TOML example
    let toml_code = r#"[package]
name = "my-awesome-crate"
version = "1.0.0"
edition = "2021"
authors = ["Developer <dev@example.com>"]
description = "An awesome Rust crate"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
criterion = "0.5"

[[bin]]
name = "my-app"
path = "src/main.rs"

[features]
default = ["std"]
std = []
async = ["tokio"]"#;

    let toml_error = CodeError {
        message: "TOML error".into(),
        src: NamedSource::new("Cargo.toml", toml_code.to_string()).with_language("toml"),
        span: (287, 9).into(), // "criterion"
        label: "failed to select a version for `criterion`".into(),
        help: Some("versions that meet the requirements `0.5` are: 0.5.1".into()),
    };
    println!("\n📦 TOML\n");
    print_diagnostic(&toml_error, &handler);

    // SQL example
    let sql_code = r#"-- User analytics query
SELECT
    u.id,
    u.name,
    u.email,
    COUNT(o.id) as order_count,
    SUM(o.total) as total_spent,
    AVG(o.total) as avg_order_value
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
WHERE u.created_at >= '2024-01-01'
  AND u.status = 'active'
GROUP BY u.id, u.name, u.email
HAVING COUNT(o.id) > 5
ORDER BY total_spent DESC
LIMIT 100;"#;

    let sql_error = CodeError {
        message: "SQL error".into(),
        src: NamedSource::new("analytics.sql", sql_code.to_string()).with_language("sql"),
        span: (343, 11).into(), // "total_spent" in ORDER BY
        label: "column 'total_spent' must appear in GROUP BY clause".into(),
        help: Some("add total_spent to GROUP BY or use an aggregate function".into()),
    };
    println!("\n🗄️ SQL\n");
    print_diagnostic(&sql_error, &handler);

    // YAML example
    let yaml_code = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
  labels:
    app: my-app
    environment: production
spec:
  replicas: 3
  selector:
    matchLabels:
      app: my-app
  template:
    metadata:
      labels:
        app: my-app
    spec:
      containers:
        - name: my-app
          image: my-registry/my-app:latest
          ports:
            - containerPort: 8080
          resources:
            limits:
              memory: "256Mi"
              cpu: "500m""#;

    let yaml_error = CodeError {
        message: "YAML validation error".into(),
        src: NamedSource::new("deployment.yaml", yaml_code.to_string()).with_language("yaml"),
        span: (451, 7).into(), // "256Mi" memory limit
        label: "resource limit too low for production workload".into(),
        help: Some("consider increasing CPU limit to at least 1000m".into()),
    };
    println!("\n📄 YAML\n");
    print_diagnostic(&yaml_error, &handler);

    // Bash example
    let bash_code = r#"#!/bin/bash
set -euo pipefail

# Configuration
readonly LOG_DIR="/var/log/myapp"
readonly MAX_RETRIES=3

function log_message() {
    local level="$1"
    local message="$2"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$level] $message" >> "$LOG_DIR/app.log"
}

function deploy_application() {
    local version="${1:-latest}"

    for i in $(seq 1 $MAX_RETRIES); do
        if docker pull "myapp:$version"; then
            log_message "INFO" "Successfully pulled version $version"
            return 0
        fi
        log_message "WARN" "Attempt $i failed, retrying..."
        sleep 5
    done

    log_message "ERROR" "Failed to pull after $MAX_RETRIES attempts"
    return 1
}

deploy_application "$@""#;

    let bash_error = CodeError {
        message: "Bash error".into(),
        src: NamedSource::new("deploy.sh", bash_code.to_string()).with_language("bash"),
        span: (64, 16).into(), // "/var/log/myapp"
        label: "directory does not exist".into(),
        help: Some("create the directory with: mkdir -p /var/log/myapp".into()),
    };
    println!("\n🐚 Bash\n");
    print_diagnostic(&bash_error, &handler);

    println!("\n================================================");
    println!("🎨 Highlighting powered by arborium + tree-sitter\n");
}
