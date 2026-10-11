import React, { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Folder,
  FileText,
  FileCode,
  FileImage,
  FileArchive,
  File,
  ShieldAlert,
  ShieldCheck,
  Zap,
  Plus,
  X,
  UploadCloud,
  DownloadCloud,
  Search,
  ChevronRight,
  Sun,
  Moon,
  FolderPlus,
  FileCheck,
  AlertTriangle,
} from "lucide-react";
import "./App.css";

interface ArchiveEntry {
  path: string;
  name: string;
  is_dir: boolean;
  size: number;
  compressed_size: number;
  modified: string;
  ratio: string;
  is_encrypted: boolean;
}

interface SecurityReport {
  is_safe: boolean;
  threat_level: "safe" | "warning" | "critical";
  total_entries: number;
  total_uncompressed_bytes: number;
  total_compressed_bytes: number;
  overall_ratio: string;
  findings: string[];
}

interface FilePreview {
  path: string;
  preview_type: "text" | "image" | "binary";
  text_content: string | null;
  base64_content: string | null;
  size: number;
  is_truncated: boolean;
}

interface BenchmarkResult {
  algorithm: string;
  data_size_mb: number;
  compress_time_ms: number;
  decompress_time_ms: number;
  compress_speed_mb_s: number;
  decompress_speed_mb_s: number;
  compressed_size_bytes: number;
  ratio_percent: number;
}

