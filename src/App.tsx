import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface FileInfo {
  name: string;
  path: string;
  size: number;
  extension: string;
}

interface FolderAnalysis {
  total_files: number;
  categories: Record<string, FileInfo[]>;
}

function App() {
  const [selectedFolder, setSelectedFolder] = useState<string>("");
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [analysis, setAnalysis] = useState<FolderAnalysis | null>(null);
  const [error, setError] = useState<string>("");

  async function selectFolder() {
    try {
      setError("");
      const folderPath = await invoke<string>("select_folder");
      if (folderPath) {
        setSelectedFolder(folderPath);
        await analyzeFolder(folderPath);
      }
    } catch (err) {
      setError(`Failed to select folder: ${err}`);
    }
  }

  async function analyzeFolder(folderPath: string) {
    try {
      setIsAnalyzing(true);
      setError("");
      
      const result = await invoke<FolderAnalysis>("analyze_folder", { 
        folderPath 
      });
      
      setAnalysis(result);
    } catch (err) {
      setError(`Failed to analyze folder: ${err}`);
    } finally {
      setIsAnalyzing(false);
    }
  }

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return "0 Bytes";
    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  const getCategoryIcon = (category: string): string => {
    const icons: Record<string, string> = {
      "Documents": "📄",
      "Images": "🖼️",
      "Videos": "🎥",
      "Audio": "🎵",
      "Archives": "📦",
      "Code": "💻",
      "Other": "📁"
    };
    return icons[category] || "📁";
  };

  return (
    <main className="container">
      {/* Header */}
      <header className="header">
        <h1>🧠 FileSense</h1>
        <p>Privacy-First Intelligent File Organization</p>
        <div className="privacy-badge">
          🔒 100% Local Processing
        </div>
      </header>

      {/* Folder Selection */}
      <section className="folder-section">
        <h2>📁 Select Folder to Organize</h2>
        <div className="folder-selector">
          <button 
            onClick={selectFolder}
            disabled={isAnalyzing}
            className="select-button"
          >
            {isAnalyzing ? "⏳ Analyzing..." : "📂 Choose Folder"}
          </button>
          
          {selectedFolder && (
            <div className="selected-folder">
              <strong>Selected:</strong> {selectedFolder}
            </div>
          )}
        </div>
      </section>

      {/* Error Display */}
      {error && (
        <div className="error">
          ❌ {error}
        </div>
      )}

      {/* Analysis Results */}
      {analysis && (
        <section className="analysis-section">
          <h2>📊 Analysis Results</h2>
          
          {/* Summary Stats */}
          <div className="stats-grid">
            <div className="stat-card">
              <div className="stat-number">{analysis.total_files}</div>
              <div className="stat-label">Total Files</div>
            </div>
            <div className="stat-card">
              <div className="stat-number">
                {Object.keys(analysis.categories).filter(cat => analysis.categories[cat].length > 0).length}
              </div>
              <div className="stat-label">Categories</div>
            </div>
          </div>

          {/* Categories */}
          <div className="categories-grid">
            {Object.entries(analysis.categories).map(([category, files]) => (
              files.length > 0 && (
                <div key={category} className="category-card">
                  <div className="category-header">
                    <span className="category-icon">{getCategoryIcon(category)}</span>
                    <span className="category-name">{category}</span>
                  </div>
                  <div className="category-count">{files.length} files</div>
                  
                  {/* File List */}
                  <div className="file-list">
                    {files.slice(0, 3).map((file, index) => (
                      <div key={index} className="file-item">
                        <span className="file-name">{file.name}</span>
                        <span className="file-size">{formatFileSize(file.size)}</span>
                      </div>
                    ))}
                    {files.length > 3 && (
                      <div className="file-item more">
                        +{files.length - 3} more files...
                      </div>
                    )}
                  </div>
                </div>
              )
            ))}
          </div>

          {/* Organization Actions */}
          <div className="actions">
            <button className="action-button primary">
              ✅ Organize Files
            </button>
            <button className="action-button secondary">
              👀 Preview Organization
            </button>
          </div>
        </section>
      )}

      {/* Getting Started */}
      {!selectedFolder && !isAnalyzing && (
        <section className="getting-started">
          <h3>🚀 Get Started</h3>
          <p>
            Select a folder (like Downloads or Desktop) and FileSense will 
            analyze your files and suggest smart organization strategies.
          </p>
          <ul>
            <li>🔒 <strong>100% Private:</strong> All processing happens on your computer</li>
            <li>🧠 <strong>AI-Powered:</strong> Intelligent categorization and organization</li>
            <li>⚡ <strong>Fast:</strong> Local processing for instant results</li>
          </ul>
        </section>
      )}
    </main>
  );
}

export default App;