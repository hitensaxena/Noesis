//! Built-in Noesis plugin — registers all existing processors, signals, and capabilities.
//!
//! This is the primary plugin that makes the cognitive system functional.
//! All 11 processors are registered here along with their declared capabilities.

use async_trait::async_trait;

use crate::kernel::capabilities::Capability;
use crate::kernel::plugin::Plugin;
use crate::kernel::signal::SignalType;
use crate::processor::processor::Processor;
use crate::signals::types;

/// The built-in Noesis plugin — provides core cognition.
///
/// Registers all existing processors and their canonical capabilities.
/// This is loaded at startup in main.rs before the cascade loop starts.
pub struct NoesisPlugin {
    capabilities: Vec<Capability>,
    signals: Vec<(SignalType, &'static str)>,
}

impl NoesisPlugin {
    pub fn new() -> Self {
        let mut plugin = Self {
            capabilities: Vec::new(),
            signals: Vec::new(),
        };
        plugin.register_builtins();
        plugin
    }

    fn register_builtins(&mut self) {
        // ---- Capabilities ----
        self.register_capability("ingestion", "Episode Ingestion", "Records raw experiences as structured episodes", 0.9, "episode");
        self.register_capability("extraction", "Triple Extraction", "Extracts entity-relation triples from episode content", 0.8, "extraction");
        self.register_capability("resolution", "Entity Resolution", "Resolves extracted triples into graph entities", 0.8, "resolution");
        self.register_capability("consolidation", "Memory Consolidation", "Summarizes and consolidates episodes into semantic memories", 0.7, "consolidation");
        self.register_capability("belief_formation", "Belief Formation", "Forms beliefs from consolidated memories", 0.8, "belief");
        self.register_capability("self_modeling", "Self Modeling", "Maintains a coherent self-model from beliefs and traits", 0.7, "identity");
        self.register_capability("goal_generation", "Goal Generation", "Generates goals from identity and values", 0.8, "goal");
        self.register_capability("attention", "Attention Management", "Manages focus stack and salience signals", 0.9, "attention");
        self.register_capability("curiosity", "Curiosity Detection", "Detects knowledge gaps from accumulated episodes", 0.7, "curiosity");
        self.register_capability("narrative", "Narrative Generation", "Generates coherent narratives from episode clusters", 0.7, "narrative");
        self.register_capability("reflection", "Reflection", "Reflects on recent experiences for insight extraction", 0.6, "reflection");
        self.register_capability("observation", "Signal Observation", "Observes and records every signal transition in the system", 1.0, "observer");
        self.register_capability("metacognition", "Metacognitive Insight", "Analyzes signal patterns for metacognitive insights", 0.6, "metacognition");
        self.register_capability("mood", "Mood Estimation", "Estimates cognitive mood from signal pattern ratios", 0.5, "mood");
        self.register_capability("context", "Context Assembly", "Assembles reasoning context windows from recent signals", 0.6, "context");
        self.register_capability("epistemic", "Epistemic Classification", "Classifies signals into epistemic categories", 0.7, "epistemic");
        self.register_capability("health", "Health Monitoring", "Monitors subsystem health and signal throughput", 0.8, "health");
        self.register_capability("value_extraction", "Value Extraction", "Extracts values from decision patterns", 0.5, "values");
        self.register_capability("principle_distillation", "Principle Distillation", "Derives principles from values and decisions", 0.4, "principles");
        self.register_capability("planning", "Plan Decomposition", "Decomposes goals into actionable plans", 0.5, "planning");
        self.register_capability("project_management", "Project Management", "Creates projects from goals", 0.5, "project");
        self.register_capability("task_decomposition", "Task Decomposition", "Breaks projects into tasks", 0.5, "task");
        self.register_capability("execution", "Task Execution", "Executes tasks and tracks progress", 0.4, "execution");
        self.register_capability("evaluation", "Outcome Evaluation", "Evaluates execution outcomes", 0.5, "evaluation");
        self.register_capability("risk_assessment", "Risk Assessment", "Assesses risks in plans", 0.4, "risk");
        self.register_capability("recovery", "Failure Recovery", "Initiates recovery from failures", 0.3, "recovery");
        self.register_capability("world_modeling", "World Modeling", "Builds causal models from episode patterns", 0.5, "world_model");
        self.register_capability("assumption_testing", "Assumption Testing", "Tests assumptions underlying strategies", 0.4, "assumption");
        self.register_capability("scenario_analysis", "Scenario Analysis", "Generates what-if scenarios from decisions", 0.5, "scenario");
        self.register_capability("counterfactual", "Counterfactual Reasoning", "Explores alternative outcomes", 0.4, "counterfactual");
        self.register_capability("forecasting", "Forecasting", "Predicts goal completion outcomes", 0.4, "forecast");
        self.register_capability("simulation_risk", "Simulation Risk", "Assesses risk from simulated scenarios", 0.4, "simulation_risk");
        self.register_capability("confidence", "Confidence Estimation", "Tracks average signal activation and salience for confidence scoring", 0.6, "confidence");

        // ---- Declared signals ----
        self.signals.push((types::INGEST_REQUEST, "Raw text input"));
        self.signals.push((types::EPISODE_RECORDED, "Structured episodic record"));
        self.signals.push((types::FACT_EXTRACTED, "Extracted factual statement"));
        self.signals.push((types::MEMORY_CONSOLIDATED, "Consolidated memory summary"));
        self.signals.push((types::PATTERN_DETECTED, "Recurring pattern across memories"));
        self.signals.push((types::BELIEF_CHANGED, "Belief created/updated/invalidated"));
        self.signals.push((types::TRAIT_DETECTED, "Personality trait detected"));
        self.signals.push((types::IDENTITY_UPDATED, "Self-model version changed"));
        self.signals.push((types::GOAL_CREATED, "New goal created"));
        self.signals.push((types::GOAL_COMPLETED, "Goal completed"));
        self.signals.push((types::DECISION_EVALUATED, "Decision outcome evaluated"));
        self.signals.push((types::ATTENTION_SHIFTED, "Focus shifted"));
        self.signals.push((types::CURIOSITY_DETECTED, "Knowledge gap detected"));
        self.signals.push((types::NARRATIVE_GENERATED, "Narrative chapter generated"));
        self.signals.push((types::ENTITY_CREATED, "Knowledge graph entity created"));
        self.signals.push((types::EDGE_CREATED, "Knowledge graph edge created"));
        self.signals.push((types::TRIPLES_EXTRACTED, "Entity-relation triples extracted"));
        self.signals.push((types::OBSERVER_TRANSITION_DETECTED, "A state transition was observed"));
        self.signals.push((types::METACOGNITION_INSIGHT, "A metacognitive insight was generated"));
        self.signals.push((types::MOOD_ESTIMATED, "Cognitive mood estimate"));
        self.signals.push((types::CONTEXT_ASSEMBLED, "Reasoning context window assembled"));
        self.signals.push((types::EPISTEMICS_CLASSIFIED, "An epistemic classification was made"));
        self.signals.push((types::HEALTH_STATUS_CHANGED, "Subsystem health status changed"));
        self.signals.push((types::VALUES_REFINED, "A value was refined from decision patterns"));
        self.signals.push((types::PRINCIPLES_DERIVED, "A principle was derived from values"));
        self.signals.push((types::PLANNING_PLAN_READY, "A plan was decomposed from a goal"));
        self.signals.push((types::PROJECT_CREATED, "A project was created from a goal"));
        self.signals.push((types::TASK_CREATED, "A task was created from a project"));
        self.signals.push((types::EXECUTION_STARTED, "Execution of a task started"));
        self.signals.push((types::EVALUATION_COMPLETED, "An execution was evaluated"));
        self.signals.push((types::RISK_ASSESSED, "A risk assessment was completed"));
        self.signals.push((types::RECOVERY_STARTED, "Recovery process initiated"));
        self.signals.push((types::WORLD_MODEL_UPDATED, "World model updated from episode data"));
        self.signals.push((types::ASSUMPTION_TESTED, "An assumption was tested against evidence"));
        self.signals.push((types::SCENARIO_READY, "A what-if scenario was generated"));
        self.signals.push((types::COUNTERFACTUAL_READY, "A counterfactual analysis was completed"));
        self.signals.push((types::FORECAST_READY, "A forecast prediction was generated"));
        self.signals.push((types::SIMULATION_RISK_ASSESSED, "A simulation-based risk assessment completed"));
        self.signals.push((types::CONCLUSION_READY, "A reasoning conclusion was drawn"));
        self.signals.push((types::MENTAL_MODEL_UPDATED, "A mental model was updated"));
        self.signals.push((types::ANALOGY_DETECTED, "An analogy between domains was detected"));
        self.signals.push((types::SYNTHESIS_READY, "Synthesized knowledge was produced"));
        self.signals.push((types::CONCEPT_FORMED, "An abstract concept was formed"));
        self.signals.push((types::HYPOTHESIS_GENERATED, "A hypothesis was generated from pattern data"));
        self.signals.push((types::PRIORITY_REORDERED, "Goal priorities were reordered"));
        self.signals.push((types::STRATEGY_UPDATED, "Strategic plan was updated"));
        self.signals.push((types::OPPORTUNITY_DETECTED, "A new opportunity was detected"));
        self.register_capability("deductive_reasoning", "Deductive Reasoning", "Draws conclusions from epistemic classifications", 0.5, "reasoning");
        self.register_capability("mental_modeling", "Mental Modeling", "Builds mental models from conclusions", 0.4, "mental_model");
        self.register_capability("decision_making", "Decision Making", "Makes decisions based on reasoning chains", 0.5, "decision");
        self.register_capability("hypothesis_generation", "Hypothesis Generation", "Generates hypotheses from patterns", 0.4, "hypothesis");
        self.register_capability("analogy_mapping", "Analogy Mapping", "Finds analogies between domains", 0.3, "analogy");
        self.register_capability("knowledge_synthesis", "Knowledge Synthesis", "Synthesizes related facts into knowledge", 0.4, "synthesis");
        self.register_capability("concept_formation", "Concept Formation", "Forms abstract concepts from patterns", 0.3, "concept");
        self.register_capability("pattern_detection", "Pattern Detection", "Detects recurring patterns in observations", 0.6, "pattern");
        self.register_capability("open_loop_tracking", "Open Loop Tracking", "Tracks unresolved curiosities and goals", 0.4, "open_loops");
        self.register_capability("memory_retrieval", "Memory Retrieval", "Retrieves relevant episodes from memory", 0.5, "retrieval");
        self.register_capability("embedding_indexing", "Embedding Indexing", "Indexes entities for embedding search", 0.3, "indexing");
        self.register_capability("memory_decay", "Memory Decay", "Applies time-based decay to old memories", 0.4, "decay");
        self.register_capability("deduplication", "Deduplication", "Detects and resolves duplicate content", 0.5, "dedup");
        self.register_capability("priority_management", "Priority Management", "Reorders goals by urgency and importance", 0.5, "priority");
        self.register_capability("strategic_planning", "Strategic Planning", "Updates strategy on slow cognitive beats", 0.4, "strategy");
        self.register_capability("opportunity_discovery", "Opportunity Discovery", "Detects opportunities from curiosity gaps", 0.3, "opportunity");
    }

