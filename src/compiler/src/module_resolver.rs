/// Module resolution for multi-file Aero projects.
///
/// Resolves `mod foo;` declarations to file system paths:
///   - `mod foo;` looks for `foo.aero` in the same directory, or `foo/mod.aero`
///
/// Maintains a registry of resolved modules to detect and prevent circular imports.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a resolved module with its file path and parsed contents.
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    /// The module name as declared in `mod <name>;`
    pub name: String,
    /// Absolute path to the source file
    pub file_path: PathBuf,
    /// Source code loaded from the file
    pub source: String,
}

/// Module resolver that maps `mod` declarations to file system paths.
pub struct ModuleResolver {
    /// Base directory for the root module (directory of the entry file)
    base_dir: PathBuf,
    /// Cache of already-resolved modules: module name -> resolved module
    resolved: HashMap<String, ResolvedModule>,
}

impl ModuleResolver {
    /// Create a new resolver rooted at the directory containing `entry_file`.
    pub fn new(entry_file: &str) -> Self {
        let entry_path = Path::new(entry_file);
        let base_dir = entry_path.parent().unwrap_or(Path::new(".")).to_path_buf();

        ModuleResolver {
            base_dir,
            resolved: HashMap::new(),
        }
    }

    /// Resolve a `mod foo;` declaration to a file path and load its source.
    ///
    /// Search order:
    ///   1. `<base_dir>/<name>.aero`
    ///   2. `<base_dir>/<name>/mod.aero`
    pub fn resolve(&mut self, module_name: &str) -> Result<ResolvedModule, String> {
        // Return cached module if already resolved
        if let Some(resolved) = self.resolved.get(module_name) {
            return Ok(resolved.clone());
        }

        // Strategy 1: <base_dir>/<name>.aero
        let file_path = self.base_dir.join(format!("{}.aero", module_name));
        if file_path.exists() {
            return self.load_module(module_name, &file_path);
        }

        // Strategy 2: <base_dir>/<name>/mod.aero
        let dir_path = self.base_dir.join(module_name).join("mod.aero");
        if dir_path.exists() {
            return self.load_module(module_name, &dir_path);
        }

        Err(format!(
            "Cannot find module `{}`. Looked for:\n  - {}\n  - {}",
            module_name,
            file_path.display(),
            dir_path.display()
        ))
    }

    /// Load a module file from disk and cache the result.
    fn load_module(&mut self, name: &str, path: &Path) -> Result<ResolvedModule, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|err| format!("Could not read module file `{}`: {}", path.display(), err))?;

        let resolved = ResolvedModule {
            name: name.to_string(),
            file_path: path.to_path_buf(),
            source,
        };

        self.resolved.insert(name.to_string(), resolved.clone());
        Ok(resolved)
    }

    /// Get all resolved modules.
    pub fn resolved_modules(&self) -> &HashMap<String, ResolvedModule> {
        &self.resolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolver_base_dir() {
        let resolver = ModuleResolver::new("examples/hello.aero");
        assert_eq!(resolver.base_dir, PathBuf::from("examples"));
    }

    #[test]
    fn resolver_missing_module() {
        let mut resolver = ModuleResolver::new("nonexistent/main.aero");
        let result = resolver.resolve("does_not_exist");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Cannot find module `does_not_exist`")
        );
    }
}
