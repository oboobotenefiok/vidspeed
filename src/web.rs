use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use maud::{html, Markup};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use std::sync::RwLock;
use uuid::Uuid;
use crate::config::Config;
use crate::processor::{VideoProcessor, cleanup_old_files};
use tracing::{info, error};
use chrono::Utc;

pub struct AppState {
    config: Config,
    semaphore: Semaphore,
    jobs: Arc<RwLock<HashMap<String, JobStatus>>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JobStatus {
    id: String,
    status: String,
    progress: u8,
    output_file: Option<String>,
    input_file: Option<String>,
    original_filename: Option<String>,
    error: Option<String>,
    created_at: chrono::DateTime<Utc>,
    speed: f64,
}

pub async fn run_server(config: Config, port: u16, host: String) -> Result<(), anyhow::Error> {
    tokio::fs::create_dir_all(&config.upload_dir).await?;
    tokio::fs::create_dir_all(&config.processed_dir).await?;

    let max_body = config.max_file_size_mb * 1024 * 1024;

    let state = Arc::new(AppState {
        config,
        semaphore: Semaphore::new(4),
        jobs: Arc::new(RwLock::new(HashMap::new())),
    });

    // Hourly cleanup task
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let upload_dir = state_clone.config.upload_dir.clone();
            let ttl = state_clone.config.temp_file_ttl_hours;
            if let Err(e) = cleanup_old_files(std::path::Path::new(&upload_dir), ttl).await {
                error!("Cleanup failed: {}", e);
            }
            let processed_dir = state_clone.config.processed_dir.clone();
            if let Err(e) = cleanup_old_files(std::path::Path::new(&processed_dir), ttl).await {
                error!("Cleanup failed: {}", e);
            }
        }
    });

    let app = Router::new()
        .route("/", get(index_page))
        .route("/api/upload", post(upload_handler))
        .route("/api/status/:job_id", get(status_handler))
        .route("/api/download/:job_id", get(download_handler))
        .layer(DefaultBodyLimit::max(max_body as usize))
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index_page() -> Html<String> {
    let markup = html! {
        html {
            head {
                title { "VidSpeed – Video Speed Changer" }
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                style {
                    r#"
                    *, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }

                    :root {
                        --bg:       #0a0a0a;
                        --surface:  #111111;
                        --border:   #2a2a2a;
                        --border-hi:#555555;
                        --text:     #f0f0f0;
                        --muted:    #888888;
                        --accent:   #ffffff;
                        --danger:   #ff4444;
                        --success:  #44cc44;
                    }

                    body {
                        font-family: 'Segoe UI', system-ui, sans-serif;
                        background: var(--bg);
                        color: var(--text);
                        min-height: 100vh;
                        display: flex;
                        align-items: flex-start;
                        justify-content: center;
                        padding: 40px 20px;
                    }

                    .container {
                        width: 100%;
                        max-width: 860px;
                        background: var(--surface);
                        border: 1px solid var(--border);
                        border-radius: 16px;
                        padding: 40px;
                    }

                    h1 {
                        font-size: 2rem;
                        font-weight: 700;
                        letter-spacing: -0.5px;
                        color: var(--accent);
                        margin-bottom: 6px;
                    }

                    .badge {
                        display: inline-block;
                        background: #1a1a1a;
                        border: 1px solid var(--border);
                        color: var(--muted);
                        padding: 2px 10px;
                        border-radius: 20px;
                        font-size: 0.72rem;
                        font-weight: 600;
                        letter-spacing: 0.05em;
                        text-transform: uppercase;
                        vertical-align: middle;
                        margin-left: 10px;
                    }

                    .subtitle {
                        color: var(--muted);
                        margin-bottom: 36px;
                        font-size: 0.9rem;
                    }

                    .upload-area {
                        border: 2px dashed var(--border);
                        border-radius: 12px;
                        padding: 48px 24px;
                        text-align: center;
                        cursor: pointer;
                        transition: border-color 0.2s, background 0.2s;
                        margin-bottom: 28px;
                        user-select: none;
                    }

                    .upload-area:hover,
                    .upload-area.dragover {
                        border-color: var(--accent);
                        background: #161616;
                    }

                    .upload-area.has-file {
                        border-color: var(--success);
                        background: #0d1a0d;
                    }

                    input[type="file"] { display: none; }

                    .upload-icon {
                        font-size: 3rem;
                        margin-bottom: 12px;
                        color: var(--muted);
                        display: block;
                    }

                    .upload-area h3 {
                        font-size: 1rem;
                        font-weight: 600;
                        color: var(--text);
                        margin-bottom: 6px;
                    }

                    .upload-area p {
                        font-size: 0.82rem;
                        color: var(--muted);
                    }

                    .file-name {
                        margin-top: 10px;
                        font-size: 0.82rem;
                        color: var(--success);
                        font-weight: 500;
                    }

                    .preview-section {
                        margin: 0 0 24px;
                        display: none;
                    }

                    .preview-section.active {
                        display: block;
                    }

                    .preview-title {
                        font-size: 0.82rem;
                        font-weight: 600;
                        color: var(--muted);
                        text-transform: uppercase;
                        letter-spacing: 0.06em;
                        margin-bottom: 12px;
                    }

                    .video-preview {
                        width: 100%;
                        background: #000;
                        border-radius: 8px;
                        overflow: hidden;
                        border: 1px solid var(--border);
                    }

                    .video-preview video {
                        width: 100%;
                        max-height: 300px;
                        object-fit: contain;
                        display: block;
                    }

                    .video-info {
                        margin-top: 8px;
                        font-size: 0.75rem;
                        color: var(--muted);
                        display: flex;
                        gap: 16px;
                        justify-content: center;
                    }

                    .speed-control {
                        margin: 0 0 28px;
                    }

                    .speed-control label {
                        display: block;
                        font-size: 0.82rem;
                        font-weight: 600;
                        color: var(--muted);
                        text-transform: uppercase;
                        letter-spacing: 0.06em;
                        margin-bottom: 10px;
                    }

                    input[type="range"] {
                        -webkit-appearance: none;
                        width: 100%;
                        height: 4px;
                        background: var(--border);
                        border-radius: 2px;
                        outline: none;
                        cursor: pointer;
                    }

                    input[type="range"]::-webkit-slider-thumb {
                        -webkit-appearance: none;
                        width: 18px;
                        height: 18px;
                        border-radius: 50%;
                        background: var(--accent);
                        cursor: pointer;
                        transition: transform 0.1s;
                    }

                    input[type="range"]::-webkit-slider-thumb:hover {
                        transform: scale(1.2);
                    }

                    .speed-value {
                        text-align: center;
                        margin-top: 10px;
                        font-size: 2rem;
                        font-weight: 700;
                        color: var(--accent);
                        letter-spacing: -1px;
                    }

                    button {
                        width: 100%;
                        background: var(--accent);
                        color: #000;
                        padding: 14px;
                        border: none;
                        border-radius: 8px;
                        font-size: 0.95rem;
                        font-weight: 700;
                        letter-spacing: 0.03em;
                        cursor: pointer;
                        transition: opacity 0.2s, transform 0.15s;
                    }

                    button:hover:not(:disabled) {
                        opacity: 0.88;
                        transform: translateY(-1px);
                    }

                    button:disabled {
                        background: var(--border);
                        color: var(--muted);
                        cursor: not-allowed;
                        transform: none;
                    }

                    .job-list { margin-top: 32px; }

                    .job-list-title {
                        font-size: 0.75rem;
                        font-weight: 600;
                        text-transform: uppercase;
                        letter-spacing: 0.08em;
                        color: var(--muted);
                        margin-bottom: 12px;
                    }

                    .job-item {
                        background: #161616;
                        border: 1px solid var(--border);
                        border-radius: 10px;
                        padding: 16px;
                        margin-bottom: 10px;
                    }

                    .job-header {
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        margin-bottom: 10px;
                        font-size: 0.82rem;
                        color: var(--muted);
                    }

                    .job-header strong { color: var(--text); font-size: 0.88rem; }

                    .progress-bar {
                        background: var(--border);
                        border-radius: 4px;
                        overflow: hidden;
                        height: 6px;
                        margin-bottom: 10px;
                    }

                    .progress-fill {
                        background: var(--accent);
                        height: 100%;
                        transition: width 0.35s ease;
                        width: 0%;
                    }

                    .job-status {
                        font-size: 0.82rem;
                        color: var(--muted);
                    }

                    .download-link {
                        display: inline-block;
                        color: var(--accent);
                        font-weight: 600;
                        text-decoration: none;
                        font-size: 0.85rem;
                        border: 1px solid var(--border-hi);
                        padding: 5px 12px;
                        border-radius: 6px;
                        transition: background 0.15s;
                    }

                    .download-link:hover { background: #222; }

                    .error-msg { color: var(--danger); font-size: 0.82rem; }
                    "#
                }
            }
            body {
                div class="container" {
                    h1 {
                        "VidSpeed"
                        span class="badge" { "No Audio" }
                    }
                    p class="subtitle" { "Change your video speed — audio stripped automatically." }

                    div class="upload-area" id="uploadArea" {
                        span class="upload-icon" { "▲" }
                        h3 { "Click or drag a video here" }
                        p { "MP4, AVI, MOV, MKV — up to 500 MB" }
                        div class="file-name" id="fileName" {}
                        input type="file" id="fileInput" accept="video/*" {}
                    }

                    div class="preview-section" id="previewSection" {
                        div class="preview-title" { "Preview" }
                        div class="video-preview" {
                            video id="videoPreview" controls {}
                        }
                        div class="video-info" {
                            span id="videoDuration" { "" }
                            span id="videoSize" { "" }
                        }
                    }

                    div class="speed-control" {
                        label { "Speed Factor" }
                        input type="range" id="speed" min="0.5" max="4" step="0.1" value="2.0";
                        div class="speed-value" {
                            span id="speedValue" { "2.0×" }
                        }
                    }

                    button id="processBtn" disabled { "Select a video to begin" }

                    div class="job-list" id="jobList" {}
                }

                script {
                    r#"
                    document.addEventListener('DOMContentLoaded', () => {
                        const uploadArea  = document.getElementById('uploadArea');
                        const fileInput   = document.getElementById('fileInput');
                        const fileNameEl  = document.getElementById('fileName');
                        const previewSection = document.getElementById('previewSection');
                        const videoPreview = document.getElementById('videoPreview');
                        const videoDuration = document.getElementById('videoDuration');
                        const videoSize = document.getElementById('videoSize');
                        const speedSlider = document.getElementById('speed');
                        const speedValue  = document.getElementById('speedValue');
                        const processBtn  = document.getElementById('processBtn');
                        const jobList     = document.getElementById('jobList');

                        let currentFile = null;
                        let activeJobs = new Map();

                        speedSlider.addEventListener('input', (e) => {
                            const val = parseFloat(e.target.value).toFixed(1);
                            speedValue.textContent = val + '×';
                        });

                        uploadArea.addEventListener('click', () => fileInput.click());

                        uploadArea.addEventListener('dragover', (e) => {
                            e.preventDefault();
                            uploadArea.classList.add('dragover');
                        });
                        uploadArea.addEventListener('dragleave', () => {
                            uploadArea.classList.remove('dragover');
                        });
                        uploadArea.addEventListener('drop', (e) => {
                            e.preventDefault();
                            uploadArea.classList.remove('dragover');
                            const file = e.dataTransfer.files[0];
                            if (file && file.type.startsWith('video/')) {
                                handleFile(file);
                            }
                        });

                        fileInput.addEventListener('change', (e) => {
                            const file = e.target.files[0];
                            if (file) handleFile(file);
                        });

                        function handleFile(file) {
                            currentFile = file;
                            uploadArea.classList.add('has-file');
                            fileNameEl.textContent = '✓ ' + file.name;
                            processBtn.disabled = false;
                            processBtn.textContent = 'Process Video';
                            showVideoPreview(file);
                        }

                        function showVideoPreview(file) {
                            const url = URL.createObjectURL(file);
                            videoPreview.src = url;
                            previewSection.classList.add('active');
                            
                            videoPreview.onloadedmetadata = () => {
                                const duration = videoPreview.duration;
                                const minutes = Math.floor(duration / 60);
                                const seconds = Math.floor(duration % 60);
                                videoDuration.textContent = `Duration: ${minutes}:${seconds.toString().padStart(2, '0')}`;
                                const sizeMB = (file.size / (1024 * 1024)).toFixed(2);
                                videoSize.textContent = `Size: ${sizeMB} MB`;
                            };
                            
                            videoPreview.onended = () => {
                                URL.revokeObjectURL(url);
                            };
                        }

                        processBtn.addEventListener('click', async () => {
                            if (!currentFile) return;

                            processBtn.disabled = true;
                            processBtn.textContent = 'Uploading…';

                            const formData = new FormData();
                            formData.append('video', currentFile);
                            formData.append('speed', speedSlider.value);

                            try {
                                const response = await fetch('/api/upload', {
                                    method: 'POST',
                                    body: formData
                                });
                                const data = await response.json();
                                if (data.job_id) {
                                    addJobToList(data.job_id, currentFile.name, parseFloat(speedSlider.value));
                                    pollJobStatus(data.job_id);
                                    
                                    previewSection.classList.remove('active');
                                    videoPreview.src = '';
                                    currentFile = null;
                                    fileInput.value = '';
                                    fileNameEl.textContent = '';
                                    uploadArea.classList.remove('has-file');
                                    processBtn.textContent = 'Select a video to begin';
                                    processBtn.disabled = true;
                                } else {
                                    alert('Error: ' + (data.error || 'Unknown error'));
                                    processBtn.disabled = false;
                                    processBtn.textContent = 'Process Video';
                                }
                            } catch (err) {
                                alert('Upload failed: ' + err.message);
                                processBtn.disabled = false;
                                processBtn.textContent = 'Process Video';
                            }
                        });

                        function addJobToList(jobId, filename, speed) {
                            if (jobList.children.length === 0) {
                                const title = document.createElement('div');
                                title.className = 'job-list-title';
                                title.textContent = 'Recent Jobs';
                                jobList.appendChild(title);
                            }
                            
                            if (document.getElementById('job-' + jobId)) {
                                return;
                            }
                            
                            const jobDiv = document.createElement('div');
                            jobDiv.className = 'job-item';
                            jobDiv.id = 'job-' + jobId;
                            jobDiv.innerHTML = `
                                <div class="job-header">
                                    <strong>${escapeHtml(filename)}</strong>
                                    <span>${speed.toFixed(1)}× speed</span>
                                </div>
                                <div class="progress-bar">
                                    <div class="progress-fill" id="pf-${jobId}"></div>
                                </div>
                                <div class="job-status" id="st-${jobId}">Queued…</div>
                            `;
                            jobList.insertBefore(jobDiv, jobList.firstChild?.nextSibling || jobList.firstChild);
                            activeJobs.set(jobId, { filename, speed });
                        }

                        async function pollJobStatus(jobId) {
                            const interval = setInterval(async () => {
                                try {
                                    const res = await fetch('/api/status/' + jobId);
                                    const s = await res.json();

                                    const pf = document.getElementById('pf-' + jobId);
                                    const st = document.getElementById('st-' + jobId);
                                    if (!pf || !st) { 
                                        clearInterval(interval); 
                                        activeJobs.delete(jobId);
                                        return; 
                                    }

                                    pf.style.width = (s.progress || 0) + '%';

                                    if (s.status === 'completed') {
                                        st.innerHTML = '<a class="download-link" href="/api/download/' + jobId + '">↓ Download Video</a>';
                                        clearInterval(interval);
                                        activeJobs.delete(jobId);
                                    } else if (s.status === 'failed') {
                                        st.innerHTML = '<span class="error-msg">✗ ' + escapeHtml(s.error || 'Processing failed') + '</span>';
                                        clearInterval(interval);
                                        activeJobs.delete(jobId);
                                    } else {
                                        const statusText = s.status === 'processing' ? '⚙ Processing...' : '⏳ Queued...';
                                        st.textContent = statusText;
                                        if (s.status === 'processing' && s.progress > 0 && s.progress < 100) {
                                            st.textContent = `⚙ Processing... ${Math.round(s.progress)}%`;
                                        }
                                    }
                                } catch (err) {
                                    console.error('Poll error:', err);
                                }
                            }, 1500);
                        }
                        
                        function escapeHtml(str) {
                            if (!str) return '';
                            return str.replace(/[&<>]/g, function(m) {
                                if (m === '&') return '&amp;';
                                if (m === '<') return '&lt;';
                                if (m === '>') return '&gt;';
                                return m;
                            });
                        }
                        
                        async function loadExistingJobs() {
                            const jobItems = jobList.querySelectorAll('.job-item');
                            if (jobItems.length === 0) {
                                const emptyMsg = document.createElement('div');
                                emptyMsg.className = 'job-item';
                                emptyMsg.style.textAlign = 'center';
                                emptyMsg.style.color = 'var(--muted)';
                                emptyMsg.innerHTML = 'No jobs yet. Upload a video to get started!';
                                jobList.appendChild(emptyMsg);
                            }
                        }
                        
                        loadExistingJobs();
                    });
                    "#
                }
            }
        }
    };

    Html(markup.into_string())
}