    fn register_capability(&mut self, id: &str, name: &str, description: &str, confidence: f32, processor: &str) {
        self.capabilities.push(Capability {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            confidence,
            processor: processor.to_string(),
        });
    }
}

#[async_trait]
impl Plugin for NoesisPlugin {
    fn name(&self) -> &str {
        "noesis.core"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "Core cognitive system — memory, identity, agency, awareness, reasoning, and simulation"
    }

    fn processors(&self) -> Vec<Box<dyn Processor + Send>> {
        vec![
            Box::new(crate::processors::episode::EpisodeProcessor::new()),
            Box::new(crate::processors::extraction::ExtractionProcessor::new()),
            Box::new(crate::processors::resolution::EntityResolutionProcessor::new()),
            Box::new(crate::processors::consolidation::ConsolidationProcessor::new()),
            Box::new(crate::processors::belief::BeliefProcessor::new()),
            Box::new(crate::processors::identity::IdentityProcessor::new()),
            Box::new(crate::processors::goal::GoalProcessor::new()),
            Box::new(crate::processors::attention::AttentionProcessor::new()),
            Box::new(crate::processors::curiosity::CuriosityProcessor::new()),
            Box::new(crate::processors::narrative::NarrativeProcessor::new()),
            Box::new(crate::processors::reflection::ReflectionProcessor::new()),
            Box::new(crate::processors::observer::ObserverProcessor::new()),
            Box::new(crate::processors::metacognition::MetaProcessor::new()),
            Box::new(crate::processors::mood::MoodProcessor::new()),
            Box::new(crate::processors::context::ContextConstructor::new()),
            Box::new(crate::processors::epistemic::EpistemicClassifier::new()),
            Box::new(crate::processors::health::HealthChecker::new()),
            Box::new(crate::processors::values::ValueExtractor::new()),
            Box::new(crate::processors::principles::PrincipleDistiller::new()),
            Box::new(crate::processors::planning::PlanDecomposer::new()),
            Box::new(crate::processors::project::ProjectProcessor::new()),
            Box::new(crate::processors::task::TaskProcessor::new()),
            Box::new(crate::processors::execution::ExecutionProcessor::new()),
            Box::new(crate::processors::evaluation::EvaluationProcessor::new()),
            Box::new(crate::processors::risk::RiskProcessor::new()),
            Box::new(crate::processors::recovery::RecoveryProcessor::new()),
            Box::new(crate::processors::confidence::ConfidenceEstimator::new()),
            Box::new(crate::processors::world_model::WorldModelProcessor::new()),
            Box::new(crate::processors::assumption::AssumptionProcessor::new()),
            Box::new(crate::processors::scenario::ScenarioProcessor::new()),
            Box::new(crate::processors::counterfactual::CounterfactualProcessor::new()),
            Box::new(crate::processors::forecast::ForecastingProcessor::new()),
            Box::new(crate::processors::simulation_risk::SimulationRiskProcessor::new()),
            Box::new(crate::processors::reasoning::ReasoningProcessor::new()),
            Box::new(crate::processors::mental_model::MentalModelProcessor::new()),
            Box::new(crate::processors::decision::DecisionProcessor::new()),
            Box::new(crate::processors::hypothesis::HypothesisProcessor::new()),
            Box::new(crate::processors::analogy::AnalogyProcessor::new()),
            Box::new(crate::processors::synthesis::SynthesisProcessor::new()),
            Box::new(crate::processors::concept::ConceptProcessor::new()),
            Box::new(crate::processors::pattern::PatternProcessor::new()),
            Box::new(crate::processors::open_loops::OpenLoopsProcessor::new()),
            Box::new(crate::processors::retrieval::RetrievalProcessor::new()),
            Box::new(crate::processors::indexing::IndexingProcessor::new()),
            Box::new(crate::processors::decay::DecayProcessor::new()),
            Box::new(crate::processors::dedup::DedupProcessor::new()),
            Box::new(crate::processors::priority::PriorityProcessor::new()),
            Box::new(crate::processors::strategy::StrategyProcessor::new()),
            Box::new(crate::processors::opportunity::OpportunityProcessor::new()),
        ]
    }

    fn signals(&self) -> Vec<(SignalType, &str)> {
        self.signals.clone()
    }

    fn capabilities(&self) -> Vec<Capability> {
        // We can't return references to self.capabilities because we need to keep them
        // For built-in plugin, we recreate on each call (small overhead, acceptable)
        self.capabilities.clone()
    }

    fn config_defaults(&self) -> Vec<(&str, serde_json::Value)> {
        vec![]
    }
}

impl Default for NoesisPlugin {
    fn default() -> Self {
        Self::new()
    }
}
