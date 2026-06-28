//! Background HTTP worker. A single thread owns the blocking `Client` and
//! processes `Req`s FIFO, emitting `Resp`s. Responses always arrive in request
//! order — no staleness races to reconcile in the UI.
//!
//! Modeled on curlyos-tui's worker.rs with reqwest::blocking for the Noesis API.

use crate::tui::api::*;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub enum Req {
    Dashboard,
    Health,
    Stats,
    SignalTypes,
    SignalHistory { limit: usize, field: Option<String> },
    ObservabilityOverview,
    ProcessorMetrics,
    SignalMetrics,
    CascadeTrace,
    Capabilities,
    Plugins,
    Config,
    /// Fetch deep field detail by name
    FieldDetail(String),
    /// Memories / episodes / identity
    Memories,
    Episodes,
    Identity,
    /// Signal stats
    SignalStats,
    /// Ingest text
    Ingest(String),
    /// Inject a signal
    Inject { signal_type: String, payload: serde_json::Value },
}

pub enum Resp {
    Dashboard(Box<DashboardData>),
    Health(Box<Health>),
    Stats(Box<Stats>),
    SignalTypes(Vec<SignalTypeItem>),
    SignalHistory(Vec<SignalHistoryEntry>),
    ObservabilityOverview(Box<Observability>),
    ProcessorMetrics(Vec<ProcessorMetric>),
    SignalMetrics(Box<SignalMetricsData>),
    CascadeTrace(Box<CascadeTraceData>),
    Capabilities(Vec<Capability>),
    Plugins(Vec<PluginSummary>),
    Config(Box<SystemConfig>),
    FieldDetail(String, serde_json::Value),
    ActionOk { msg: String, refresh: bool },
    Error(String),
}

pub fn spawn(client: Client, rx: Receiver<Req>, tx: Sender<Resp>) {
    thread::spawn(move || {
        for req in rx {
            let resp = handle(&client, req);
            if tx.send(resp).is_err() {
                break; // UI gone
            }
        }
    });
}

fn handle(c: &Client, req: Req) -> Resp {
    match req {
        Req::Dashboard => match (c.stats(), c.health(), c.observability()) {
            (Ok(s), Ok(h), Ok(o)) => Resp::Dashboard(Box::new(DashboardData { stats: s, health: h, observability: o })),
            (Err(e), ..) | (_, Err(e), _) | (.., Err(e)) => Resp::Error(format!("dashboard: {e}")),
        },
        Req::Health => wrap(c.health(), |h| Resp::Health(Box::new(h))),
        Req::Stats => wrap(c.stats(), |s| Resp::Stats(Box::new(s))),
        Req::SignalTypes => wrap(c.signal_types(), Resp::SignalTypes),
        Req::SignalHistory { limit, field } => wrap(c.signal_history(limit, field.as_deref()), Resp::SignalHistory),
        Req::ObservabilityOverview => wrap(c.observability(), |o| Resp::ObservabilityOverview(Box::new(o))),
        Req::ProcessorMetrics => wrap(c.processor_metrics(), Resp::ProcessorMetrics),
        Req::SignalMetrics => wrap(c.signal_metrics(), |m| Resp::SignalMetrics(Box::new(m))),
        Req::CascadeTrace => wrap(c.cascade_trace(), |t| Resp::CascadeTrace(Box::new(t))),
        Req::Capabilities => wrap(c.capabilities(), Resp::Capabilities),
        Req::Plugins => wrap(c.plugins(), Resp::Plugins),
        Req::Config => wrap(c.config(), |c| Resp::Config(Box::new(c))),
        Req::FieldDetail(name) => match c.detail_for(&name) {
            Ok(v) => Resp::FieldDetail(name, v),
            Err(e) => Resp::Error(format!("detail {name}: {e}")),
        },
        Req::Memories => wrap(c.memory_state(), |v| Resp::FieldDetail("memories".into(), v)),
        Req::Episodes => wrap(c.episodes(), |v| Resp::FieldDetail("episodes".into(), v)),
        Req::Identity => wrap(c.identity(), |v| Resp::FieldDetail("identity_facts".into(), v)),
        Req::SignalStats => wrap(c.signal_stats(), |v| Resp::FieldDetail("signal_stats".into(), v)),
        Req::Ingest(text) => match c.ingest(&text) {
            Ok(_) => Resp::ActionOk { msg: "Ingested".into(), refresh: true },
            Err(e) => Resp::Error(format!("ingest: {e}")),
        },
        Req::Inject { signal_type, payload } => match c.inject_signal(&signal_type, payload) {
            Ok(_) => Resp::ActionOk { msg: format!("Injected {signal_type}"), refresh: true },
            Err(e) => Resp::Error(format!("inject: {e}")),
        },
    }
}

fn wrap<T>(r: anyhow::Result<T>, ok: impl FnOnce(T) -> Resp) -> Resp {
    match r {
        Ok(v) => ok(v),
        Err(e) => Resp::Error(format!("{e}")),
    }
}