interface Tab {
  id: string;
  archivePath: string | null;
  archiveName: string;
  entries: ArchiveEntry[];
  currentFolder: string;
  selectedEntry: ArchiveEntry | null;
  searchQuery: string;
  securityReport: SecurityReport | null;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function getFileIcon(entry: ArchiveEntry) {
  if (entry.is_dir) {
    return <Folder size={18} color="#38bdf8" />;
  }
  const ext = entry.name.split(".").pop()?.toLowerCase();
  switch (ext) {
    case "png":
    case "jpg":
    case "jpeg":
    case "gif":
    case "webp":
    case "svg":
      return <FileImage size={18} color="#a78bfa" />;
    case "js":
    case "ts":
    case "tsx":
    case "py":
    case "rs":
    case "html":
    case "css":
    case "json":
    case "c":
    case "cpp":
      return <FileCode size={18} color="#34d399" />;
    case "zip":
    case "tar":
    case "gz":
    case "zst":
    case "7z":
    case "rar":
      return <FileArchive size={18} color="#f59e0b" />;
    case "txt":
    case "md":
    case "pdf":
    case "doc":
      return <FileText size={18} color="#94a3b8" />;
    default:
      return <File size={18} color="#64748b" />;
  }
}

export default function App() {
  const [theme, setTheme] = useState<"dark" | "light">("dark");
  const [tabs, setTabs] = useState<Tab[]>([
    {
      id: "tab-1",
      archivePath: null,
      archiveName: "New Tab",
      entries: [],
      currentFolder: "",
      selectedEntry: null,
      searchQuery: "",
      securityReport: null,
    },
  ]);
  const [activeTabId, setActiveTabId] = useState<string>("tab-1");
  const [preview, setPreview] = useState<FilePreview | null>(null);
  const [isLoadingPreview, setIsLoadingPreview] = useState(false);

  // Modals
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [createFormat, setCreateFormat] = useState("tar.zst");
  const [createLevel, setCreateLevel] = useState(3);
  const [sourceFiles, setSourceFiles] = useState<string[]>([]);
  const [isCompressing, setIsCompressing] = useState(false);

  const [showSecurityModal, setShowSecurityModal] = useState(false);
  const [showBenchmarkModal, setShowBenchmarkModal] = useState(false);
  const [benchmarks, setBenchmarks] = useState<BenchmarkResult[]>([]);
  const [isBenchmarking, setIsBenchmarking] = useState(false);

  const activeTab = useMemo(
    () => tabs.find((t) => t.id === activeTabId) || tabs[0],
    [tabs, activeTabId]
  );

  const updateActiveTab = (updater: (prev: Tab) => Tab) => {
    setTabs((prev) => prev.map((t) => (t.id === activeTabId ? updater(t) : t)));
  };

  const openArchiveFromPath = async (archivePath: string) => {
    try {
      const entries = await invoke<ArchiveEntry[]>("list_archive", {
        archivePath,
      });

      const report = await invoke<SecurityReport>("get_security_report", {
        archivePath,
      });

      const fileName = archivePath.split(/[\\/]/).pop() || "Archive";

      updateActiveTab((tab) => ({
        ...tab,
        archivePath,
        archiveName: fileName,
        entries,
        currentFolder: "",
        selectedEntry: null,
        searchQuery: "",
        securityReport: report,
      }));
      setPreview(null);
    } catch (err) {
      alert(`Error loading archive: ${err}`);
    }
  };

  const handleOpenArchive = async () => {
    try {
      const selectedPath = await invoke<string | null>("pick_archive_file");
      if (!selectedPath) return;
      await openArchiveFromPath(selectedPath);
    } catch (err) {
      alert(`Error opening archive: ${err}`);
    }
  };

  useEffect(() => {
    invoke<string | null>("get_initial_archive_path")
      .then((path) => {
        if (path) {
          openArchiveFromPath(path);
        }
      })
      .catch((err) => console.error("CLI startup check:", err));
  }, []);

  const handleSelectEntry = async (entry: ArchiveEntry) => {
    updateActiveTab((t) => ({ ...t, selectedEntry: entry }));
    if (entry.is_dir || !activeTab.archivePath) {
      setPreview(null);
      return;
    }

    setIsLoadingPreview(true);
    try {
      const res = await invoke<FilePreview>("get_file_preview", {
        archivePath: activeTab.archivePath,
        fileInside: entry.path,
        maxBytes: 256 * 1024,
      });
      setPreview(res);
    } catch {
      setPreview(null);
    } finally {
      setIsLoadingPreview(false);
    }
  };

  const handleDoubleClickEntry = (entry: ArchiveEntry) => {
    if (entry.is_dir) {
      const newFolder = entry.path.endsWith("/")
        ? entry.path
        : entry.path + "/";
      updateActiveTab((t) => ({ ...t, currentFolder: newFolder, selectedEntry: null }));
    }
  };

  const handleExtractAll = async () => {
    if (!activeTab.archivePath) return;
    try {
      const dest = await invoke<string | null>("pick_destination_folder");
      if (!dest) return;
      const count = await invoke<number>("extract_archive", {
        archivePath: activeTab.archivePath,
        destination: dest,
        selectedFiles: null,
      });
      alert(`Successfully extracted ${count} files to:\n${dest}`);
    } catch (err) {
      alert(`Extraction failed: ${err}`);
    }
  };

  const handleExtractSelected = async () => {
    if (!activeTab.archivePath || !activeTab.selectedEntry) return;
    try {
      const dest = await invoke<string | null>("pick_destination_folder");
      if (!dest) return;
      await invoke<number>("extract_archive", {
        archivePath: activeTab.archivePath,
        destination: dest,
        selectedFiles: [activeTab.selectedEntry.path],
      });
      alert(`Successfully extracted selected file to:\n${dest}`);
    } catch (err) {
      alert(`Extraction failed: ${err}`);
    }
  };

  const handlePickFilesToCompress = async () => {
    const files = await invoke<string[]>("pick_source_files");
    if (files && files.length > 0) {
      setSourceFiles((prev) => [...prev, ...files]);
    }
  };

  const handlePickFolderToCompress = async () => {
    const folder = await invoke<string | null>("pick_source_folder");
    if (folder) {
      setSourceFiles((prev) => [...prev, folder]);
    }
  };

  const handleStartCompression = async () => {
    if (sourceFiles.length === 0) {
      alert("Please select files or a folder to compress.");
      return;
    }
    const defaultName = "archive";
    const savePath = await invoke<string | null>("pick_save_archive", {
      defaultName,
      format: createFormat,
    });
    if (!savePath) return;

    setIsCompressing(true);
    try {
      const count = await invoke<number>("create_archive", {
        outputPath: savePath,
        format: createFormat,
        sourcePaths: sourceFiles,
        compressionLevel: createLevel,
      });
      setShowCreateModal(false);
      setSourceFiles([]);
      alert(`Success! Packed ${count} items into:\n${savePath}`);
    } catch (err) {
      alert(`Failed to create archive: ${err}`);
    } finally {
      setIsCompressing(false);
    }
  };

  const handleRunBenchmark = async () => {
    setIsBenchmarking(true);
    try {
      const res = await invoke<BenchmarkResult[]>("run_benchmark", {
        dataSizeMb: 25.0,
      });
      setBenchmarks(res);
    } catch (err) {
      alert(`Benchmark error: ${err}`);
    } finally {
      setIsBenchmarking(false);
    }
  };

  const handleCloseTab = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (tabs.length === 1) return;
    const remaining = tabs.filter((t) => t.id !== id);
    setTabs(remaining);
    if (activeTabId === id) {
      setActiveTabId(remaining[0].id);
    }
  };

