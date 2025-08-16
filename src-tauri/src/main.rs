// FileSense - Privacy-First File Organization
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FileInfo {
    name: String,
    path: String,
    size: u64,
    extension: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FolderAnalysis {
    total_files: usize,
    categories: HashMap<String, Vec<FileInfo>>,
}

// Smart file categorization logic
fn categorize_file(file_info: &FileInfo) -> String {
    let file_name = file_info.name.to_lowercase();
    let extension = file_info.extension.to_lowercase();
    
    // Check for sensitive files first
    if is_sensitive_file(&file_name) {
        return "Sensitive".to_string();
    }
    
    // Categorize by extension and content
    match extension.as_str() {
        // Documents
        "pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" => {
            if is_work_document(&file_name) {
                "Work Documents".to_string()
            } else {
                "Documents".to_string()
            }
        },
        
        // Spreadsheets & Presentations
        "xls" | "xlsx" | "csv" | "ods" | "ppt" | "pptx" | "odp" => "Documents".to_string(),
        
        // Images
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "svg" | "webp" | "heic" => {
            if is_personal_photo(&file_name) {
                "Personal Photos".to_string()
            } else {
                "Images".to_string()
            }
        },
        
        // Videos
        "mp4" | "avi" | "mov" | "wmv" | "flv" | "mkv" | "webm" | "m4v" => "Videos".to_string(),
        
        // Audio
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => "Audio".to_string(),
        
        // Archives
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => "Archives".to_string(),
        
        // Code files
        "js" | "ts" | "jsx" | "tsx" | "py" | "java" | "cpp" | "c" | "h" | "css" | "html" | 
        "php" | "rb" | "go" | "rs" | "swift" | "kt" | "cs" | "vb" | "sql" | "json" | "xml" | "yml" | "yaml" => {
            "Code".to_string()
        },
        
        // Executables and installers
        "exe" | "msi" | "dmg" | "pkg" | "deb" | "rpm" | "appx" | "app" => "Software".to_string(),
        
        // Default category
        _ => "Other".to_string(),
    }
}

// Check if file contains sensitive information
fn is_sensitive_file(file_name: &str) -> bool {
    let sensitive_keywords = [
        "tax", "irs", "w2", "1099", "ssn", "social", "security",
        "bank", "account", "statement", "routing", "financial",
        "password", "credential", "key", "secret", "login", "auth",
        "medical", "health", "prescription", "doctor", "patient",
        "personal", "private", "confidential", "classified"
    ];
    
    sensitive_keywords.iter().any(|&keyword| file_name.contains(keyword))
}

// Check if file is work-related
fn is_work_document(file_name: &str) -> bool {
    let work_keywords = [
        "meeting", "presentation", "report", "proposal", "contract",
        "client", "project", "deadline", "invoice", "budget",
        "company", "corporate", "business", "professional",
        "quarterly", "annual", "fiscal", "revenue", "salary"
    ];
    
    work_keywords.iter().any(|&keyword| file_name.contains(keyword))
}

// Check if image is a personal photo
fn is_personal_photo(file_name: &str) -> bool {
    let personal_keywords = [
        "vacation", "holiday", "trip", "travel", "family",
        "birthday", "wedding", "anniversary", "graduation",
        "photo", "pic", "img", "selfie", "camera"
    ];
    
    // Check for date patterns
    let has_date_pattern = file_name.contains("20") && 
        (file_name.contains("2023") || file_name.contains("2024") || file_name.contains("2025"));
    
    personal_keywords.iter().any(|&keyword| file_name.contains(keyword)) || has_date_pattern
}

// Get file size safely
fn get_file_size(path: &Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

// Get file extension safely
fn get_file_extension(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase()
}

#[tauri::command]
async fn select_folder() -> Result<String, String> {
    println!("📂 Opening folder selection dialog...");
    
    // Simple file picker - manually handle for now
    // We'll implement the actual dialog differently
    
    // For testing, let's use a hardcoded path first
    // Replace this with your actual Downloads or Desktop path for testing
    let test_path = std::env::var("HOME")
        .map(|home| format!("{}/Downloads", home))
        .unwrap_or_else(|_| "/Users".to_string());
    
    if std::path::Path::new(&test_path).exists() {
        println!("✅ Using test folder: {}", test_path);
        Ok(test_path)
    } else {
        Err("Test folder not found. Please check the path.".to_string())
    }
}

#[tauri::command]
async fn analyze_folder(folder_path: String) -> Result<FolderAnalysis, String> {
    println!("🔍 Starting analysis of folder: {}", folder_path);
    
    let path = Path::new(&folder_path);
    
    if !path.exists() {
        return Err(format!("Folder does not exist: {}", folder_path));
    }
    
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", folder_path));
    }
    
    let mut categories: HashMap<String, Vec<FileInfo>> = HashMap::new();
    let mut total_files = 0;
    
    // Initialize categories
    let category_names = vec![
        "Documents", "Images", "Videos", "Audio", "Archives", 
        "Code", "Software", "Work Documents", "Personal Photos", 
        "Sensitive", "Other"
    ];
    
    for category in category_names {
        categories.insert(category.to_string(), Vec::new());
    }
    
    // Read directory contents
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    };
    
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                println!("⚠️ Error reading entry: {}", e);
                continue;
            }
        };
        
        let file_path = entry.path();
        
        // Skip directories and hidden files
        if file_path.is_dir() || 
           file_path.file_name()
               .and_then(|name| name.to_str())
               .map(|name| name.starts_with('.'))
               .unwrap_or(false) {
            continue;
        }
        
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let file_info = FileInfo {
            name: file_name,
            path: file_path.to_string_lossy().to_string(),
            size: get_file_size(&file_path),
            extension: get_file_extension(&file_path),
        };
        
        let category = categorize_file(&file_info);
        
        if let Some(category_files) = categories.get_mut(&category) {
            category_files.push(file_info);
            total_files += 1;
        }
        
        // Log progress for large folders
        if total_files % 100 == 0 {
            println!("📊 Processed {} files...", total_files);
        }
    }
    
    println!("✅ Analysis complete: {} files categorized", total_files);
    
    // Log category summary
    for (category, files) in &categories {
        if !files.is_empty() {
            println!("📁 {}: {} files", category, files.len());
        }
    }
    
    Ok(FolderAnalysis {
        total_files,
        categories,
    })
}

#[tauri::command]
async fn organize_files(organization_plan: serde_json::Value) -> Result<String, String> {
    println!("📁 Starting file organization...");
    
    // For now, just return success - we'll implement actual file moving later
    // This prevents errors while we're building the UI
    
    println!("✅ Organization plan received: {}", organization_plan);
    Ok("Organization plan ready! (File moving not implemented yet)".to_string())
}

fn main() {
    println!("🚀 Starting FileSense...");
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            select_folder,
            analyze_folder,
            organize_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}