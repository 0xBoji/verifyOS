"use client";

import React, { useEffect, useMemo, useRef, useState } from "react";
import { FaGithub, FaChevronRight, FaChevronDown } from "react-icons/fa";
import { SiRust } from "react-icons/si";
import { VscVscode } from "react-icons/vsc";
import { FiAlertCircle, FiAlertTriangle, FiFolder, FiTarget, FiActivity, FiZoomIn, FiZoomOut, FiMaximize, FiX, FiShare2, FiPackage } from "react-icons/fi";
import JSZip from "jszip";

interface Finding {
  rule_id: string;
  rule_name: string;
  category: string;
  severity: string;
  status: string;
  message: string;
  recommendation?: string;
  evidence?: string | Record<string, unknown>;
  duration_ms?: number;
  target: string;
}

const ASTBranch = ({ children, count }: { children: React.ReactNode; count: number }) => {
  if (count <= 0) return <div className="ast-level">{children}</div>;
  
  return (
    <div className="ast-branch">
      <div className="ast-trunk" />
      {count > 1 && (
        <div className="ast-bar" style={{ 
          width: `calc(100% - ${100 / count}%)`,
          margin: '0 auto'
        }} />
      )}
      <div className="ast-level">
        {React.Children.map(children, (child) => (
          <div className="ast-leg-wrapper">
             <div className="ast-leg" />
             {child}
          </div>
        ))}
      </div>
    </div>
  );
};

