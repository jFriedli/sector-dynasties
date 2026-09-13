//! Thin wasm-bindgen bridge. Contains no simulation rules of its own: it
//! only translates between JS-friendly values and `sim-core` types. If a
//! decision belongs here instead of in `sim-core`, that's a bug: the UI
//! must not carry authoritative game rules.

use sim_core::world::{GovernmentComponent, PolicyDirection};
use sim_core::SimState as CoreState;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SimHandle {
    inner: CoreState,
}

#[wasm_bindgen]
impl SimHandle {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64) -> SimHandle {
        SimHandle {
            inner: CoreState::new(seed),
        }
    }

    pub fn step_days(&mut self, days: u32) {
        self.inner.step_days(days);
    }

    #[wasm_bindgen(js_name = resolveEvent)]
    pub fn resolve_event(&mut self, pending_id: u32, choice_key: &str) -> Result<(), JsError> {
        self.inner
            .resolve_event(pending_id, choice_key)
            .map_err(|e| JsError::new(&format!("{e:?}")))
    }

    #[wasm_bindgen(js_name = lobbyPolicy)]
    pub fn lobby_policy(
        &mut self,
        country_id: u32,
        component: &str,
        direction: &str,
    ) -> Result<String, JsError> {
        let component = component
            .parse::<GovernmentComponent>()
            .map_err(|_| JsError::new("unknown government component"))?;
        let direction = direction
            .parse::<PolicyDirection>()
            .map_err(|_| JsError::new("unknown policy direction"))?;

        let outcome = self
            .inner
            .lobby_policy(country_id, component, direction)
            .map_err(|e| JsError::new(&format!("{e:?}")))?;
        serde_json::to_string(&outcome).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Returns a JSON-serialized `StateSummary`. Kept as a JSON string
    /// (rather than a `JsValue` object) so the wire format is identical to
    /// what `sim-cli --json` and native tests already exercise.
    pub fn summary_json(&self) -> String {
        serde_json::to_string(&self.inner.summary()).expect("StateSummary always serializes")
    }

    pub fn to_json(&self) -> String {
        self.inner.to_json()
    }

    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<SimHandle, JsError> {
        CoreState::from_json(json)
            .map(|inner| SimHandle { inner })
            .map_err(|e| JsError::new(&e.to_string()))
    }
}
