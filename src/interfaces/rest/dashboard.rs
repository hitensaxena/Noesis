//! Web dashboard — full management UI for Noesis.
//!
//! Serves a complete single-page HTML/CSS/JS application at `/api/dashboard/`.
//! 8 views with tab navigation, SSE live updates, and full system management.
//!
//! # Views
//! - **Overview**  — System health, stats, field status cards
//! - **Fields**    — Per-field state with expandable detail panels
//! - **Signals**   — Real-time SSE signal stream with type filtering
//! - **Inject**    — Signal injection console with type/payload builder
//! - **Processors** — All 49 processors with subscriptions and capabilities
//! - **Plugins**   — Loaded plugins, version info, reload trigger
//! - **Events**    — Browsable event history with pagination + type filter
//! - **Metrics**   — Signal throughput, processor latency, field state sizes
//!
//! All data is fetched client-side from the Noesis REST API.
//! The Signals tab connects to `/api/events/stream` via SSE for live updates.

/// Generate the complete dashboard HTML page.
///
/// Returns a `&'static str` containing the full embedded SPA.
pub fn dashboard_html() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Noesis — System Management</title>
<style>
  :root {
    --bg: #0d0d1a;
    --surface: #16162b;
    --surface2: #1e1e38;
    --surface3: #2a2a48;
    --text: #e0e0f0;
    --text-dim: #8888aa;
    --text-muted: #666688;
    --accent: #6c63ff;
    --accent-hover: #7b73ff;
    --accent2: #00d4aa;
    --accent3: #ff6b9d;
    --warn: #ffa500;
    --err: #ff4757;
    --ok: #2ed573;
    --radius: 8px;
    --radius-sm: 4px;
    --border: 1px solid var(--surface3);
    --transition: 0.2s ease;
  }
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    font: 14px/1.5 -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
    background: var(--bg);
    color: var(--text);
    overflow-y: scroll;
  }

  /* Layout */
  .app { max-width: 1400px; margin: 0 auto; padding: 0 20px 40px; }
  .header {
    display: flex; align-items: center; gap: 12px; padding: 16px 0;
    border-bottom: var(--border); margin-bottom: 0; position: sticky; top: 0;
    background: var(--bg); z-index: 100;
  }
  .header h1 { font-size: 1.3rem; font-weight: 600; letter-spacing: -0.3px; }
  .header h1 span { color: var(--accent); }
  .badge {
    font-size: 0.65rem; padding: 2px 8px; border-radius: 10px;
    font-weight: 500; letter-spacing: 0.3px;
    background: var(--surface2); color: var(--text-dim);
  }
  .badge.accent { background: var(--accent); color: #fff; }
  .badge.ok { background: var(--ok); color: #000; }
  .badge.err { background: var(--err); color: #fff; }
  .header-right { margin-left: auto; display: flex; align-items: center; gap: 12px; }
  .status-dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
  .status-dot.ok { background: var(--ok); box-shadow: 0 0 6px var(--ok); }
  .status-dot.err { background: var(--err); box-shadow: 0 0 6px var(--err); }
  .status-dot.warn { background: var(--warn); box-shadow: 0 0 6px var(--warn); }
  .health-label { font-size: 0.8rem; color: var(--text-dim); }

  /* Tabs */
  .tabs {
    display: flex; gap: 0; border-bottom: var(--border); margin-bottom: 20px;
    overflow-x: auto; scrollbar-width: none; position: sticky; top: 60px;
    background: var(--bg); z-index: 90;
  }
  .tabs::-webkit-scrollbar { display: none; }
  .tab {
    padding: 10px 18px; font-size: 0.8rem; font-weight: 500; cursor: pointer;
    color: var(--text-dim); border-bottom: 2px solid transparent;
    transition: color var(--transition), border-color var(--transition);
    white-space: nowrap; user-select: none;
  }
  .tab:hover { color: var(--text); }
  .tab.active { color: var(--accent); border-bottom-color: var(--accent); }
  .tab .tab-badge {
    display: inline-block; font-size: 0.6rem; padding: 1px 6px;
    border-radius: 8px; background: var(--surface2); color: var(--text-muted);
    margin-left: 6px; vertical-align: middle;
  }
  .tab.active .tab-badge { background: var(--accent); color: #fff; }

  /* View containers */
  .view { display: none; }
  .view.active { display: block; }

  /* Cards */
  .card {
    background: var(--surface); border: var(--border); border-radius: var(--radius);
    padding: 16px; transition: border-color var(--transition);
  }
  .card:hover { border-color: var(--accent); }
  .card h3 {
    font-size: 0.75rem; color: var(--accent); margin-bottom: 8px;
    text-transform: uppercase; letter-spacing: 0.5px;
  }
  .card .stat { font-size: 1.6rem; font-weight: 700; }
  .card .sub { font-size: 0.72rem; color: var(--text-dim); margin-top: 4px; }

  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 12px; margin-bottom: 20px; }

  /* Panels */
  .panel {
    background: var(--surface); border: var(--border); border-radius: var(--radius);
    padding: 16px; flex: 1; min-width: 300px;
  }
  .panel h2 { font-size: 0.95rem; color: var(--accent2); margin-bottom: 12px; font-weight: 600; }
  .panel-row { display: flex; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; }

  /* Tables */
  table { width: 100%; border-collapse: collapse; font-size: 0.8rem; }
  th, td { padding: 8px 12px; text-align: left; border-bottom: var(--border); }
  th { color: var(--text-dim); font-weight: 500; font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.3px; }
  td { color: var(--text); font-family: 'SF Mono', 'Fira Code', monospace; font-size: 0.75rem; }
  tr:hover td { background: var(--surface2); }
  .mono { font-family: 'SF Mono', 'Fira Code', monospace; font-size: 0.75rem; }

  /* Items */
  .item-row {
    display: flex; justify-content: space-between; align-items: center;
    padding: 6px 0; border-bottom: var(--border); font-size: 0.8rem;
  }
  .item-row:last-child { border-bottom: none; }
  .item-label { color: var(--text-dim); }
  .item-value { color: var(--text); font-family: 'SF Mono', 'Fira Code', monospace; font-size: 0.75rem; }

  /* Forms */
  .form-row { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 8px; }
  .form-row label { font-size: 0.75rem; color: var(--text-dim); min-width: 100px; padding-top: 8px; }
  .form-row input, .form-row select, .form-row textarea {
    background: var(--surface2); border: var(--border); color: var(--text);
    padding: 8px 12px; border-radius: var(--radius-sm); font-size: 0.85rem; flex: 1;
  }
  .form-row textarea { min-height: 60px; font-family: 'SF Mono', 'Fira Code', monospace; font-size: 0.75rem; resize: vertical; }
  .form-row input:focus, .form-row select:focus, .form-row textarea:focus {
    outline: none; border-color: var(--accent);
  }
  .btn {
    background: var(--surface2); color: var(--text); border: var(--border);
    padding: 8px 16px; border-radius: var(--radius-sm); cursor: pointer;
    font-size: 0.8rem; transition: background var(--transition);
  }
  .btn:hover { background: var(--surface3); }
  .btn.primary { background: var(--accent); color: #fff; border-color: var(--accent); }
  .btn.primary:hover { background: var(--accent-hover); }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-sm { padding: 4px 10px; font-size: 0.72rem; }

  /* SSE signal feed */
  .signal-feed { max-height: 500px; overflow-y: auto; font-size: 0.75rem; }
  .signal-feed .entry {
    padding: 3px 0; border-bottom: 1px solid var(--surface2);
    display: flex; gap: 8px; align-items: center;
    animation: fadeIn 0.3s ease;
  }
  @keyframes fadeIn { from { opacity: 0; transform: translateY(-4px); } to { opacity: 1; transform: translateY(0); } }
  .signal-feed .entry .type { color: var(--accent); min-width: 260px; font-family: monospace; }
  .signal-feed .entry .depth {
    font-size: 0.65rem; padding: 1px 6px; border-radius: 8px;
    background: var(--surface2); color: var(--text-dim);
  }
  .signal-feed .entry .src { color: var(--text-muted); font-size: 0.65rem; }
  .signal-feed .entry .time { margin-left: auto; color: var(--text-muted); font-size: 0.65rem; }
  .signal-feed-controls {
    display: flex; gap: 8px; align-items: center; margin-bottom: 8px;
  }
  .signal-filter {
    background: var(--surface2); border: var(--border); color: var(--text);
    padding: 4px 8px; border-radius: var(--radius-sm); font-size: 0.75rem; flex: 1;
  }

  /* Event browser */
  .pagination { display: flex; gap: 8px; align-items: center; margin-top: 12px; }
  .pagination .page-info { font-size: 0.75rem; color: var(--text-dim); }

  /* Metrics bars */
  .metric-bar { margin-bottom: 8px; }
  .metric-bar .label { font-size: 0.72rem; color: var(--text-dim); margin-bottom: 2px; }
  .metric-bar .bar-wrap { background: var(--surface2); border-radius: 4px; height: 16px; overflow: hidden; }
  .metric-bar .bar-fill {
    height: 100%; border-radius: 4px; transition: width 0.5s ease;
    background: linear-gradient(90deg, var(--accent), var(--accent2));
  }
  .metric-bar .bar-fill.warn { background: linear-gradient(90deg, var(--warn), var(--err)); }
  .metric-bar .bar-value { font-size: 0.68rem; color: var(--text-dim); margin-top: 1px; }

  /* Detail expand */
  .detail-json {
    font-size: 0.68rem; line-height: 1.4; background: var(--surface2);
    padding: 8px; border-radius: var(--radius-sm); overflow-x: auto;
    max-height: 300px; overflow-y: auto; white-space: pre-wrap; display: none;
    color: var(--text-dim); margin-top: 6px;
  }
  .detail-json.show { display: block; }

  /* Quick inject */
  .quick-injects { display: flex; gap: 6px; flex-wrap: wrap; margin: 8px 0; }
  .quick-injects .qi-btn {
    font-size: 0.68rem; padding: 4px 10px; border-radius: 12px;
    border: 1px solid var(--surface3); background: var(--surface2);
    color: var(--text-dim); cursor: pointer; transition: all var(--transition);
  }
  .quick-injects .qi-btn:hover { border-color: var(--accent); color: var(--accent); }

  /* Error/loading */
  .error-msg { color: var(--err); font-size: 0.8rem; padding: 8px; }
  .loading { color: var(--text-dim); font-size: 0.8rem; padding: 8px; font-style: italic; }

  /* Plugin manager */
  .plugin-card {
    background: var(--surface2); border: var(--border); border-radius: var(--radius);
    padding: 12px; margin-bottom: 8px;
  }
  .plugin-card .plugin-name { font-weight: 600; color: var(--accent2); }
  .plugin-card .plugin-meta { font-size: 0.72rem; color: var(--text-dim); margin: 4px 0; }

  /* Responsive */
  @media (max-width: 768px) {
    .tabs { gap: 0; font-size: 0.75rem; }
    .tab { padding: 8px 12px; }
    .grid { grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); }
    .panel-row { flex-direction: column; }
    .header { flex-wrap: wrap; }
  }

  /* Cascade flow visualization */
  .cf-group { margin-bottom: 12px; }
  .cf-group-title { font-size:0.72rem; color:var(--accent2); margin-bottom:4px; font-weight:600; text-transform:uppercase; letter-spacing:0.3px; }
  .cf-row { display:flex; gap:4px; flex-wrap:wrap; }
  .cf-node {
    display:inline-flex; align-items:center; gap:4px; padding:3px 8px; border-radius:4px;
    font-size:0.68rem; font-family:'SF Mono','Fira Code',monospace; border:1px solid var(--surface3);
    background:var(--surface2); transition:all 0.2s; cursor:default;
  }
  .cf-node:hover { border-color:var(--accent); transform:translateY(-1px); }
  .cf-node .cf-count { font-weight:700; min-width:18px; text-align:right; }
  .cf-node .cf-name { color:var(--text); max-width:160px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .cf-arrow { color:var(--text-muted); font-size:0.6rem; padding:0 2px; align-self:center; }

  /* Cascade field groups - colors match signal type prefixes */
  .cf-memory .cf-count { color:#2ed573; }
  .cf-identity .cf-count { color:#6c63ff; }
  .cf-agency .cf-count { color:#ffa500; }
  .cf-awareness .cf-count { color:#00d4aa; }
  .cf-reasoning .cf-count { color:#ff6b9d; }
  .cf-action .cf-count { color:#ff4757; }
  .cf-simulation .cf-count { color:#a855f7; }
  .cf-kernel .cf-count { color:#8888aa; }
  .cf-generic .cf-count { color:#8888aa; }

  /* Scrollbar */
  ::-webkit-scrollbar { width: 6px; height: 6px; }
  ::-webkit-scrollbar-track { background: var(--bg); }
  ::-webkit-scrollbar-thumb { background: var(--surface3); border-radius: 3px; }
  ::-webkit-scrollbar-thumb:hover { background: var(--text-muted); }
</style>
</head>
<body>
<div class="app">
  <!-- Header -->
  <div class="header">
    <h1><span>N</span>oesis</h1>
    <span class="badge accent">v0.1.0</span>
    <span class="badge" id="fieldBadge">0 fields</span>
    <span class="badge" id="procBadge">0 procs</span>
    <div class="header-right">
      <span class="status-dot ok" id="healthDot"></span>
      <span class="health-label" id="healthLabel">running</span>
    </div>
  </div>

  <!-- Tab Navigation -->
  <div class="tabs" id="tabs">
    <div class="tab active" data-view="overview">Overview <span class="tab-badge" id="badge-ov">8F</span></div>
    <div class="tab" data-view="fields">Fields <span class="tab-badge">8</span></div>
    <div class="tab" data-view="signals">Signals <span class="tab-badge" id="badge-sig">0/s</span></div>
    <div class="tab" data-view="inject">Inject</div>
    <div class="tab" data-view="processors">Processors <span class="tab-badge" id="badge-proc">-</span></div>
    <div class="tab" data-view="plugins">Plugins</div>
    <div class="tab" data-view="events">Events</div>
    <div class="tab" data-view="metrics">Metrics</div>
  </div>

  <!-- ====== VIEW: Overview ====== -->
  <div class="view active" id="view-overview">
    <div class="grid" id="ovFieldGrid">
      <div class="card"><h3>Memory</h3><div class="stat" id="ov-memory">-</div><div class="sub" id="ov-sub-memory">episodes / memories</div></div>
      <div class="card"><h3>Identity</h3><div class="stat" id="ov-identity">-</div><div class="sub" id="ov-sub-identity">beliefs / traits</div></div>
      <div class="card"><h3>Agency</h3><div class="stat" id="ov-agency">-</div><div class="sub" id="ov-sub-agency">goals</div></div>
      <div class="card"><h3>Action</h3><div class="stat" id="ov-action">-</div><div class="sub" id="ov-sub-action">projects / tasks</div></div>
      <div class="card"><h3>Awareness</h3><div class="stat" id="ov-awareness">-</div><div class="sub" id="ov-sub-awareness">attention / curiosity</div></div>
      <div class="card"><h3>Reasoning</h3><div class="stat" id="ov-reasoning">-</div><div class="sub" id="ov-sub-reasoning">concepts / analogies</div></div>
      <div class="card"><h3>Simulation</h3><div class="stat" id="ov-simulation">-</div><div class="sub" id="ov-sub-simulation">scenarios / forecasts</div></div>
      <div class="card"><h3>Graph</h3><div class="stat" id="ov-graph">-</div><div class="sub" id="ov-sub-graph">entities / relations</div></div>
    </div>
    <div class="panel-row">
      <div class="panel">
        <h2>System</h2>
        <div id="ovSysStats">
          <div class="item-row"><span class="item-label">Fields</span><span class="item-value" id="ov-sys-fields">-</span></div>
          <div class="item-row"><span class="item-label">Processors</span><span class="item-value" id="ov-sys-procs">-</span></div>
          <div class="item-row"><span class="item-label">Signal Types</span><span class="item-value" id="ov-sys-signals">-</span></div>
          <div class="item-row"><span class="item-label">Capabilities</span><span class="item-value" id="ov-sys-caps">-</span></div>
          <div class="item-row"><span class="item-label">Signals Processed</span><span class="item-value" id="ov-sys-total">-</span></div>
        </div>
      </div>
      <div class="panel">
        <h2>Signal Rate</h2>
        <div id="ovSignalRate">
          <div style="font-size:2rem;font-weight:700;color:var(--accent2);" id="ov-rate-val">0</div>
          <div style="font-size:0.72rem;color:var(--text-dim);">signals / min</div>
          <div style="margin-top:12px;">
            <div id="ovRateBars"></div>
          </div>
        </div>
      </div>
    </div>
    <!-- Cascade Flow Visualization -->
    <div class="panel" style="min-width:100%;">
      <h2>Signal Cascade Flow</h2>
      <div id="cascadeFlow" style="font-size:0.75rem;">
        <div class="text-dim" style="padding:8px;text-align:center;">Loading cascade data...</div>
      </div>
      <div id="cascadeLegend" style="display:flex;gap:12px;flex-wrap:wrap;margin-top:8px;font-size:0.65rem;"></div>
    </div>
  </div>

  <!-- ====== VIEW: Fields ====== -->
  <div class="view" id="view-fields">
    <div style="display:flex;gap:8px;flex-wrap:wrap;margin-bottom:12px;" id="fieldChips">
      <div class="text-dim" style="font-size:0.8rem;padding:8px;">Loading fields...</div>
    </div>
    <div id="fieldDetailArea" style="display:none;">
      <div class="panel">
        <div id="fieldDetailHeader" style="display:flex;justify-content:space-between;align-items:center;margin-bottom:8px;">
          <h2 id="fieldDetailTitle" style="color:var(--accent);margin:0;">Field</h2>
          <button class="btn btn-sm" onclick="document.getElementById('fieldDetailArea').style.display='none';document.getElementById('fieldChips').style.display='';">Close</button>
        </div>
        <div id="fieldDetailContent"></div>
      </div>
    </div>
  </div>

  <!-- ====== VIEW: Signals (SSE) ====== -->
  <div class="view" id="view-signals">
    <div class="panel">
      <div class="signal-feed-controls">
        <select class="signal-filter" id="sigFilter">
          <option value="">All signal types</option>
          <option value="memory">memory.*</option>
          <option value="identity">identity.*</option>
          <option value="agency">agency.*</option>
          <option value="action">action.*</option>
          <option value="awareness">awareness.*</option>
          <option value="reasoning">reasoning.*</option>
          <option value="simulation">simulation.*</option>
          <option value="kernel">kernel.*</option>
        </select>
        <span style="font-size:0.72rem;color:var(--text-dim);" id="sigCount">0 events</span>
        <button class="btn btn-sm" id="sigPauseBtn" onclick="toggleSigPause()">Pause</button>
        <button class="btn btn-sm" onclick="clearSignals()">Clear</button>
      </div>
      <div class="signal-feed" id="signalFeed"><div class="text-dim" style="padding:12px;text-align:center;">Connecting to SSE stream...</div></div>
    </div>
  </div>

  <!-- ====== VIEW: Inject ====== -->
  <div class="view" id="view-inject">
    <div class="panel-row">
      <div class="panel" style="max-width:600px;">
        <h2>Signal Injection Console</h2>
        <div class="form-row">
          <label>Signal Type</label>
          <select id="injType"><option value="">Loading types...</option></select>
        </div>
        <div class="form-row">
          <label>Payload (JSON)</label>
          <textarea id="injPayload" placeholder='{"text": "optional content"}'>{"text":"injected via dashboard"}</textarea>
        </div>
        <div class="form-row">
          <label></label>
          <button class="btn primary" onclick="injectSignal()" id="injBtn">Inject Signal</button>
        </div>
        <div id="injResult"></div>
      </div>
      <div class="panel">
        <h2>Quick Inject</h2>
        <div class="quick-injects" id="quickInject"></div>
      </div>
    </div>
  </div>

  <!-- ====== VIEW: Processors ====== -->
  <div class="view" id="view-processors">
    <div class="panel-row">
      <div class="panel">
        <h2>Processor Registry</h2>
        <div class="form-row" style="margin-bottom:12px;">
          <input type="text" id="procSearch" placeholder="Search processors..." oninput="renderProcessors()" style="flex:1;">
        </div>
        <div id="processorList"></div>
      </div>
    </div>
  </div>

  <!-- ====== VIEW: Plugins ====== -->
  <div class="view" id="view-plugins">
    <div class="panel-row">
      <div class="panel" style="max-width:600px;">
        <h2>Plugin Manager</h2>
        <div style="margin-bottom:12px;">
          <button class="btn btn-sm" onclick="reloadPlugins()">Reload Plugins</button>
          <span style="font-size:0.72rem;color:var(--text-dim);margin-left:8px;" id="pluginStatus"></span>
        </div>
        <div id="pluginList"></div>
      </div>
    </div>
  </div>

  <!-- ====== VIEW: Events ====== -->
  <div class="view" id="view-events">
    <div class="panel">
      <h2>Event History Browser</h2>
      <div class="form-row">
        <label>Filter</label>
        <input type="text" id="evtFilter" placeholder="event type filter (e.g. memory.*)" style="flex:1;">
        <button class="btn btn-sm" onclick="loadEvents()">Apply</button>
        <button class="btn btn-sm" onclick="document.getElementById('evtFilter').value='';loadEvents();">Clear</button>
      </div>
      <div id="eventTable"></div>
      <div class="pagination">
        <button class="btn btn-sm" onclick="prevEvents()">← Prev</button>
        <span class="page-info" id="evtPageInfo">Page 1</span>
        <button class="btn btn-sm" onclick="nextEvents()">Next →</button>
        <select id="evtLimit" onchange="loadEvents()" style="background:var(--surface2);border:var(--border);color:var(--text);padding:4px 8px;border-radius:4px;font-size:0.75rem;margin-left:auto;">
          <option value="20">20</option>
          <option value="50" selected>50</option>
          <option value="100">100</option>
        </select>
      </div>
    </div>
  </div>

  <!-- ====== VIEW: Metrics ====== -->
  <div class="view" id="view-metrics">
    <div class="panel-row">
      <div class="panel">
        <h2>Signal Distribution</h2>
        <div id="sigDistBars"></div>
      </div>
      <div class="panel">
        <h2>Processor Latency</h2>
        <div id="procLatencyBars"></div>
      </div>
    </div>
    <div class="panel-row">
      <div class="panel">
        <h2>Field State Sizes</h2>
        <div id="fieldSizeBars"></div>
      </div>
      <div class="panel">
        <h2>Metrics Snapshot</h2>
        <div id="metricsRaw"><div class="loading">Loading...</div></div>
      </div>
    </div>
  </div>
</div>

<script>
// ============================================================
// State
// ============================================================
const API = "";
const AUTH = "Bearer sk-0b29553a15600589-a12a35-70230fbc";
const AUTH_HEADER = { "Authorization": AUTH };
let activeTab = 'overview';
let fields = [];
let signalCount = 0;
let sigPaused = false;
let sigBuffer = [];
let evtPage = 0;
let signalTypes = [];
let sseConnected = false;
let sseReconnectTimer = null;
let signalRateTimestamps = [];

// ============================================================
// Utilities
// ============================================================
const $ = id => document.getElementById(id);
const api = async (url, opts) => {
  const headers = opts?.headers || {};
  const r = await fetch(API + url, { ...opts, headers: { ...headers, ...AUTH_HEADER } });
  if (!r.ok) throw new Error(`${r.status} ${r.statusText}`);
  return r.json();
};
const fmt = v => v !== undefined && v !== null ? v : '-';
const esc = s => String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');

// ============================================================
// Tab Switching
// ============================================================
document.querySelectorAll('.tab').forEach(tab => {
  tab.addEventListener('click', () => {
    document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.view').forEach(v => v.classList.remove('active'));
    tab.classList.add('active');
    const view = document.getElementById('view-' + tab.dataset.view);
    if (view) view.classList.add('active');
    activeTab = tab.dataset.view;
    if (activeTab === 'events') loadEvents();
    if (activeTab === 'metrics') loadMetrics();
    if (activeTab === 'processors') loadProcessors();
    if (activeTab === 'plugins') loadPlugins();
    if (activeTab === 'fields') loadFields();
  });
});

// ============================================================
// Header/Health refresh + Overview
// ============================================================
async function refreshOverview() {
  try {
    const health = await api('/api/health');
    $('healthDot').className = 'status-dot ok';
    $('healthLabel').textContent = 'running';
  } catch(e) {
    $('healthDot').className = 'status-dot err';
    $('healthLabel').textContent = 'unreachable';
  }

  try {
    const stats = await api('/api/stats');
    $('fieldBadge').textContent = stats.fields + ' fields';
    $('procBadge').textContent = stats.processors + ' procs';
    $('ov-sys-fields').textContent = fmt(stats.fields);
    $('ov-sys-procs').textContent = fmt(stats.processors);
    $('ov-sys-signals').textContent = fmt(stats.signal_types);
    signalTypes = stats.signal_names || [];
  } catch(e) {}

  try {
    const caps = await api('/api/capabilities');
    $('ov-sys-caps').textContent = fmt(caps.total);
  } catch(e) {}

  try {
    const ov = await api('/api/observability/overview');
    const total = ov.signals_total;
    $('ov-sys-total').textContent = fmt(total);
  } catch(e) {}

  // Helper: safe nested property access
  function getCount(obj, path) {
    if (!obj) return 0;
    var parts = path.split('.');
    var cur = obj;
    for (var i = 0; i < parts.length; i++) {
      if (cur === null || cur === undefined) return 0;
      cur = cur[parts[i]];
    }
    if (cur === undefined || cur === null) return 0;
    if (typeof cur === 'number') return cur;
    if (typeof cur === 'object') {
      if (Array.isArray(cur)) return cur.length;
      if (cur.count !== undefined) return cur.count;
      if (cur.total !== undefined) return cur.total;
      if (cur.length !== undefined) return cur.length;
      if (cur.active !== undefined) return cur.active;
      if (cur.depth !== undefined) return cur.depth;
      return Object.keys(cur).length;
    }
    return 0;
  }

  // Field cards with nested path support
  const fDefs = [
    {key:'memory', ep:'/api/memory/detail', fields:'episodic.count', sub:'semantic.count'},
    {key:'identity', ep:'/api/identity/detail', fields:'beliefs.count', sub:'traits.count'},
    {key:'agency', ep:'/api/agency/detail', fields:'goals.active', sub:'goals.items.length'},
    {key:'action', ep:'/api/core/detail', fields:'config.features.length', sub:'config.rest_api_enabled'},
    {key:'awareness', ep:'/api/awareness/detail', fields:'attention.focus_stack.depth', sub:'curiosity.count'},
    {key:'reasoning', ep:'/api/cognition/meta', fields:'insights.length', sub:'decisions.length'},
    {key:'simulation', ep:'/api/simulation/detail', fields:'scenarios.count', sub:'forecasts.count'},
    {key:'graph', ep:'/api/graph', fields:'graph.entity_count', sub:'graph.relation_count'},
  ];
  for (const f of fDefs) {
    const statEl = document.getElementById('ov-'+f.key);
    const subEl = document.getElementById('ov-sub-'+f.key);
    try {
      const data = await api(f.ep);
      const v1 = getCount(data, f.fields);
      const v2 = getCount(data, f.sub);
      statEl.textContent = v1 + ' / ' + v2;
      subEl.textContent = f.sub.split('.').pop();
    } catch(e) {
      statEl.textContent = '—';
      subEl.textContent = f.key + ' awaiting data';
    }
  }

  // Cascade flow visualization
  try {
    signalStats = await api('/api/observability/signals');
  } catch(e) {}
  renderCascadeFlow();

  // Signal rate
  updateSignalRate();
}

// ============================================================// Signal Rate Tracking
// ============================================================
function updateSignalRate() {
  const now = Date.now();
  signalRateTimestamps = signalRateTimestamps.filter(t => now - t < 60000);
  const rate = signalRateTimestamps.length;
  $('ov-rate-val').textContent = rate;
  $('badge-sig').textContent = rate + '/m';

  // Mini bar chart of last 6 10-second buckets
  const buckets = Array(6).fill(0);
  for (const t of signalRateTimestamps) {
    const idx = Math.min(5, Math.floor((now - t) / 10000));
    if (idx < 6) buckets[5-idx]++;
  }
  const max = Math.max(1, ...buckets);
  $('ovRateBars').innerHTML = buckets.map((b,i) =>
    `<div style="display:flex;align-items:center;gap:4px;margin:2px 0;">
      <span style="font-size:0.6rem;color:var(--text-muted);width:20px;">${(5-i)*10}s</span>
      <div style="flex:1;background:var(--surface2);height:10px;border-radius:3px;overflow:hidden;">
        <div style="width:${b/max*100}%;height:100%;background:var(--accent2);border-radius:3px;transition:width 0.5s;"></div>
      </div>
      <span style="font-size:0.6rem;color:var(--text-dim);width:24px;">${b}</span>
    </div>`
  ).join('');
}

// ============================================================
// Cascade Flow Visualization
// ============================================================
var signalStats = null;

function renderCascadeFlow() {
  try {
    var stats = signalStats || {};
    var groups = {
      memory: [], identity: [], agency: [], awareness: [],
      reasoning: [], action: [], simulation: [], kernel: []
    };
    var groupNames = {
      memory:'Memory', identity:'Identity', agency:'Agency', awareness:'Awareness',
      reasoning:'Reasoning', action:'Action', simulation:'Simulation', kernel:'Kernel'
    };
    var groupColors = {
      memory:'#2ed573', identity:'#6c63ff', agency:'#ffa500', awareness:'#00d4aa',
      reasoning:'#ff6b9d', action:'#ff4757', simulation:'#a855f7', kernel:'#8888aa'
    };
    var groupIcons = {
      memory:'📀', identity:'🧑', agency:'🎯', awareness:'💡',
      reasoning:'🧠', action:'⚡', simulation:'🔮', kernel:'⚙️'
    };

    for (var key in stats) {
      var val = stats[key];
      if (typeof val !== 'number') continue;
      var prefix = key.split('.')[0];
      var group = groups[prefix];
      if (group) group.push({key:key, val:val});
    }

    for (var g in groups) {
      groups[g].sort(function(a,b) { return b.val - a.val; });
    }

    var flowEl = document.getElementById('cascadeFlow');
    var legendEl = document.getElementById('cascadeLegend');
    var html = '';
    var legendHtml = '';

    for (var group in groups) {
      var items = groups[group];
      if (items.length === 0) continue;
      var icon = groupIcons[group] || '○';
      var name = groupNames[group] || group;
      var color = groupColors[group] || '#8888aa';
      var total = 0;
      for (var i = 0; i < items.length; i++) total += items[i].val;

      html += '<div class="cf-group">';
      html += '<div class="cf-group-title">' + icon + ' ' + name + ' (' + total + ')</div>';
      html += '<div class="cf-row">';
      for (var i = 0; i < Math.min(items.length, 10); i++) {
        var item = items[i];
        var shortName = item.key.split('.').slice(1).join('.');
        html += '<span class="cf-node cf-' + group + '">';
        html += '<span class="cf-count">' + item.val + '</span>';
        html += '<span class="cf-name" title="' + item.key + '">' + shortName + '</span>';
        html += '</span>';
      }
      if (items.length > 10) {
        html += '<span class="cf-node" style="border-color:transparent;">+' + (items.length - 10) + ' more</span>';
      }
      html += '</div></div>';

      legendHtml += '<span style="display:inline-flex;align-items:center;gap:3px;">';
      legendHtml += '<span style="width:8px;height:8px;border-radius:50%;display:inline-block;background:' + color + ';"></span>';
      legendHtml += '<span style="color:var(--text-dim);font-size:0.65rem;">' + name + '</span>';
      legendHtml += '</span> ';
    }

    if (!html) {
      html = '<div class="text-dim" style="text-align:center;padding:8px;">No signal data yet</div>';
    }

    flowEl.innerHTML = html;
    legendEl.innerHTML = legendHtml;
  } catch(e) {
    document.getElementById('cascadeFlow').innerHTML = '<div class="text-dim">Cascade data pending...</div>';
  }
}

// ============================================================
// SSE Connection — Live Signal Stream
// ============================================================
function connectSSE() {
  if (sseReconnectTimer) { clearTimeout(sseReconnectTimer); sseReconnectTimer = null; }
  const es = new EventSource('/api/events/stream');
  sseConnected = true;

  es.addEventListener('signal', e => {
    try {
      const sig = JSON.parse(e.data);
      signalCount++;
      signalRateTimestamps.push(Date.now());
      if (signalRateTimestamps.length > 1000) signalRateTimestamps = signalRateTimestamps.slice(-500);
      addSignalEntry(sig);
      updateSignalRate();
    } catch(err) {}
  });

  es.onerror = () => {
    sseConnected = false;
    es.close();
    $('signalFeed').insertAdjacentHTML('afterbegin',
      '<div class="entry" style="color:var(--err);">⚠ SSE disconnected — reconnecting...</div>');
    sseReconnectTimer = setTimeout(connectSSE, 3000);
  };
}

function addSignalEntry(sig) {
  if (sigPaused) { sigBuffer.push(sig); if (sigBuffer.length > 200) sigBuffer.shift(); return; }
  const filter = $('sigFilter').value;
  if (filter && !sig.type.startsWith(filter)) return;

  const feed = $('signalFeed');
  const depth = sig.depth || 0;
  const depthClass = depth > 3 ? 'color:var(--err);' : depth > 1 ? 'color:var(--warn);' : 'color:var(--accent2);';
  const ts = sig.timestamp ? new Date(sig.timestamp).toLocaleTimeString() : '';

  feed.insertAdjacentHTML('afterbegin',
    `<div class="entry">
      <span class="type">${esc(sig.type)}</span>
      <span class="depth" style="${depthClass}">d${depth}</span>
      <span class="src">${esc(sig.source||'')}</span>
      <span class="time">${ts}</span>
    </div>`);

  // Limit entries
  while (feed.children.length > 500) feed.removeChild(feed.lastChild);
  $('sigCount').textContent = signalCount + ' events';
}

function toggleSigPause() {
  sigPaused = !sigPaused;
  $('sigPauseBtn').textContent = sigPaused ? 'Resume' : 'Pause';
  if (!sigPaused) {
    // Flush buffer
    for (const sig of sigBuffer) addSignalEntry(sig);
    sigBuffer = [];
  }
}

function clearSignals() {
  $('signalFeed').innerHTML = '';
  signalCount = 0;
  $('sigCount').textContent = '0 events';
}

// ============================================================
// Fields View
// ============================================================
var fieldEndpoints = {
  memory: '/api/memory/detail',
  identity: '/api/identity/detail',
  agency: '/api/agency/detail',
  action: '/api/awareness/detail',
  awareness: '/api/awareness/detail',
  reasoning: '/api/cognition/meta',
  simulation: '/api/simulation/detail',
  knowledge_graph: '/api/graph',
};

var fieldIcons = {
  memory: '📀', identity: '🧑', agency: '🎯', action: '⚡',
  awareness: '💡', reasoning: '🧠', simulation: '🔮', knowledge_graph: '🌐'
};

function renderFieldObj(v) {
  if (v === null || v === undefined) return '<span class="text-dim">—</span>';
  if (typeof v === 'string') return esc(v);
  if (typeof v === 'number') return '<span style="color:var(--accent2);font-weight:600;">' + v + '</span>';
  if (typeof v === 'boolean') return v ? '✓' : '✗';
  if (Array.isArray(v)) {
    if (v.length === 0) return '<span class="text-dim">empty</span>';
    return '<div style="margin-left:8px;">' + v.map(function(item) {
      if (typeof item === 'object') {
        var txt = '';
        for (var k in item) {
          if (k === 'meta' || k === 'id' || k === 'episode_ids' || k === 'narrative_id' || k === 'curiosity_id' || k === 'goal_id' || k === 'edge_id' || k === 'entity_id') continue;
          var val = typeof item[k] === 'object' ? JSON.stringify(item[k]).slice(0,60) : item[k];
          if (String(val).length > 80) val = String(val).slice(0,80) + '…';
          txt += '<span style="color:var(--text-dim);font-size:0.68rem;">' + esc(k) + ': </span><span style="font-size:0.68rem;">' + esc(val) + '</span> ';
        }
        return '<div style="padding:2px 0;border-bottom:1px solid var(--surface2);">' + txt + '</div>';
      }
      return '<div style="padding:2px 0;">' + esc(item) + '</div>';
    }).join('') + '</div>';
  }
  if (typeof v === 'object') {
    var keys = Object.keys(v);
    if (keys.length === 0) return '<span class="text-dim">empty</span>';
    var html = '<div style="margin-left:4px;">';
    for (var i = 0; i < keys.length; i++) {
      var k = keys[i];
      if (k === '_meta' || k === 'note') continue;
      var val = v[k];
      var display = renderFieldObj(val);
      html += '<div style="padding:1px 0;"><span style="color:var(--text-dim);font-size:0.72rem;">' + esc(k) + ': </span>' + display + '</div>';
    }
    html += '</div>';
    return html;
  }
  return esc(String(v));
}

function fieldSummary(name, data) {
  try {
    switch(name) {
      case 'memory':
        var ep = ((data.episodic||{}).count||0);
        var sem = ((data.semantic||{}).count||0);
        var graph = ((data.graph||{}).entities||0);
        return ep + ' ep, ' + sem + ' sem, ' + graph + ' graph';
      case 'identity':
        var b = ((data.beliefs||{}).items||[]).length;
        var t = ((data.traits||{}).items||[]).length;
        var v = ((data.values||{}).items||[]).length;
        return b + ' beliefs, ' + t + ' traits, ' + v + ' values';
      case 'agency':
        var g = (data.goals||{}).items||[];
        var p = ((data.projects||{}).count||0);
        return g.length + ' goals, ' + p + ' projects';
      case 'awareness':
        var f = (data.attention||{}).focus_stack||{};
        var c = ((data.curiosity||{}).count||0);
        return 'focus:' + (f.depth||0) + ', curiosity:' + c;
      case 'reasoning':
        var i = (data.insights||[]).length;
        var d = (data.decisions||[]).length;
        return i + ' insights, ' + d + ' decisions';
      case 'simulation':
        var s = ((data.scenarios||{}).count||0);
        var a = ((data.assumptions||{}).count||0);
        return s + ' scenarios, ' + a + ' assumptions';
      case 'knowledge_graph':
        var g = data.graph || data;
        return (g.entity_count||0) + ' entities, ' + (g.relation_count||0) + ' relations';
      default: return '';
    }
  } catch(e) { return ''; }
}

async function loadFields() {
  try {
    var stats = await api('/api/stats');
    var names = stats.field_names || [];
    var chips = document.getElementById('fieldChips');
    var html = '';
    for (var i = 0; i < names.length; i++) {
      var name = names[i];
      var icon = fieldIcons[name] || '○';
      try {
        var ep = fieldEndpoints[name] || '';
        var data = ep ? await api(ep) : {};
        var summary = fieldSummary(name, data);
        html += '<div class="cf-node" onclick="showFieldDetail(\'' + name + '\')" style="cursor:pointer;flex:1;min-width:160px;">';
        html += '<span style="font-size:0.8rem;">' + icon + ' <b>' + name + '</b></span>';
        html += '<br><span style="font-size:0.65rem;color:var(--text-dim);white-space:nowrap;">' + summary + '</span>';
        html += '</div>';
      } catch(e) {
        html += '<div class="cf-node" onclick="showFieldDetail(\'' + name + '\')" style="cursor:pointer;opacity:0.6;">' + icon + ' ' + name + ' <span class="text-dim">(pending)</span></div>';
      }
    }
    chips.innerHTML = html || '<div class="text-dim">No fields</div>';
  } catch(e) {
    document.getElementById('fieldChips').innerHTML = '<div class="error-msg">' + e.message + '</div>';
  }
}

async function showFieldDetail(name) {
  document.getElementById('fieldChips').style.display = 'none';
  document.getElementById('fieldDetailArea').style.display = 'block';
  document.getElementById('fieldDetailTitle').textContent = name;
  var content = document.getElementById('fieldDetailContent');
  content.innerHTML = '<div class="loading">Loading...</div>';

  try {
    var ep = fieldEndpoints[name] || '';
    var data = ep ? await api(ep) : {};
    var html = '';
    var keys = Object.keys(data);

    for (var i = 0; i < keys.length; i++) {
      var k = keys[i];
      if (k === '_meta') continue;

      var val = data[k];
      html += '<div style="margin-bottom:8px;background:var(--surface2);border-radius:var(--radius);padding:8px;">';
      html += '<div style="font-size:0.75rem;color:var(--accent2);font-weight:600;text-transform:uppercase;letter-spacing:0.3px;margin-bottom:4px;">' + esc(k) + '</div>';
      html += renderFieldObj(val);
      if (typeof val === 'object' && val !== null && val.note) {
        html += '<div style="font-size:0.65rem;color:var(--text-dim);margin-top:4px;font-style:italic;">' + esc(val.note) + '</div>';
      }
      html += '</div>';
    }

    content.innerHTML = html || '<div class="text-dim">No field data</div>';
  } catch(e) {
    content.innerHTML = '<div class="error-msg">' + e.message + '</div>';
  }
}

// ============================================================
// Inject View
// ============================================================
async function loadInjectTypes() {
  try {
    const types = await api('/api/signals');
    const sel = $('injType');
    sel.innerHTML = types.signal_types.map(t =>
      `<option value="${t.type}">${t.type}</option>`
    ).join('') || '<option>No types loaded</option>';

    // Quick inject buttons
    const qi = $('quickInject');
    const quick = ['memory.capture.ingested','agency.goals.created','awareness.curiosity.detected',
                   'reasoning.metacognition.insight','action.planning.plan_ready'];
    qi.innerHTML = quick.map(t =>
      `<button class="qi-btn" onclick="quickInject('${t}')">${t.split('.').slice(-2).join('.')}</button>`
    ).join('');
  } catch(e) {}
}

async function injectSignal() {
  const btn = $('injBtn');
  btn.disabled = true;
  $('injResult').innerHTML = '<span class="loading">Sending...</span>';
  try {
    const sigType = $('injType').value;
    let payload = {};
    try { payload = JSON.parse($('injPayload').value || '{}'); } catch(e) {}
    const data = await api('/api/signals/inject', {
      method: 'POST',
      headers: {'Content-Type':'application/json'},
      body: JSON.stringify({signal_type: sigType, payload}),
    });
    $('injResult').innerHTML = `<span style="color:var(--ok)">✓ ${data.status || 'injected'}</span>`;
  } catch(e) {
    $('injResult').innerHTML = `<span style="color:var(--err)">✗ ${e.message}</span>`;
  }
  btn.disabled = false;
}

function quickInject(type) {
  $('injType').value = type;
  injectSignal();
}

// ============================================================
// Processors View
// ============================================================
let processorData = [];

async function loadProcessors() {
  try {
    const caps = await api('/api/capabilities');
    processorData = caps.capabilities || [];
    renderProcessors();
    $('badge-proc').textContent = processorData.length;
  } catch(e) {
    $('processorList').innerHTML = `<div class="error-msg">${e.message}</div>`;
  }
}

function renderProcessors() {
  const query = ($('procSearch').value || '').toLowerCase();
  const filtered = processorData.filter(c =>
    c.id.toLowerCase().includes(query) ||
    (c.providers||[]).some(p => (p.processor||'').toLowerCase().includes(query))
  );
  $('processorList').innerHTML = filtered.map(c => {
    const provs = (c.providers||[]).map(p =>
      `<div style="font-size:0.72rem;padding:2px 0;color:var(--text-dim);">
        <span style="color:var(--accent2);">${esc(p.name)}</span>
        <span style="color:var(--text-muted);"> (${esc(p.processor||'built-in')})</span>
        ${p.confidence ? `<span style="margin-left:8px;">confidence: ${p.confidence}</span>` : ''}
      </div>`).join('');
    return `<div class="plugin-card">
      <div class="plugin-name">${esc(c.id)}</div>
      <div class="plugin-meta">available: ${c.available}</div>
      ${provs || '<div style="font-size:0.72rem;color:var(--text-muted);">No providers</div>'}
    </div>`;
  }).join('') || '<div class="text-dim">No processors match filter</div>';
}

// ============================================================
// Plugins View
// ============================================================
async function loadPlugins() {
  try {
    const caps = await api('/api/capabilities');
    // Group by provider (processor) to show as plugins
    const provs = {};
    for (const c of caps.capabilities||[]) {
      for (const p of c.providers||[]) {
        const key = p.processor || 'built-in';
        if (!provs[key]) provs[key] = [];
        provs[key].push(c.id);
      }
    }
    const html = Object.entries(provs).map(([proc, capsList]) =>
      `<div class="plugin-card">
        <div class="plugin-name">${esc(proc)}</div>
        <div class="plugin-meta">${capsList.length} capabilities</div>
        <div style="font-size:0.72rem;color:var(--text-muted);margin-top:4px;">
          ${capsList.slice(0,8).map(c => `<span style="background:var(--surface3);padding:1px 6px;border-radius:4px;margin:2px;display:inline-block;font-size:0.65rem;">${esc(c)}</span>`).join('')}
          ${capsList.length > 8 ? `<span style="color:var(--text-muted);font-size:0.65rem;">+${capsList.length-8} more</span>` : ''}
        </div>
      </div>`
    ).join('') || '<div class="text-dim">No plugins loaded</div>';
    $('pluginList').innerHTML = html;
  } catch(e) {
    $('pluginList').innerHTML = `<div class="error-msg">${e.message}</div>`;
  }
}

async function reloadPlugins() {
  $('pluginStatus').textContent = 'Reloading...';
  try {
    await api('/api/plugins/reload', {method: 'POST'});
    $('pluginStatus').textContent = '✓ Reloaded';
    setTimeout(loadPlugins, 500);
  } catch(e) {
    $('pluginStatus').textContent = '✗ ' + e.message;
  }
}

// ============================================================
// Events View
// ============================================================
async function loadEvents() {
  const limit = parseInt($('evtLimit').value) || 50;
  const fromSeq = evtPage * limit + 1;
  const filter = $('evtFilter').value.trim();
  let url = `/api/signals/history?from_seq=${fromSeq}&limit=${limit}`;
  if (filter) url += '&event_type=' + encodeURIComponent(filter);

  $('eventTable').innerHTML = '<div class="loading">Loading...</div>';
  try {
    const data = await api(url);
    const events = data.signals || [];
    $('eventTable').innerHTML =
      `<table>
        <thead><tr><th>Seq</th><th>Type</th><th>Time</th><th style="width:60%;">Payload</th></tr></thead>
        <tbody>
          ${events.map(e => `<tr>
            <td style="color:var(--text-muted);">${e.seq || '-'}</td>
            <td style="color:var(--accent);max-width:250px;overflow:hidden;text-overflow:ellipsis;">${esc(e.event_type||'')}</td>
            <td style="color:var(--text-dim);white-space:nowrap;">${e.time ? new Date(e.time).toLocaleString() : '-'}</td>
            <td style="font-size:0.65rem;color:var(--text-dim);max-width:400px;overflow:hidden;text-overflow:ellipsis;">
              ${e.data ? esc(JSON.stringify(e.data).slice(0,120)) : '-'}
              ${e.data && JSON.stringify(e.data).length > 120 ? '...' : ''}
            </td>
          </tr>`).join('')}
          ${events.length === 0 ? '<tr><td colspan="4" style="text-align:center;color:var(--text-dim);padding:20px;">No events found</td></tr>' : ''}
        </tbody>
      </table>`;
    $('evtPageInfo').textContent = `Page ${evtPage+1} (${events.length} events)`;
  } catch(e) {
    $('eventTable').innerHTML = `<div class="error-msg">${e.message}</div>`;
  }
}

function nextEvents() { evtPage++; loadEvents(); }
function prevEvents() { if (evtPage > 0) { evtPage--; loadEvents(); } }

// ============================================================
// Metrics View
// ============================================================
async function loadMetrics() {
  try {
    const [stats, obsSignals] = await Promise.all([
      api('/api/stats'),
      api('/api/observability/signals').catch(() => ({}))
    ]);

    // Signal distribution bars
    const sigDist = typeof obsSignals === 'object' ? Object.entries(obsSignals).filter(([k]) => k !== 'total').slice(0,20) : [];
    const maxSigCount = Math.max(1, ...sigDist.map(([,v]) => typeof v === 'number' ? v : 0));
    $('sigDistBars').innerHTML = sigDist.map(([k, v]) => {
      const val = typeof v === 'number' ? v : (typeof v === 'object' && v !== null ? JSON.stringify(v).length : 0);
      return `<div class="metric-bar">
        <div class="label">${esc(k)}</div>
        <div class="bar-wrap"><div class="bar-fill" style="width:${val/maxSigCount*100}%;"></div></div>
        <div class="bar-value">${fmt(val)}</div>
      </div>`;
    }).join('') || '<div class="text-dim">No signal metrics yet</div>';

    // Processor latency bars
    try {
      const procMetrics = await api('/api/observability/processors');
      const procs = Object.entries(procMetrics).filter(([k]) => k !== 'total');
      const maxLat = Math.max(1, ...procs.map(([,v]) => typeof v === 'number' ? v : 0));
      $('procLatencyBars').innerHTML = procs.slice(0,20).map(([k, v]) => {
        const val = typeof v === 'number' ? v : 0;
        const pct = val / maxLat * 100;
        return `<div class="metric-bar">
          <div class="label">${esc(k)} <span style="color:var(--text-muted);float:right;">${val.toFixed(0)}ns</span></div>
          <div class="bar-wrap"><div class="bar-fill ${pct > 80 ? 'warn' : ''}" style="width:${pct}%;"></div></div>
        </div>`;
      }).join('') || '<div class="text-dim">No processor metrics yet</div>';
    } catch(e) {
      $('procLatencyBars').innerHTML = '<div class="text-dim">No processor metrics yet</div>';
    }

    // Field sizes
    try {
      const fieldsData = await api('/api/stats');
      const flds = fieldsData.field_names || [];
      $('fieldSizeBars').innerHTML = flds.map((f, i) =>
        `<div class="metric-bar">
          <div class="label">${esc(f)}</div>
          <div class="bar-wrap"><div class="bar-fill" style="width:${Math.max(10, 100 - i*10)}%;"></div></div>
          <div class="bar-value">active</div>
        </div>`
      ).join('') || '<div class="text-dim">No field data</div>';
    } catch(e) {
      $('fieldSizeBars').innerHTML = '<div class="text-dim">No field data</div>';
    }

    // Raw metrics
    $('metricsRaw').innerHTML = `<pre style="font-size:0.68rem;color:var(--text-dim);overflow-x:auto;max-height:300px;">${esc(JSON.stringify({stats: stats, signals: obsSignals}, null, 2))}</pre>`;

  } catch(e) {
    $('metricsRaw').innerHTML = `<div class="error-msg">${e.message}</div>`;
  }
}

// ============================================================
// Initialization
// ============================================================
refreshOverview();
loadFields();
loadInjectTypes();
connectSSE();
setInterval(refreshOverview, 5000);
setInterval(loadMetrics, 15000);
</script>
</body>
</html>"#
}