const ASTViewer = ({ data, astFocus }: { data: any; astFocus: string | null }) => {
  const targets = (data?.report?.scanned_targets as string[]) ?? [];
  const findings = (data?.report?.results as Finding[]) ?? [];
  const [selectedNode, setSelectedNode] = useState<Finding | null>(null);

  // AST Pan & Zoom State
  const [zoom, setZoom] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const dragStart = useRef({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  // Center node when astFocus changes
  useEffect(() => {
    if (astFocus && containerRef.current) {
      setTimeout(() => {
        const el = document.getElementById(`ast-node-${astFocus}`);
        if (el && containerRef.current) {
          const containerRect = containerRef.current.getBoundingClientRect();
          const nodeRect = el.getBoundingClientRect();
          
          const dx = (containerRect.width / 2) - (nodeRect.left - containerRect.left + nodeRect.width / 2);
          const dy = (containerRect.height / 2) - (nodeRect.top - containerRect.top + nodeRect.height / 2);
          
          setOffset(prev => ({
            x: prev.x + dx,
            y: prev.y + dy
          }));
        }
      }, 100);
    }
  }, [astFocus, data]);

  const renderFindingNode = (f: Finding, idx: number) => (
    <div 
      key={`${f.rule_id}-${idx}`} 
      className={`ast-node ${f.severity === 'Error' ? 'ast-node--error' : 'ast-node--warning'} ${astFocus === f.rule_id || selectedNode?.rule_id === f.rule_id ? 'is-focused' : ''}`} 
      id={`ast-node-${f.rule_id}`}
      onClick={(e) => {
        e.stopPropagation();
        setSelectedNode(f);
      }}
      style={{ cursor: 'pointer' }}
    >
      <div className="ast-node-icon"><FiAlertCircle /></div>
      <span className="ast-node-label">{f.rule_name}</span>
      <span className="ast-node-sublabel">{f.rule_id}</span>
    </div>
  );

  const renderTargetBranch = (target: string) => {
    const targetFindings = findings.filter(f => f.target === target && (f.status === 'Fail' || f.status === 'Error'));
    const hasError = targetFindings.some(f => f.severity === 'Error');
    const hasWarning = targetFindings.some(f => f.severity === 'Warning');

    return (
      <div key={target} className="ast-node-group">
        <div className={`ast-node ${hasError ? 'ast-node--error' : hasWarning ? 'ast-node--warning' : ''}`}>
          <div className="ast-node-icon"><FiTarget /></div>
          <span className="ast-node-label">{target}</span>
          <span className="ast-node-sublabel">Scan Target</span>
        </div>
        
        <ASTBranch count={targetFindings.length}>
          {targetFindings.map(renderFindingNode)}
        </ASTBranch>
      </div>
    );
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('.ast-node')) return;
    setIsDragging(true);
    dragStart.current = { x: e.clientX - offset.x, y: e.clientY - offset.y };
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDragging) return;
    setOffset({
      x: e.clientX - dragStart.current.x,
      y: e.clientY - dragStart.current.y
    });
  };

  const handleMouseUp = () => setIsDragging(false);

  const handleWheel = (e: React.WheelEvent) => {
    if (e.ctrlKey || e.metaKey) {
      const delta = e.deltaY > 0 ? 0.9 : 1.1;
      setZoom(prev => Math.min(Math.max(prev * delta, 0.1), 3));
    } else {
      setOffset(prev => ({
        x: prev.x - e.deltaX,
        y: prev.y - e.deltaY
      }));
    }
  };

  return (
    <div className="ast-viewer-layout">
      <div 
        ref={containerRef}
        className="ast-container"
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        onWheel={handleWheel}
        style={{ cursor: isDragging ? 'grabbing' : 'grab' }}
      >
        <div 
          className="ast-tree-wrapper"
          style={{ 
            transform: `translate(${offset.x}px, ${offset.y}px) scale(${zoom})`,
            transformOrigin: 'center center',
            transition: isDragging ? 'none' : 'transform 0.1s ease-out'
          }}
        >
          {/* Root Node */}
          <div className="ast-branch">
            <div className="ast-node ast-node--root">
              <div className="ast-node-icon"><FiPackage /></div>
              <span className="ast-node-label">Application Bundle</span>
              <span className="ast-node-sublabel">Root Container</span>
            </div>
            
            <ASTBranch count={targets.length}>
              {targets.length > 0 ? targets.map(renderTargetBranch) : [renderTargetBranch("Default Target")]}
            </ASTBranch>
          </div>
        </div>
      </div>
      
      <div className="ast-controls">
        <button className="pill-chip" onClick={() => setZoom(z => Math.min(z + 0.1, 3))}><FiZoomIn /></button>
        <button className="pill-chip" onClick={() => setZoom(z => Math.max(z - 0.1, 0.1))}><FiZoomOut /></button>
        <button className="pill-chip" onClick={() => { setZoom(1); setOffset({ x: 0, y: 0 }); }}><FiMaximize /></button>
        <div className="zoom-label">{Math.round(zoom * 100)}%</div>
      </div>

      {selectedNode && (
        <div className="ast-details-panel">
          <div className="ast-details-header">
            <div className={`pill-chip pill-chip--${String(selectedNode.severity).toLowerCase()}`}>
              {selectedNode.severity}
            </div>
            <h4>{selectedNode.rule_name}</h4>
            <button className="ast-close-button" onClick={() => setSelectedNode(null)} aria-label="Close details">
              <FiX />
            </button>
          </div>
          <div className="ast-details-body">
            <div className="ast-details-section">
              <label>Message</label>
              <p>{selectedNode.message}</p>
            </div>
            {selectedNode.evidence && (
              <div className="ast-details-section">
                <label>Evidence</label>
                <pre>{typeof selectedNode.evidence === 'string' ? selectedNode.evidence : JSON.stringify(selectedNode.evidence, null, 2)}</pre>
              </div>
            )}
            {selectedNode.recommendation && (
              <div className="ast-details-section">
                <label>Recommendation</label>
                <p>{selectedNode.recommendation}</p>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

const ASTModal = ({ isOpen, onClose, data, astFocus }: { isOpen: boolean; onClose: () => void; data: any; astFocus: string | null }) => {
  if (!isOpen) return null;

  return (
    <div className="ast-modal-overlay">
      <div className="ast-modal-content">
        <div className="ast-modal-header">
          <h3>Diagnostic AST</h3>
          <button className="ghost-button" onClick={onClose}>Close</button>
        </div>
        <ASTViewer data={data} astFocus={astFocus} />
      </div>
    </div>
  );
};

const AlertModal = ({ isOpen, title, message, actions }: { 
  isOpen: boolean; 
  title: string; 
  message: string; 
  actions: { label: string; onClick: () => void; bold?: boolean; danger?: boolean; }[] 
}) => {
  if (!isOpen) return null;

  return (
    <div className="alert-overlay">
      <div className="alert-content">
        <div className="alert-body">
          <div className="alert-title">{title}</div>
          <div className="alert-message">{message}</div>
        </div>
        <div className={`alert-actions ${actions.length > 2 ? 'alert-actions--vertical' : ''}`}>
          {actions.map((action, i) => (
            <button 
              key={i} 
              className={`alert-action ${action.bold ? 'alert-action--bold' : ''} ${action.danger ? 'alert-action--danger' : ''}`}
              onClick={action.onClick}
            >
              {action.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};

const Toast = ({ message, onClear }: { message: string | null; onClear: () => void }) => {
  useEffect(() => {
    if (message) {
      const timer = setTimeout(onClear, 3000);
      return () => clearTimeout(timer);
    }
  }, [message, onClear]);

  if (!message) return null;

  return (
    <div className="toast-container">
      <div className="toast">
        <FiActivity />
        <span>{message}</span>
      </div>
    </div>
  );
};

interface DiscoveryTarget {
  path: string;
  name: string;
  type: "app" | "project" | "workspace" | "ipa";
}

export default function Home() {
  const fileRef = useRef<HTMLInputElement>(null);
  const folderRef = useRef<HTMLInputElement>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [result, setResult] = useState<Record<string, unknown> | null>(null);
  const [rawResult, setRawResult] = useState<string | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());
  const [severityFilter, setSeverityFilter] = useState<string | null>(null);
  const [bundleType, setBundleType] = useState<"ipa" | "project">("ipa");
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [discoveredTargets, setDiscoveredTargets] = useState<DiscoveryTarget[]>([]);
  const [pendingFiles, setPendingFiles] = useState<File[]>([]);

  // Custom Alert/Toast state
  const [alertConfig, setAlertConfig] = useState<{ 
    title: string; 
    message: string; 
    actions: { label: string; onClick: () => void; bold?: boolean; danger?: boolean; }[] 
  } | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<"list" | "ast">("list");
  const [astFocus, setAstFocus] = useState<string | null>(null);
  const [isASTModalOpen, setIsASTModalOpen] = useState(false);

  useEffect(() => {
    if (isASTModalOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = 'unset';
    }
    return () => {
      document.body.style.overflow = 'unset';
    };
  }, [isASTModalOpen]);

  const backendBaseUrl =
    process.env.NEXT_PUBLIC_BACKEND_URL ?? "http://127.0.0.1:7070";


  const examplePayload = {
    report: {
      ruleset_version: "0.8.2",
      generated_at_unix: 1710604800,
      total_duration_ms: 1240,
      scanned_targets: ["ProductionApp.app"],
      results: [
        {
          rule_id: "RULE_XCODE_26_MANDATE",
          rule_name: "Xcode 26 Mandate",
          category: "Compliance",
          severity: "Error",
          status: "Fail",
          message: "App built with Xcode 15.4, iOS 17.5 SDK",
          recommendation: "Upgrade to Xcode 26 for 2026 App Store requirements.",
          target: "ProductionApp.app"
        },
        {
          rule_id: "RULE_PRIVACY_MANIFEST",
          rule_name: "Privacy Manifest Missing",
          category: "Privacy",
          severity: "Error",
          status: "Fail",
          message: "PrivacyInfo.xcprivacy not found",
          recommendation: "Add a PrivacyInfo.xcprivacy file to your app bundle.",
          target: "ProductionApp.app"
        },
        {
          rule_id: "RULE_ENTITLEMENTS_MISMATCH",
          rule_name: "Entitlements Mismatch",
          category: "Entitlements",
          severity: "Error",
          status: "Fail",
          message: "get-task-allow=true found in entitlements",
          recommendation: "Remove get-task-allow for production builds.",
          target: "ProductionApp.app"
        },
        {
          rule_id: "RULE_APP_ICON_MISSING",
          rule_name: "App Icon Missing",
          category: "Metadata",
          severity: "Error",
          status: "Fail",
          message: "Missing 1024px App Store icon",
          recommendation: "Ensure 1024x1024 icon is in Assets.car.",
          target: "ProductionApp.app"
        },
        {
          rule_id: "RULE_MISSING_CFBUNDLEVERSION",
          rule_name: "Missing Build Version",
          category: "Metadata",
          severity: "Error",
          status: "Fail",
          message: "CFBundleVersion is empty",
          recommendation: "Set a unique build number for every submission.",
          target: "ProductionApp.app"
        },
        {
          rule_id: "RULE_ATS_AUDIT",
          rule_name: "ATS Audit",
          category: "Security",
          severity: "Warning",
          status: "Fail",
          message: "NSAllowsArbitraryLoads enabled",
          recommendation: "Scope ATS exceptions to specific domains.",
          target: "ProductionApp.app"
        },
        {
          rule_id: "RULE_EXPORT_COMPLIANCE",
          rule_name: "Export Declaration",
          category: "Metadata",
          severity: "Warning",
          status: "Fail",
          message: "ITSAppUsesNonExemptEncryption not set",
          recommendation: "Update Info.plist to specify encryption usage.",
          target: "ProductionApp.app"
        },
        {
          rule_id: "RULE_PRIVATE_API",
          rule_name: "Private API Usage",
          category: "Private API",
          severity: "Warning",
          status: "Fail",
          message: "_GSSystemAdditions usage detected",
          recommendation: "Replace with public API to avoid rejection.",
          target: "ProductionApp.app"
        }
      ],
    },
  };


  const handleChooseFile = () => {
    fileRef.current?.click();
  };

  const handleChooseFolder = () => {
    folderRef.current?.click();
  };

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0] ?? null;
    setSelectedFile(file);
    if (!file && event.target.files?.length === 0) {
      setToastMessage("No file selected");
    } else {
      setToastMessage(file ? `Selected ${file.name}` : "No file selected");
    }
    setResult(null);
    setRawResult(null);
  };

  const handleFolderChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;
    if (!files || files.length === 0) return;

    setIsDiscovering(true);
    setToastMessage("Analyzing folder...");
    const allFiles = Array.from(files);
    setPendingFiles(allFiles);

    // Discovery logic
    const targets: DiscoveryTarget[] = [];
    for (const f of allFiles) {
      const path = f.webkitRelativePath;
      if (path.includes('node_modules') || path.includes('.git') || path.includes('DerivedData') || path.includes('build/')) continue;

      const parts = path.split('/');
      for (const part of parts) {
        const ext = part.split('.').pop();
        if (ext === 'xcodeproj' || ext === 'xcworkspace' || ext === 'app') {
          const idx = parts.indexOf(part);
          const fullPath = parts.slice(0, idx + 1).join('/');
          if (!targets.find(t => t.path === fullPath)) {
            targets.push({
              path: fullPath,
              name: part,
              type: ext === 'xcodeproj' ? 'project' : ext === 'xcworkspace' ? 'workspace' : 'app'
            });
          }
        }
      }
      if (f.name.endsWith('.ipa')) {
        targets.push({ path: path, name: f.name, type: 'ipa' });
      }
    }

    if (targets.length > 1) {
      setDiscoveredTargets(targets);
      setAlertConfig({
        title: "Multiple targets found",
        message: "We found multiple scannable items in your folder. Please select one to proceed.",
        actions: targets.map(t => ({
          label: `${t.name} (${t.type})`,
          onClick: () => {
            setAlertConfig(null);
            void bundleAndSelect(allFiles, t);
          }
        })).concat([{
          label: "Scan entire folder",
          onClick: () => {
            setAlertConfig(null);
            void bundleAndSelect(allFiles, null);
          }
        }])
      });
      setToastMessage(`Found ${targets.length} potential targets.`);
      setIsDiscovering(false);
      return;
    }

    // If only one found, or none (scan root)
    const target = targets[0] || null;
    await bundleAndSelect(allFiles, target);
  };

  const bundleAndSelect = async (allFiles: File[], target: DiscoveryTarget | null) => {
    setIsUploading(true);
    setDiscoveredTargets([]);
    setIsDiscovering(false);

    const rootFolderName = allFiles[0].webkitRelativePath.split('/')[0];
    const targetName = target ? target.name : rootFolderName;
    setToastMessage(`Bundling ${targetName}...`);

    try {
      const zip = new JSZip();
      for (const file of allFiles) {
        const path = file.webkitRelativePath;

        // Smart Filtering
        if (
          path.includes('node_modules/') ||
          path.includes('.git/') ||
          path.includes('.DS_Store') ||
          path.includes('DerivedData/') ||
          path.includes('build/')
        ) {
          continue;
        }

        // Scope to target if selected
        if (target && !path.startsWith(target.path)) {
          continue;
        }

        zip.file(path, file);
      }

      const content = await zip.generateAsync({ type: "blob" });
      const zippedFile = new File([content], `${targetName}.zip`, {
        type: "application/zip",
      });

      setSelectedFile(zippedFile);
      setPendingFiles([]);
      setToastMessage(`Ready: ${targetName}.zip`);
      setResult(null);
      setRawResult(null);
    } catch {
      setToastMessage("Failed to bundle folder");
    } finally {
      setIsUploading(false);
    }
  };


  const handleUpload = async () => {
    if (!selectedFile || isUploading) {
      return;
    }

    setIsUploading(true);
    setToastMessage("Scanning...");
    setResult(null);

    try {
      const form = new FormData();
      form.append("bundle", selectedFile);
      form.append("profile", "full");

      const response = await fetch(`${backendBaseUrl}/api/v1/scan`, {
        method: "POST",
        body: form,
      });

      const rawText = await response.text();
      let payload: unknown = rawText;
      if (rawText) {
        try {
          payload = JSON.parse(rawText);
        } catch {
          payload = rawText;
        }
      }

      if (!response.ok) {
        const message =
          typeof payload === "object" && payload !== null && "error" in payload
            ? String((payload as { error?: string }).error)
            : `Scan failed (${response.status})`;
        setAlertConfig({
          title: "Scan error",
          message: message,
          actions: [{ label: "OK", onClick: () => setAlertConfig(null), bold: true }]
        });
        setToastMessage("Scan failed");
        setResult(null);
        setRawResult(rawText || null);
        return;
      }

      setToastMessage("Scan complete");
      if (payload && typeof payload === "object") {
        setResult(payload as Record<string, unknown>);
        setRawResult(JSON.stringify(payload, null, 2));
      } else {
        setResult(null);
        setRawResult(rawText || null);
      }
    } catch {
      setAlertConfig({
        title: "Backend Unreachable",
        message: "Failed to reach backend. Please ensure the server is running on port 7070.",
        actions: [{ label: "Close", onClick: () => setAlertConfig(null) }]
      });
      setToastMessage("Connection failed");
      setResult(null);
      setRawResult(null);
    } finally {
      setIsUploading(false);
    }
  };

  const handleDownloadBundle = async () => {
    if (!selectedFile || isDownloading) {
      return;
    }

    setIsDownloading(true);
    setToastMessage("Preparing agent bundle...");

    try {
      const form = new FormData();
      form.append("bundle", selectedFile);
      form.append("profile", "full");

      const response = await fetch(`${backendBaseUrl}/api/v1/handoff`, {
        method: "POST",
        body: form,
      });

      if (!response.ok) {
        const text = await response.text();
        setToastMessage(text || `Bundle failed (${response.status})`);
        return;
      }

      const blob = await response.blob();
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = "verifyos-handoff.zip";
      link.click();
      URL.revokeObjectURL(url);
      setToastMessage("Agent bundle downloaded");
    } catch {
      setToastMessage("Failed to download agent bundle");
    } finally {
      setIsDownloading(false);
    }
  };

  const handleExampleReport = () => {
    setToastMessage("Loaded example report");
    setSelectedFile(null);
    setResult(examplePayload as Record<string, unknown>);
    setRawResult(JSON.stringify(examplePayload, null, 2));
  };

  const summary = useMemo(() => {
    const report = result?.report as
      | {
          results?: Array<Record<string, unknown>>;
          total_duration_ms?: number;
        }
      | undefined;
    const results = report?.results ?? [];
    const failures = (results as unknown as Finding[]).filter((item) => {
      const status = item.status as string | undefined;
      return status === "Fail" || status === "Error";
    });
    const errorCount = failures.filter((item) => item.severity === "Error").length;
    const warningCount = failures.filter((item) => item.severity === "Warning").length;
    const duration =
      typeof report?.total_duration_ms === "number"
        ? `${report.total_duration_ms}ms`
        : null;

    const byCategory = failures.reduce<Record<string, number>>((acc, item) => {
      const category = String(item.category ?? "Other");
      acc[category] = (acc[category] ?? 0) + 1;
      return acc;
    }, {});

    const bySeverity = failures.reduce<Record<string, number>>((acc, item) => {
      const severity = String(item.severity ?? "Unknown");
      acc[severity] = (acc[severity] ?? 0) + 1;
      return acc;
    }, {});

    const findingsByCategory = failures.reduce<Record<string, Finding[]>>((acc, item) => {
      const category = String(item.category ?? "Other");
      if (!acc[category]) acc[category] = [];
      acc[category].push(item);
      return acc;
    }, {});

    return {
      results,
      failures,
      errorCount,
      warningCount,
      duration,
      byCategory,
      bySeverity,
      findingsByCategory,
    };
  }, [result]);

  const toggleCategory = (category: string) => {
    const next = new Set(expandedCategories);
    if (next.has(category)) {
      next.delete(category);
    } else {
      next.add(category);
    }
    setExpandedCategories(next);
  };

  const expandAll = (categories: string[]) => {
    setExpandedCategories(new Set(categories));
  };

  const collapseAll = () => {
    setExpandedCategories(new Set());
  };

  const [selectedNode, setSelectedNode] = useState<Finding | null>(null);



  return (
    <div className="page">
      <div className="page-glow page-glow--left" />
      <div className="page-glow page-glow--right" />

      <header className="nav">
        <div className="logo">
          <span className="logo-mark" aria-hidden="true">
            <img src="/logo/verifyOS_web_128x.png" alt="" />
          </span>
          <div>
            <div className="logo-title">verifyOS</div>
            <div className="logo-subtitle">App Store review confidence</div>
          </div>
        </div>
        <div className="nav-actions">
          <a
            className="ghost-button"
            href="https://github.com/0xBoji/verifyOS#readme"
            target="_blank"
            rel="noreferrer"
          >
            Docs
          </a>
        </div>
      </header>

      <main className="shell">
        <section className="hero">
          <div className="hero-copy">
            <div className="badge">iOS-friendly diagnostics</div>
            <h1>
              Ship App Store reviews with{" "}
              <span className="accent">zero surprises</span>.
            </h1>
            <p>
              Scan <span className="pill">.ipa</span>,{" "}
              <span className="pill">.app</span>,{" "}
              <span className="pill">.xcodeproj</span>, or{" "}
              <span className="pill">.xcworkspace</span> (zip) and get a clean,
              structured report for privacy, entitlements, signing, metadata,
              and more. Designed for AI agents and human reviewers.
            </p>
          </div>
        </section>

        <section className="steps">
          <div className="step">
            <div className="step-number">1</div>
            <div>
              <h4>Upload bundle</h4>
              <p>Scan locally with zero external uploads or cloud storage.</p>
            </div>
          </div>
          <div className="step">
            <div className="step-number">2</div>
            <div>
              <h4>Review findings</h4>
              <p>
                Clear severity, evidence, and recommendations for each rule.
              </p>
            </div>
          </div>
          <div className="step">
            <div className="step-number">3</div>
            <div>
              <h4>Hand off to AI</h4>
              <p>Generate agent packs, PR comments, and fix prompts in one tap.</p>
            </div>
          </div>
        </section>

        <section className="scan-panel" id="quick-scan">
          <div className="hero-card">
            <div className="card-header">
              <div>
                <h3>Quick Scan</h3>
                <span>Best for pre-submit checks</span>
              </div>
              <span className="chip">Profile: Full</span>
            </div>
            <div className="dropzone">
              <div className="dropzone-content">
                <div className="drop-icon">⬆</div>
                <strong>Drag &amp; drop your bundle</strong>
                <span>.ipa, .app, .zip or <strong>zipped</strong> Xcode projects</span>
              </div>
              <input
                ref={fileRef}
                className="file-input"
                type="file"
                accept=".ipa,.app,.zip,.xcodeproj,.xcworkspace,.pbxproj,.xcworkspacedata,.plist"
                onChange={handleFileChange}
                hidden
              />
              <input
                ref={folderRef}
                className="file-input"
                type="file"
                {...({
                  webkitdirectory: "",
                  directory: "",
                } as unknown as Record<string, string>)}
                onChange={handleFolderChange}
                hidden
              />
              <div className="button-row" style={{ display: 'flex', gap: '8px', justifyContent: 'center' }}>
                <button className="secondary-button" type="button" onClick={handleChooseFile}>
                  Choose file
                </button>
                <button className="secondary-button" type="button" onClick={handleChooseFolder}>
                  <FiFolder style={{ marginRight: '6px' }} />
                  Choose folder
                </button>
              </div>
            </div>

            <div className="upload-actions">
              <button
                className="primary-button"
                type="button"
                onClick={handleUpload}
                disabled={!selectedFile || isUploading || isDiscovering}
                style={{ 
                  height: '52px', 
                  fontSize: '16px', 
                  borderRadius: '16px',
                  opacity: (!selectedFile || isUploading || isDiscovering) ? 0.5 : 1
                }}
              >
                {isUploading ? (
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <div className="spinner" />
                    <span>Analyzing...</span>
                  </div>
                ) : isDiscovering ? "Analyzing folder..." : "Run scan"}
              </button>
            </div>

            {discoveredTargets.length > 0 && (
              <div className="discovery-results" style={{ 
                marginTop: '1.5rem', 
                background: 'rgba(0, 122, 255, 0.03)', 
                border: '1px solid rgba(0, 122, 255, 0.1)', 
                padding: '1.5rem', 
                borderRadius: '20px',
                animation: 'slideDown 0.3s ease-out'
              }}>
                <div style={{ marginBottom: '1rem', fontWeight: 600, fontSize: '1rem', color: 'var(--ios-ink)' }}>
                  Auto-discovered targets:
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: '10px' }}>
                  {discoveredTargets.map((t, idx) => (
                    <button
                      key={idx}
                      className="secondary-button"
                      style={{ 
                        justifyContent: 'flex-start', 
                        padding: '14px 18px', 
                        fontSize: '14px',
                        borderRadius: '14px',
                        background: 'var(--ios-surface)',
                        boxShadow: '0 4px 12px rgba(0,0,0,0.03)'
                      }}
                      onClick={() => bundleAndSelect(pendingFiles, t)}
                    >
                      <FiFolder style={{ marginRight: '12px', color: '#007aff', fontSize: '18px' }} />
                      <div style={{ flex: 1, textAlign: 'left', display: 'flex', flexDirection: 'column' }}>
                        <span style={{ fontWeight: 600 }}>{t.name}</span>
                        <span style={{ opacity: 0.5, fontSize: '11px', textTransform: 'uppercase', letterSpacing: '0.05em' }}>{t.type} found at {t.path}</span>
                      </div>
                      <FaChevronRight style={{ opacity: 0.3, fontSize: '12px' }} />
                    </button>
                  ))}
                </div>
              </div>
            )}

            {result ? (
              <div className="report-stack">
                <div className="report-summary">
                  <div>
                    <div className="summary-label">Errors</div>
                    <div className="summary-value summary-value--error">
                      {summary.errorCount}
                    </div>
                  </div>
                  <div>
                    <div className="summary-label">Warnings</div>
                    <div className="summary-value summary-value--warning">
                      {summary.warningCount}
                    </div>
                  </div>
                  <div>
                    <div className="summary-label">Findings</div>
                    <div className="summary-value">{summary.failures.length}</div>
                  </div>
                  <div>
                    <div className="summary-label">Duration</div>
                    <div className="summary-value">{summary.duration ?? "—"}</div>
                  </div>
                </div>

                <div className="result-card">
                  <div className="result-header">Top findings</div>
                  <ul className="finding-list">
                    {summary.failures.slice(0, 5).map((item, index) => (
                      <li key={`${item.rule_id ?? "rule"}-${index}`}>
                        <strong>{String(item.rule_name ?? "Untitled rule")}</strong>
                        <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                          <span className="pill-chip" style={{ fontSize: '9px', padding: '1px 6px', opacity: 0.8 }}>{String(item.target)}</span>
                          <span>{String(item.recommendation ?? "Review this rule")}</span>
                        </div>
                      </li>
                    ))}
                    {summary.failures.length === 0 ? (
                      <li className="finding-empty">No failing rules detected.</li>
                    ) : null}
                  </ul>
                </div>

                <div className="result-card">
                  <div className="result-header">Findings by category</div>
                  <div className="category-stats">
                    {Object.entries(summary.byCategory).map(([cat, count]) => (
                      <div key={cat} className="category-stat-pill" onClick={() => toggleCategory(cat)}>
                        <span className="stat-label">{cat}</span>
                        <span className="stat-count">{count}</span>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="result-card">
                  <div className="result-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span>Detailed findings</span>
                    <div style={{ display: 'flex', gap: '8px' }}>
                      <button className="pill-chip" onClick={() => expandAll(Object.keys(summary.findingsByCategory))}>Expand all</button>
                      <button className="pill-chip" onClick={collapseAll}>Collapse all</button>
                    </div>
                  </div>
                  <div className="finding-groups">
                    {Object.entries(summary.findingsByCategory).map(([category, items]) => (
                      <div key={category} id={`cat-${category}`} className="finding-group">
                        <button className="category-header" onClick={() => toggleCategory(category)}>
                          {expandedCategories.has(category) ? <FaChevronDown /> : <FaChevronRight />}
                          <span>{category}</span>
                          <span className="pill-chip">{items.length}</span>
                        </button>
                        {expandedCategories.has(category) && (
                          <div className="category-content">
                            {items.map((item, idx) => (
                              <div key={idx} className={`finding-item finding-item--${item.severity?.toLowerCase()}`}>
                                <div className="finding-item-header">
                                  <div className="finding-item-main">
                                    <div className="finding-severity-line">
                                      <span className={`severity-pill severity-pill--${item.severity?.toLowerCase()}`}>
                                        {item.severity}
                                      </span>
                                      <strong>{item.rule_name}</strong>
                                    </div>
                                    <p className="finding-message">{item.message}</p>
                                  </div>
                                  <div className="finding-actions">
                                    <button
                                      className="ghost-button"
                                      onClick={() => {
                                        setSelectedNode(item);
                                        setAstFocus(item.rule_id || null);
                                        setIsASTModalOpen(true);
                                      }}
                                    >
                                      <FiShare2 />
                                      View in AST
                                    </button>
                                  </div>
                                </div>
                                {item.evidence ? (
                                  <div className="finding-evidence">
                                    <label>Evidence</label>
                                    <pre>
                                      {typeof item.evidence === "string" 
                                        ? item.evidence 
                                        : JSON.stringify(item.evidence, null, 2)}
                                    </pre>
                                  </div>
                                ) : null}
                                <div className="finding-recommendation">
                                  <label>Recommendation</label>
                                  <p>{item.recommendation}</p>
                                </div>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>

                <div className="result-card">
                  <div className="result-header">Export & Share</div>
                  <div className="button-row">
                    <button
                      className="secondary-button"
                      type="button"
                      onClick={handleDownloadBundle}
                      disabled={isDownloading || !selectedFile}
                    >
                      {isDownloading ? (
                        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                          <div className="spinner" />
                          <span>Preparing...</span>
                        </div>
                      ) : (
                        <>
                          <FiPackage style={{ marginRight: '8px' }} />
                          Download Agent Pack
                        </>
                      )}
                    </button>
                    <button
                      className="secondary-button"
                      type="button"
                      onClick={() => {
                        if (!rawResult) return;
                        const blob = new Blob([rawResult], { type: "application/json" });
                        const url = URL.createObjectURL(blob);
                        const link = document.createElement("a");
                        link.href = url;
                        link.download = "verifyos-report.json";
                        link.click();
                        URL.revokeObjectURL(url);
                      }}
                    >
                      Download JSON
                    </button>
                    <button
                      className={`ghost-button copy-button ${copied ? "is-copied" : ""}`}
                      type="button"
                      onClick={() => {
                        if (!rawResult) return;
                        void navigator.clipboard?.writeText(rawResult);
                        setCopied(true);
                        setToastMessage("Copied JSON to clipboard");
                        window.setTimeout(() => setCopied(false), 1500);
                      }}
                    >
                      <span className="copy-icon" aria-hidden="true" />
                      {copied ? "Copied" : "Copy JSON"}
                    </button>
                  </div>
                </div>

                {rawResult ? (
                  <details className="result-card">
                    <summary className="result-header">Raw report</summary>
                    <pre>{rawResult}</pre>
                  </details>
                ) : null}
              </div>
            ) : null}
            <div className="card-footer">
              <div>
                <strong>Next:</strong> privacy manifest, entitlements, ATS rules
              </div>
              <button className="ghost-button" type="button" onClick={handleExampleReport}>
                View example report
              </button>
            </div>
          </div>
        </section>

        <footer className="app-footer">
          <div>
            <div className="footer-label">
              verifyOS
            </div>
            <div className="footer-subtitle">Resources &amp; downloads</div>
          </div>
          <nav className="footer-links" aria-label="verifyOS links">
            <a
              href="https://github.com/0xBoji/verifyOS"
              target="_blank"
              rel="noreferrer"
              className="footer-link"
            >
              <FaGithub className="footer-icon" aria-hidden="true" />
              GitHub Repo
            </a>
            <a
              href="https://marketplace.visualstudio.com/items?itemName=0xBoji.verifyos-vscode"
              target="_blank"
              rel="noreferrer"
              className="footer-link"
            >
              <VscVscode className="footer-icon" aria-hidden="true" />
              VS Code Extension
            </a>
            <a
              href="https://crates.io/crates/verifyos-cli"
              target="_blank"
              rel="noreferrer"
              className="footer-link"
            >
              <SiRust className="footer-icon" aria-hidden="true" />
              crates.io
            </a>
          </nav>
        </footer>
      </main>

      <ASTModal 
        isOpen={isASTModalOpen} 
        onClose={() => setIsASTModalOpen(false)} 
        data={result}
        astFocus={astFocus}
      />

      <AlertModal 
        isOpen={!!alertConfig}
        title={alertConfig?.title || ""}
        message={alertConfig?.message || ""}
        actions={alertConfig?.actions || []}
      />

      <Toast 
        message={toastMessage} 
        onClear={() => setToastMessage(null)} 
      />
    </div>
  );
}