  const handleNewTab = () => {
    const newId = `tab-${Date.now()}`;
    const newTab: Tab = {
      id: newId,
      archivePath: null,
      archiveName: "New Tab",
      entries: [],
      currentFolder: "",
      selectedEntry: null,
      searchQuery: "",
      securityReport: null,
    };
    setTabs([...tabs, newTab]);
    setActiveTabId(newId);
  };

  // Filter entries in current folder
  const currentEntries = useMemo(() => {
    if (!activeTab.entries) return [];
    const prefix = activeTab.currentFolder;
    const q = activeTab.searchQuery.toLowerCase();

    // If searching, show all matching entries flatly
    if (q) {
      return activeTab.entries.filter((e) =>
        e.path.toLowerCase().includes(q)
      );
    }

    // Otherwise show hierarchical entries inside currentFolder
    const items: ArchiveEntry[] = [];
    const seenFolders = new Set<string>();

    for (const e of activeTab.entries) {
      if (prefix && !e.path.startsWith(prefix)) continue;
      const rel = prefix ? e.path.slice(prefix.length) : e.path;
      if (!rel) continue;

      const parts = rel.split("/").filter(Boolean);
      if (parts.length > 1) {
        // Child folder
        const folderName = parts[0];
        if (!seenFolders.has(folderName)) {
          seenFolders.add(folderName);
          items.push({
            path: prefix ? `${prefix}${folderName}/` : `${folderName}/`,
            name: folderName,
            is_dir: true,
            size: 0,
            compressed_size: 0,
            modified: "",
            ratio: "-",
            is_encrypted: false,
          });
        }
      } else {
        items.push(e);
      }
    }

    // Folders first, then alphabetical
    return items.sort((a, b) => {
      if (a.is_dir === b.is_dir) {
        return a.name.localeCompare(b.name);
      }
      return a.is_dir ? -1 : 1;
    });
  }, [activeTab.entries, activeTab.currentFolder, activeTab.searchQuery]);

  // Breadcrumbs
  const breadcrumbParts = useMemo(() => {
    if (!activeTab.currentFolder) return [];
    return activeTab.currentFolder.split("/").filter(Boolean);
  }, [activeTab.currentFolder]);