async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Response {
    let mut video_data = None;
    let mut speed = 2.0f64;
    let mut original_filename = String::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "video" => {
                original_filename = field.file_name()
                    .unwrap_or("video.mp4")
                    .to_string();
                match field.bytes().await {
                    Ok(data) => video_data = Some(data),
                    Err(e) => {
                        return Json(serde_json::json!({
                            "error": format!("Failed to read video field: {}", e)
                        }))
                        .into_response();
                    }
                }
            }
            "speed" => {
                if let Ok(val) = field.text().await {
                    speed = val.parse().unwrap_or(2.0);
                }
            }
            _ => {}
        }
    }

    let video_data = match video_data {
        Some(d) => d,
        None => {
            return Json(serde_json::json!({ "error": "No video file received" }))
                .into_response();
        }
    };

    let max_size = state.config.max_file_size_mb * 1024 * 1024;
    if video_data.len() > max_size as usize {
        return Json(serde_json::json!({
            "error": format!("File too large (max {}MB)", state.config.max_file_size_mb)
        }))
        .into_response();
    }

    let job_id = Uuid::new_v4().to_string();
    let input_path = std::path::Path::new(&state.config.upload_dir)
        .join(format!("{}.mp4", job_id));
    let output_path = std::path::Path::new(&state.config.processed_dir)
        .join(format!("{}_out.mp4", job_id));

    if let Err(e) = tokio::fs::write(&input_path, &video_data).await {
        error!("Failed to save uploaded file: {}", e);
        return Json(serde_json::json!({ "error": "Failed to save uploaded file" }))
            .into_response();
    }

    let status = JobStatus {
        id: job_id.clone(),
        status: "queued".to_string(),
        progress: 0,
        output_file: None,
        input_file: Some(input_path.to_string_lossy().to_string()),
        original_filename: Some(original_filename),
        error: None,
        created_at: Utc::now(),
        speed,
    };

    {
        let mut jobs = state.jobs.write().unwrap();
        jobs.insert(job_id.clone(), status);
    }

    // Spawn background processing
    let state_clone = state.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        let _permit = state_clone.semaphore.acquire().await.unwrap();

        {
            let mut jobs = state_clone.jobs.write().unwrap();
            if let Some(job) = jobs.get_mut(&job_id_clone) {
                job.status = "processing".to_string();
                job.progress = 5;
            }
        }

        let processor = VideoProcessor::new(input_path.clone(), output_path.clone(), speed);

        match processor.process().await {
            Ok(_) => {
                let mut jobs = state_clone.jobs.write().unwrap();
                if let Some(job) = jobs.get_mut(&job_id_clone) {
                    job.status = "completed".to_string();
                    job.progress = 100;
                    job.output_file = Some(output_path.to_string_lossy().to_string());
                }
                info!("Job {} completed", job_id_clone);
            }
            Err(e) => {
                let mut jobs = state_clone.jobs.write().unwrap();
                if let Some(job) = jobs.get_mut(&job_id_clone) {
                    job.status = "failed".to_string();
                    job.error = Some(e.to_string());
                }
                error!("Job {} failed: {}", job_id_clone, e);
            }
        }
        
        // Clean up input file after processing
        let _ = tokio::fs::remove_file(input_path).await;
    });

    Json(serde_json::json!({ "job_id": job_id, "status": "queued" })).into_response()
}

