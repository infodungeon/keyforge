// libs/keyforge-physics/src/context.rs

use crate::kernel::EngineContext;
use keyforge_model::config::EngineConfig;
use keyforge_model::{Corpus, CostModel, Keyboard, Rubric};
use std::marker::PhantomData;
use std::sync::Arc;

/// Marker trait for scoring context states.
pub trait ScoringState: Send + Sync + std::fmt::Debug {}

/// State representing a context that has not yet been compiled into a high-performance engine.
#[derive(Debug, Clone)]
pub struct Uncompiled {
    /// Physical keyboard definition.
    pub keyboard: Arc<Keyboard>,
    /// Language frequency data.
    pub corpus: Arc<Corpus>,
    /// Scoring weights and penalties.
    pub rubric: Arc<Rubric>,
    /// Biomechanical cost model.
    pub cost_model: Arc<CostModel>,
    /// Engine hardware optimization parameters.
    pub engine_config: EngineConfig,
}

/// State representing a context that has been compiled and is ready for high-performance scoring.
#[derive(Debug, Clone)]
pub struct Compiled {
    /// The underlying compiled context data.
    pub inner: EngineContext,
}

impl ScoringState for Uncompiled {}
impl ScoringState for Compiled {}

/// A unified context for the scoring engine, using typestate to enforce the compilation lifecycle.
#[derive(Debug, Clone)]
pub struct ScoringContext<S: ScoringState> {
    pub(crate) state: S,
    _marker: PhantomData<S>,
}

impl ScoringContext<Uncompiled> {
    /// Creates a new uncompiled scoring context.
    #[must_use]
    pub fn new(
        keyboard: Arc<Keyboard>,
        corpus: Arc<Corpus>,
        rubric: Arc<Rubric>,
        cost_model: Arc<CostModel>,
        engine_config: EngineConfig,
    ) -> Self {
        Self {
            state: Uncompiled {
                keyboard,
                corpus,
                rubric,
                cost_model,
                engine_config,
            },
            _marker: PhantomData,
        }
    }

    /// Accesses the uncompiled state data.
    #[must_use]
    pub fn data(&self) -> &Uncompiled {
        &self.state
    }

    /// Compiles the context into a high-performance `ScoringContext<Compiled>`.
    ///
    /// # Errors
    /// Returns `PhysicsError` if compilation fails.
    pub fn compile(self) -> Result<ScoringContext<Compiled>, crate::error::PhysicsError> {
        let inner = crate::kernel::compiler::Compiler::compile(
            &self.state.keyboard,
            &self.state.corpus,
            &self.state.rubric,
            &self.state.cost_model,
        )?;

        Ok(ScoringContext {
            state: Compiled { inner },
            _marker: PhantomData,
        })
    }
}

impl ScoringContext<Compiled> {
    /// Accesses the compiled context data.
    #[must_use]
    pub fn data(&self) -> &EngineContext {
        &self.state.inner
    }
}
