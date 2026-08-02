/// Module resolution for multi-file Aero projects.
///
/// Resolves `mod foo;` declarations to file system paths:
///   - `mod foo;` looks for `foo.aero` in the same directory, or `foo/mod.aero`
///
/// Nested module declarations are rejected by the shared collection boundary until
/// module-graph, namespace, and cycle semantics are specified.
use crate::ast::{AstNode, Statement};
use crate::{lexer, parser};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a resolved module with its file path and parsed contents.
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    /// The module name as declared in `mod <name>;`
    pub name: String,
    /// File-system path to the source file.
    pub file_path: PathBuf,
    /// Source code loaded from the file
    pub source: String,
}

/// A direct module whose exact source has passed strict lexing and fatal parsing.
#[derive(Debug, Clone)]
pub(crate) struct ParsedDirectModule {
    /// The root-level declaration name.
    pub(crate) name: String,
    /// Stable resolver candidate using `/`, independent of the host path syntax.
    pub(crate) candidate: String,
    /// File-system path used for located lexer/parser diagnostics.
    pub(crate) file_path: PathBuf,
    /// Exact UTF-8 source bytes represented as a Rust string.
    pub(crate) source: String,
    /// Parsed top-level nodes appended under the current flattened compatibility model.
    pub(crate) ast: Vec<AstNode>,
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

/// Resolve, read, strictly lex, and fatally parse every root-level direct module.
///
/// `None` is the source-only public API boundary: a module declaration cannot be
/// interpreted without an entry-file directory, so it fails rather than consulting
/// the process working directory.
pub(crate) fn collect_direct_modules(
    root_ast: &[AstNode],
    entry_file: Option<&str>,
) -> Result<Vec<ParsedDirectModule>, String> {
    let declarations = root_ast.iter().filter_map(|node| match node {
        AstNode::Statement(Statement::ModDecl { name, .. }) => Some(name.as_str()),
        _ => None,
    });

    let Some(entry_file) = entry_file else {
        if let Some(name) = declarations.into_iter().next() {
            return Err(format!(
                "Module resolution failed for `{name}`: module declarations require an entry-file context"
            ));
        }
        return Ok(Vec::new());
    };

    let mut resolver = ModuleResolver::new(entry_file);
    let mut modules = Vec::new();
    for name in declarations {
        let resolved = resolver
            .resolve(name)
            .map_err(|error| format!("Module resolution failed for `{name}`: {error}"))?;
        let candidate = if resolved.file_path == resolver.base_dir.join(format!("{name}.aero")) {
            format!("{name}.aero")
        } else {
            format!("{name}/mod.aero")
        };
        let module_filename = resolved.file_path.to_string_lossy().into_owned();
        let tokens = lexer::try_tokenize_with_locations(&resolved.source, Some(module_filename))
            .map_err(|error| format!("Lex error: {error}"))?;
        let ast = parser::parse_with_locations(tokens)
            .map_err(|error| format!("Parse error: {error}"))?;

        if let Some(nested_name) = ast.iter().find_map(|node| match node {
            AstNode::Statement(Statement::ModDecl { name, .. }) => Some(name.as_str()),
            _ => None,
        }) {
            return Err(format!(
                "Module resolution failed for `{nested_name}`: nested module declarations are not supported by the current compiler"
            ));
        }

        modules.push(ParsedDirectModule {
            name: resolved.name,
            candidate,
            file_path: resolved.file_path,
            source: resolved.source,
            ast,
        });
    }

    Ok(modules)
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