async fn status_handler(
    Path(job_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let jobs = state.jobs.read().unwrap();

    match jobs.get(&job_id) {
        Some(job) => Json(serde_json::json!({
            "status":    job.status,
            "progress":  job.progress,
            "error":     job.error,
            "completed": job.status == "completed",
            "speed":     job.speed,
            "original_filename": job.original_filename,
        }))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Job not found" })),
        )
            .into_response(),
    }
}

async fn download_handler(
    Path(job_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let jobs = state.jobs.read().unwrap();

    if let Some(job) = jobs.get(&job_id) {
        if job.status == "completed" {
            if let Some(path) = &job.output_file {
                if let Ok(data) = tokio::fs::read(path).await {
                    let original_name = job.original_filename.as_deref().unwrap_or("video");
                    let stem = std::path::Path::new(original_name)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy();
                    let filename = format!("{}_speed{}x_noaudio.mp4", stem, job.speed);
                    
                    return (
                        StatusCode::OK,
                        [
                            (header::CONTENT_TYPE, "video/mp4"),
                            (
                                header::CONTENT_DISPOSITION,
                                &format!("attachment; filename=\"{}\"", filename),
                            ),
                        ],
                        data,
                    )
                        .into_response();
                }
            }
        }
    }

    (StatusCode::NOT_FOUND, "File not found").into_response()
}