  return (
    <div className={`app-container ${theme}`}>
      {/* Tabs Header */}
      <div className="tabs-bar">
        {tabs.map((tab) => (
          <div
            key={tab.id}
            className={`tab-item ${tab.id === activeTabId ? "active" : ""}`}
            onClick={() => setActiveTabId(tab.id)}
          >
            <FileArchive size={14} />
            <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {tab.archiveName}
            </span>
            {tabs.length > 1 && (
              <span
                className="tab-close"
                onClick={(e) => handleCloseTab(tab.id, e)}
              >
                <X size={12} />
              </span>
            )}
          </div>
        ))}
        <button className="tab-add-btn" onClick={handleNewTab} title="New Tab">
          <Plus size={16} />
        </button>

        <button
          className="theme-toggle-btn"
          onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
        >
          {theme === "dark" ? <Sun size={14} /> : <Moon size={14} />}
          {theme === "dark" ? "Light" : "Dark"}
        </button>
      </div>

      {/* Action Toolbar */}
      <div className="toolbar">
        <button className="btn-tool primary" onClick={handleOpenArchive}>
          <UploadCloud size={16} /> Open Archive
        </button>

        <button className="btn-tool" onClick={() => setShowCreateModal(true)}>
          <Plus size={16} /> New Archive
        </button>

        <button
          className="btn-tool"
          disabled={!activeTab.archivePath}
          onClick={handleExtractAll}
        >
          <DownloadCloud size={16} /> Extract All
        </button>

        {activeTab.selectedEntry && !activeTab.selectedEntry.is_dir && (
          <button className="btn-tool" onClick={handleExtractSelected}>
            <DownloadCloud size={16} /> Extract Selected
          </button>
        )}

        <button
          className="btn-tool"
          disabled={!activeTab.securityReport}
          onClick={() => setShowSecurityModal(true)}
          style={{
            color:
              activeTab.securityReport?.threat_level === "critical"
                ? "var(--danger)"
                : activeTab.securityReport?.threat_level === "warning"
                ? "var(--warning)"
                : "var(--success)",
          }}
        >
          {activeTab.securityReport?.is_safe ? (
            <ShieldCheck size={16} />
          ) : (
            <ShieldAlert size={16} />
          )}
          Security Scan
        </button>

        <button
          className="btn-tool"
          onClick={() => {
            setShowBenchmarkModal(true);
            if (benchmarks.length === 0) handleRunBenchmark();
          }}
        >
          <Zap size={16} /> Speed Benchmark
        </button>
      </div>

      {/* Navigation & Search Bar */}
      {activeTab.archivePath && (
        <div className="nav-bar">
          <div className="breadcrumbs">
            <span
              className={`crumb-item ${breadcrumbParts.length === 0 ? "active" : ""}`}
              onClick={() => updateActiveTab((t) => ({ ...t, currentFolder: "" }))}
            >
              root
            </span>
            {breadcrumbParts.map((part, idx) => {
              const pathUpTo = breadcrumbParts.slice(0, idx + 1).join("/") + "/";
              const isLast = idx === breadcrumbParts.length - 1;
              return (
                <React.Fragment key={idx}>
                  <ChevronRight size={14} />
                  <span
                    className={`crumb-item ${isLast ? "active" : ""}`}
                    onClick={() => updateActiveTab((t) => ({ ...t, currentFolder: pathUpTo }))}
                  >
                    {part}
                  </span>
                </React.Fragment>
              );
            })}
          </div>

          <div className="search-box">
            <Search size={14} color="var(--text-dim)" />
            <input
              type="text"
              className="search-input"
              placeholder="Search in archive..."
              value={activeTab.searchQuery}
              onChange={(e) => updateActiveTab((t) => ({ ...t, searchQuery: e.target.value }))}
            />
          </div>
        </div>
      )}

      {/* Main Content Area */}
      <div className="main-view">
        {activeTab.archivePath ? (
          <>
            {/* File Table */}
            <div className="table-container">
              <table className="file-table">
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Size</th>
                    <th>Packed</th>
                    <th>Ratio</th>
                    <th>Modified</th>
                  </tr>
                </thead>
                <tbody>
                  {currentEntries.map((entry, idx) => {
                    const isSelected = activeTab.selectedEntry?.path === entry.path;
                    return (
                      <tr
                        key={idx}
                        className={`file-row ${isSelected ? "selected" : ""}`}
                        onClick={() => handleSelectEntry(entry)}
                        onDoubleClick={() => handleDoubleClickEntry(entry)}
                      >
                        <td className="file-name-cell">
                          <span className="file-icon">{getFileIcon(entry)}</span>
                          <span>{entry.name}</span>
                        </td>
                        <td>{entry.is_dir ? "-" : formatBytes(entry.size)}</td>
                        <td>{entry.is_dir ? "-" : formatBytes(entry.compressed_size)}</td>
                        <td>{entry.ratio}</td>
                        <td>{entry.modified || "-"}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>

            {/* Quick-Look Inspector Panel */}
            {activeTab.selectedEntry && !activeTab.selectedEntry.is_dir && (
              <div className="inspector-panel">
                <div className="inspector-header">
                  <div className="inspector-title">
                    {getFileIcon(activeTab.selectedEntry)}
                    {activeTab.selectedEntry.name}
                  </div>
                  <button
                    className="tab-close"
                    onClick={() => updateActiveTab((t) => ({ ...t, selectedEntry: null }))}
                  >
                    <X size={14} />
                  </button>
                </div>

                <div className="inspector-body">
                  <div className="inspector-stat-grid">
                    <div className="stat-card">
                      <div className="stat-label">Original Size</div>
                      <div className="stat-val">{formatBytes(activeTab.selectedEntry.size)}</div>
                    </div>
                    <div className="stat-card">
                      <div className="stat-label">Compressed</div>
                      <div className="stat-val">{formatBytes(activeTab.selectedEntry.compressed_size)}</div>
                    </div>
                  </div>

                  {/* Live In-Memory Preview */}
                  <div className="preview-container">
                    <div className="preview-header">
                      <span>Quick-Look Preview</span>
                      {preview?.is_truncated && (
                        <span style={{ color: "var(--warning)" }}>Truncated (256 KB)</span>
                      )}
                    </div>

                    {isLoadingPreview ? (
                      <div style={{ padding: 20, textAlign: "center", color: "var(--text-dim)" }}>
                        Loading preview...
                      </div>
                    ) : preview?.preview_type === "image" && preview.base64_content ? (
                      <img
                        src={preview.base64_content}
                        alt="Preview"
                        className="preview-image"
                      />
                    ) : preview?.text_content ? (
                      <pre className="preview-content">{preview.text_content}</pre>
                    ) : (
                      <div style={{ padding: 20, textAlign: "center", color: "var(--text-dim)" }}>
                        Select a text file or image for instant in-memory preview without extracting to disk.
                      </div>
                    )}
                  </div>

                  <button className="btn-tool primary" onClick={handleExtractSelected}>
                    <DownloadCloud size={14} /> Extract this file
                  </button>
                </div>
              </div>
            )}
          </>
        ) : (
          /* Empty / Welcome State */
          <div className="empty-state">
            <FileArchive size={64} className="empty-icon" />
            <h2 className="empty-title">GenesisArchive Pro</h2>
            <p className="empty-desc">
              Open an archive to explore files, preview images and code instantly without unpacking, or create a high-speed Zstandard archive.
            </p>
            <div style={{ display: "flex", gap: 12 }}>
              <button className="btn-tool primary" onClick={handleOpenArchive}>
                <UploadCloud size={16} /> Open Archive (.rar, .zip, .7z, .tar.zst)
              </button>
              <button className="btn-tool" onClick={() => setShowCreateModal(true)}>
                <Plus size={16} /> Create New Archive
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Status Bar */}
      <div className="status-bar">
        <div>
          {activeTab.archivePath ? (
            <span>
              {activeTab.entries.length} items ({formatBytes(
                activeTab.entries.reduce((acc, e) => acc + e.size, 0)
              )})
            </span>
          ) : (
            <span>Ready</span>
          )}
        </div>
        <div className="status-stat">
          <span className="speed-badge">⚡ Zstandard 3+ GB/s Engine</span>
          <span>Linux + Windows Free Edition</span>
        </div>
      </div>

      {/* Create Archive Modal */}
      {showCreateModal && (
        <div className="modal-backdrop" onClick={() => setShowCreateModal(false)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <div className="modal-title">
                <Plus size={18} color="var(--accent)" /> Create New Archive
              </div>
              <button className="tab-close" onClick={() => setShowCreateModal(false)}>
                <X size={16} />
              </button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <div className="form-label">Select Target Format</div>
                <div className="format-grid">
                  <div
                    className={`format-pill ${createFormat === "tar.zst" ? "active" : ""}`}
                    onClick={() => setCreateFormat("tar.zst")}
                  >
                    <div className="format-name">
                      <span>.tar.zst</span>
                      <span className="speed-badge">Ultra Fast</span>
                    </div>
                    <div className="format-desc">Meta Zstandard. 3-5x faster than RAR/ZIP.</div>
                  </div>

                  <div
                    className={`format-pill ${createFormat === "zip" ? "active" : ""}`}
                    onClick={() => setCreateFormat("zip")}
                  >
                    <div className="format-name">.zip</div>
                    <div className="format-desc">Universal compatibility across all OSes.</div>
                  </div>

                  <div
                    className={`format-pill ${createFormat === "7z" ? "active" : ""}`}
                    onClick={() => setCreateFormat("7z")}
                  >
                    <div className="format-name">.7z</div>
                    <div className="format-desc">High compression ratio LZMA/7-Zip format.</div>
                  </div>

                  <div
                    className={`format-pill ${createFormat === "tar.gz" ? "active" : ""}`}
                    onClick={() => setCreateFormat("tar.gz")}
                  >
                    <div className="format-name">.tar.gz</div>
                    <div className="format-desc">Standard Linux POSIX compressed archive.</div>
                  </div>
                </div>
              </div>

              <div className="form-group">
                <div className="form-label">Compression Level (1 = Fastest, 9 = Maximum)</div>
                <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                  <input
                    type="range"
                    min={1}
                    max={9}
                    value={createLevel}
                    onChange={(e) => setCreateLevel(Number(e.target.value))}
                    style={{ flex: 1 }}
                  />
                  <span style={{ fontWeight: 600, width: 30 }}>{createLevel}</span>
                </div>
              </div>

              <div className="form-group">
                <div className="form-label">Files & Folders to Compress ({sourceFiles.length})</div>
                <div style={{ display: "flex", gap: 8 }}>
                  <button className="btn-tool" onClick={handlePickFilesToCompress}>
                    <Plus size={14} /> Add Files
                  </button>
                  <button className="btn-tool" onClick={handlePickFolderToCompress}>
                    <FolderPlus size={14} /> Add Folder
                  </button>
                </div>
                {sourceFiles.length > 0 && (
                  <div
                    style={{
                      maxHeight: 120,
                      overflowY: "auto",
                      backgroundColor: "var(--bg-app)",
                      border: "1px solid var(--border-color)",
                      borderRadius: "var(--radius)",
                      padding: 8,
                      fontSize: 12,
                      display: "flex",
                      flexDirection: "column",
                      gap: 4,
                    }}
                  >
                    {sourceFiles.map((file, i) => (
                      <div
                        key={i}
                        style={{
                          display: "flex",
                          justifyContent: "space-between",
                          alignItems: "center",
                        }}
                      >
                        <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                          {file}
                        </span>
                        <X
                          size={12}
                          style={{ cursor: "pointer", opacity: 0.7 }}
                          onClick={() => setSourceFiles(sourceFiles.filter((_, idx) => idx !== i))}
                        />
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
            <div className="modal-footer">
              <button className="btn-tool" onClick={() => setShowCreateModal(false)}>
                Cancel
              </button>
              <button
                className="btn-tool primary"
                disabled={isCompressing || sourceFiles.length === 0}
                onClick={handleStartCompression}
              >
                {isCompressing ? "Compressing..." : "Choose Destination & Compress"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Security Audit Modal */}
      {showSecurityModal && activeTab.securityReport && (
        <div className="modal-backdrop" onClick={() => setShowSecurityModal(false)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <div className="modal-title">
                <ShieldCheck size={18} color="var(--accent)" /> Archive Security Audit
              </div>
              <button className="tab-close" onClick={() => setShowSecurityModal(false)}>
                <X size={16} />
              </button>
            </div>
            <div className="modal-body">
              <div className="inspector-stat-grid">
                <div className="stat-card">
                  <div className="stat-label">Threat Status</div>
                  <div
                    className="stat-val"
                    style={{
                      color:
                        activeTab.securityReport.threat_level === "safe"
                          ? "var(--success)"
                          : activeTab.securityReport.threat_level === "warning"
                          ? "var(--warning)"
                          : "var(--danger)",
                      textTransform: "uppercase",
                    }}
                  >
                    {activeTab.securityReport.threat_level}
                  </div>
                </div>
                <div className="stat-card">
                  <div className="stat-label">Compression Ratio</div>
                  <div className="stat-val">{activeTab.securityReport.overall_ratio}</div>
                </div>
              </div>

              <div className="form-group">
                <div className="form-label">Audit Findings & Inspection Checklist</div>
                <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                  {activeTab.securityReport.findings.map((finding, idx) => (
                    <div
                      key={idx}
                      style={{
                        padding: 10,
                        borderRadius: "var(--radius)",
                        backgroundColor: "var(--bg-app)",
                        border: "1px solid var(--border-color)",
                        fontSize: 13,
                        display: "flex",
                        alignItems: "center",
                        gap: 8,
                      }}
                    >
                      {finding.includes("⚠️") || finding.includes("🚨") ? (
                        <AlertTriangle size={16} color="var(--warning)" />
                      ) : (
                        <FileCheck size={16} color="var(--success)" />
                      )}
                      <span>{finding}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
            <div className="modal-footer">
              <button className="btn-tool primary" onClick={() => setShowSecurityModal(false)}>
                Dismiss
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Speed Benchmark Modal */}
      {showBenchmarkModal && (
        <div className="modal-backdrop" onClick={() => setShowBenchmarkModal(false)}>
          <div className="modal-card" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <div className="modal-title">
                <Zap size={18} color="var(--accent)" /> Compression Speed Benchmark
              </div>
              <button className="tab-close" onClick={() => setShowBenchmarkModal(false)}>
                <X size={16} />
              </button>
            </div>
            <div className="modal-body">
              <p style={{ fontSize: 13, color: "var(--text-muted)" }}>
                Testing live CPU compression and decompression throughput using modern streaming buffers on 25 MB sample payload.
              </p>

              {isBenchmarking ? (
                <div style={{ padding: 30, textAlign: "center", color: "var(--accent)" }}>
                  Running multi-algorithm benchmark...
                </div>
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                  {benchmarks.map((res, idx) => (
                    <div key={idx} className="benchmark-card">
                      <div className="benchmark-badge">{res.algorithm}</div>
                      <div className="benchmark-metric">
                        <span>Compression Speed:</span>
                        <span style={{ color: "var(--accent)" }}>{res.compress_speed_mb_s} MB/s ({res.compress_time_ms} ms)</span>
                      </div>
                      <div className="benchmark-metric">
                        <span>Decompression Speed:</span>
                        <span style={{ color: "var(--success)" }}>{res.decompress_speed_mb_s} MB/s ({res.decompress_time_ms} ms)</span>
                      </div>
                      <div className="benchmark-metric">
                        <span>Payload Size / Ratio:</span>
                        <span>{formatBytes(res.compressed_size_bytes)} ({res.ratio_percent.toFixed(1)}%)</span>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
            <div className="modal-footer">
              <button className="btn-tool" onClick={handleRunBenchmark} disabled={isBenchmarking}>
                Re-Run Test
              </button>
              <button className="btn-tool primary" onClick={() => setShowBenchmarkModal(false)}>
                Done
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
