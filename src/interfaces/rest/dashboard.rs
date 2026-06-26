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
  </div>

  <!-- ====== VIEW: Fields ====== -->
  <div class="view" id="view-fields">
    <div class="panel-row">
      <div class="panel">
        <h2>Field Inspector</h2>
        <div id="fieldList"></div>
      </div>
      <div class="panel">
        <h2>Raw State</h2>
        <div id="fieldStateDetail">
          <div class="text-dim" style="font-size:0.8rem;">Click a field to inspect its state.</div>
        </div>
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
const API = '';
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
  const r = await fetch(API + url, opts);
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
    const total = ov.signals_processed && ov.signals_processed.total;
    $('ov-sys-total').textContent = fmt(total);
  } catch(e) {}

  // Field detail cards
  const fDefs = [
    {key:'memory', ep:'/api/memory/detail', stats:['episodic','semantic']},
    {key:'identity', ep:'/api/identity/detail', stats:['beliefs','traits']},
    {key:'agency', ep:'/api/agency/detail', stats:['goals','projects']},
    {key:'action', ep:'/api/awareness/detail', alt:'action'},
    {key:'awareness', ep:'/api/awareness/detail', stats:['attention','curiosity']},
    {key:'reasoning', ep:'/api/cognition/meta'},
    {key:'simulation', ep:'/api/simulation/detail', stats:['scenarios','forecasts']},
    {key:'graph', ep:'/api/graph', stats:['entity_count','relation_count']},
  ];
  for (const f of fDefs) {
    const statEl = $('ov-'+f.key);
    const subEl = $('ov-sub-'+f.key);
    try {
      const data = await api(f.ep);
      if (f.key === 'graph') {
        statEl.textContent = fmt(data.entity_count) + ' / ' + fmt(data.relation_count);
      } else if (f.key === 'reasoning') {
        statEl.textContent = '✓';
        subEl.textContent = 'metacognition active';
      } else if (f.alt) {
        statEl.textContent = '—';
        subEl.textContent = 'via awareness detail';
      } else {
        const parts = f.stats.map(s => {
          const v = data[s];
          if (typeof v === 'object' && v.count !== undefined) return v.count;
          if (typeof v === 'object' && v.total !== undefined) return v.total;
          if (typeof v === 'number') return v;
          return 0;
        });
        statEl.textContent = parts.join(' / ');
        subEl.textContent = f.stats.join(' / ');
      }
    } catch(e) {
      statEl.textContent = 'err';
      subEl.textContent = e.message;
    }
  }

  // Signal rate
  updateSignalRate();
}

// ============================================================
// Signal Rate Tracking
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
async function loadFields() {
  try {
    const stats = await api('/api/stats');
    fields = stats.field_names || [];
    const list = $('fieldList');
    list.innerHTML = fields.map(f =>
      `<div style="padding:8px 12px;border-bottom:var(--border);cursor:pointer;display:flex;justify-content:space-between;align-items:center;"
            onclick="inspectField('${f}')" onmouseover="this.style.background='var(--surface2)'"
            onmouseout="this.style.background=''">
        <span style="font-weight:500;">${f}</span>
        <span style="font-size:0.65rem;color:var(--text-dim);">click to inspect →</span>
      </div>`
    ).join('') || '<div class="text-dim">No fields registered</div>';
  } catch(e) {
    $('fieldList').innerHTML = `<div class="error-msg">${e.message}</div>`;
  }
}

async function inspectField(name) {
  const detail = $('fieldStateDetail');
  detail.innerHTML = '<div class="loading">Loading...</div>';
  const endpoints = {
    memory: '/api/memory/detail',
    identity: '/api/identity/detail',
    agency: '/api/agency/detail',
    action: '/api/awareness/detail',
    awareness: '/api/awareness/detail',
    reasoning: '/api/cognition/meta',
    simulation: '/api/simulation/detail',
    knowledge_graph: '/api/graph',
  };
  const ep = endpoints[name] || `/api/${name}/detail`;
  try {
    const data = await api(ep);
    detail.innerHTML = `<h3 style="color:var(--accent);margin-bottom:8px;">${name}</h3>
      <pre style="font-size:0.68rem;line-height:1.4;color:var(--text-dim);overflow-x:auto;">${esc(JSON.stringify(data, null, 2))}</pre>`;
  } catch(e) {
    detail.innerHTML = `<div class="error-msg">${e.message}</div>`;
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
