//! Template Service
//!
//! Provides hot-reloading of templates in development mode.
//! - Debug: Uses minijinja to load templates from disk at runtime
//! - Release: Uses pre-compiled askama templates for performance

use minijinja::Environment;
use serde::Serialize;

/// Render a template with the given context
/// 
/// In debug mode, this reloads templates from disk on each request.
/// In release mode, askama handlers are used instead (see handlers/templates.rs).
#[cfg(debug_assertions)]
pub fn render_template<T: Serialize>(name: &str, context: T) -> Result<String, String> {
    // Create fresh environment each time to pick up file changes
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("templates"));
    
    let template = env.get_template(name).map_err(|e| format!("Template load error: {}", e))?;
    template.render(context).map_err(|e| format!("Template render error: {}", e))
}

/// In release mode, this function is not used - askama templates are used directly
#[cfg(not(debug_assertions))]
pub fn render_template<T: Serialize>(_name: &str, _context: T) -> Result<String, String> {
    Err("Runtime templates not available in release mode".to_string())
}

/// Check if we're in development mode (for conditional template loading)
pub fn is_dev_mode() -> bool {
    cfg!(debug_assertions)
}
