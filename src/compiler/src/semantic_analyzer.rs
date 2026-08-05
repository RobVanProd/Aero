use crate::ast::{
    AstNode, Block, ComparisonOp, Expression, LogicalOp, Parameter, Statement, UnaryOp,
};
use crate::binding_annotation::{
    BindingAnnotationDisposition, classify_binding_annotation, is_statically_empty_fixed_array,
    typed_empty_numeric_array_contract,
};
use crate::enum_match_contract::{EnumExecutionContext, EnumFunctionContract, EnumRegistry};
use crate::local_reference::{
    LocalReferenceDisposition, LocalReferenceSourceFacts, MutableReferenceAssignmentDisposition,
    MutableReferenceAssignmentFacts, ReferenceCallDisposition, ReferenceFunctionContract,
    ReferenceFunctionDisposition, classify_local_borrow, classify_local_dereference,
    classify_local_reference_annotation, classify_mutable_reference_assignment,
    classify_mutable_reference_binding, classify_reference_call, classify_reference_function,
};
use crate::scalar_assignment::{
    ScalarAssignmentDisposition, ScalarAssignmentTargetFacts, classify_scalar_assignment,
};
use crate::struct_contract::{
    CopyStructArrayIndexDisposition, StructExecutionContext, StructRegistry,
};
use crate::types::{OwnershipState, Ty, infer_binary_type};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

type ArrayInferenceCache = HashMap<*const Expression, Ty>;

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub ty: Ty,
    pub mutable: bool,
    pub initialized: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Ty,
    pub defined_at: Option<String>,
}

#[derive(Debug, Clone)]
struct AdmittedFunctionContract {
    name: String,
    parameters: Vec<(String, Ty)>,
    return_type: Ty,
}

#[derive(Debug, Clone)]
pub struct VariableInfoNew {
    pub name: String,
    pub var_type: Ty,
    pub mutable: bool,
    pub initialized: bool,
    pub scope_level: u32,
    pub ptr_name: String,
    pub ownership: OwnershipState,
    pub borrowed_from: Option<(String, bool)>,
}

pub struct FunctionTable {
    functions: HashMap<String, FunctionInfo>,
    admitted_contracts: HashMap<String, AdmittedFunctionContract>,
}

impl FunctionTable {
    pub fn clear(&mut self) {
        self.functions.clear();
        self.admitted_contracts.clear();
    }

    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            admitted_contracts: HashMap::new(),
        }
    }
}

impl Default for FunctionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionTable {
    pub fn define_function(&mut self, info: FunctionInfo) -> Result<(), String> {
        if self.functions.contains_key(&info.name) {
            return Err(format!(
                "Error: Function `{}` is already defined.",
                info.name
            ));
        }
        self.functions.insert(info.name.clone(), info);
        Ok(())
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }

    pub fn validate_call(&self, name: &str, args: &[Ty]) -> Result<Ty, String> {
        if let Some(func) = self.functions.get(name) {
            if func.parameters.len() != args.len() {
                return Err(format!(
                    "Error: Function `{}` expects {} arguments, but {} were provided.",
                    name,
                    func.parameters.len(),
                    args.len()
                ));
            }

            for (i, (param, arg_type)) in func.parameters.iter().zip(args.iter()).enumerate() {
                let expected_type = match &param.param_type {
                    crate::ast::Type::Named(type_name) => match type_name.as_str() {
                        "i32" | "int" => Ty::Int,
                        "f64" | "float" => Ty::Float,
                        "bool" => Ty::Bool,
                        "String" => Ty::String,
                        _ => Ty::Int,
                    },
                    crate::ast::Type::Array(_, _) | crate::ast::Type::Tuple(_) => Ty::Int,
                    crate::ast::Type::Reference(_, _) | crate::ast::Type::Generic(_, _) => Ty::Int,
                };

                if expected_type != *arg_type {
                    return Err(format!(
                        "Error: Function `{}` expects type `{}` for argument {}, but `{}` was provided.",
                        name,
                        expected_type,
                        i + 1,
                        arg_type
                    ));
                }
            }

            Ok(func.return_type.clone())
        } else {
            Err(format!("Error: Function `{}` is not defined.", name))
        }
    }

    fn define_admitted_contract(&mut self, contract: AdmittedFunctionContract) {
        self.admitted_contracts
            .insert(contract.name.clone(), contract);
    }

    fn get_admitted_contract(&self, name: &str) -> Option<&AdmittedFunctionContract> {
        self.admitted_contracts.get(name)
    }

    fn validate_admitted_call(&self, name: &str, args: &[Ty]) -> Result<Ty, String> {
        let contract = self
            .admitted_contracts
            .get(name)
            .expect("admitted function contract must be registered");

        if contract.parameters.len() != args.len() {
            return Err(format!(
                "Error: Function `{}` arity mismatch: expected {}, actual {}.",
                name,
                contract.parameters.len(),
                args.len()
            ));
        }

        for ((param_name, expected_type), arg_type) in contract.parameters.iter().zip(args.iter()) {
            if expected_type != arg_type {
                return Err(format!(
                    "Error: Function `{}` parameter `{}` type mismatch: expected {}, actual {}.",
                    name, param_name, expected_type, arg_type
                ));
            }
        }

        Ok(contract.return_type.clone())
    }

    pub fn list_functions(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }
}

pub struct ScopeManager {
    scopes: Vec<HashMap<String, VariableInfoNew>>,
    current_function: Option<String>,
    loop_depth: u32,
    next_ptr: u32,
}

impl ScopeManager {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Start with global scope
            current_function: None,
            loop_depth: 0,
            next_ptr: 0,
        }
    }
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeManager {
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            let scope = self.scopes.pop().expect("non-global scope exists");
            for (_, variable) in scope {
                if let Some((source, mutable)) = variable.borrowed_from {
                    self.release_borrow(&source, mutable);
                }
            }
        }
    }

    pub fn enter_function(&mut self, name: String) {
        self.current_function = Some(name);
        self.enter_scope(); // Functions create their own scope
    }

    pub fn exit_function(&mut self) {
        self.current_function = None;
        self.exit_scope(); // Exit function scope
    }

    pub fn enter_loop(&mut self) {
        self.loop_depth += 1;
        self.enter_scope(); // Loops create their own scope
    }

    pub fn exit_loop(&mut self) {
        if self.loop_depth > 0 {
            self.loop_depth -= 1;
            self.exit_scope(); // Exit loop scope
        }
    }

    pub fn define_variable(
        &mut self,
        name: String,
        var_type: Ty,
        mutable: bool,
        initialized: bool,
    ) -> Result<String, String> {
        // Check if variable already exists in current scope
        if let Some(current_scope) = self.scopes.last()
            && current_scope.contains_key(&name)
        {
            return Err(format!(
                "Error: Variable `{}` is already defined in this scope.",
                name
            ));
        }

        // Generate unique pointer name
        let ptr_name = format!("ptr{}", self.next_ptr);
        self.next_ptr += 1;

        let var_info = VariableInfoNew {
            name: name.clone(),
            var_type,
            mutable,
            initialized,
            scope_level: (self.scopes.len() - 1) as u32,
            ptr_name: ptr_name.clone(),
            ownership: OwnershipState::Owned,
            borrowed_from: None,
        };

        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, var_info);
        }

        Ok(ptr_name)
    }

    pub fn get_variable(&self, name: &str) -> Option<&VariableInfoNew> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.get(name) {
                return Some(var_info);
            }
        }
        None
    }

    pub fn variable_exists_in_current_scope(&self, name: &str) -> bool {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.contains_key(name)
        } else {
            false
        }
    }

    pub fn is_in_function(&self) -> bool {
        self.current_function.is_some()
    }

    pub fn get_current_function(&self) -> Option<&String> {
        self.current_function.as_ref()
    }

    pub fn can_break_continue(&self) -> bool {
        self.loop_depth > 0
    }

    pub fn get_scope_level(&self) -> u32 {
        (self.scopes.len() - 1) as u32
    }

    pub fn get_loop_depth(&self) -> u32 {
        self.loop_depth
    }

    pub fn check_mutability(&self, name: &str) -> Result<bool, String> {
        if let Some(var_info) = self.get_variable(name) {
            Ok(var_info.mutable)
        } else {
            Err(format!("Error: Variable `{}` not found.", name))
        }
    }

    pub fn update_variable_initialization(
        &mut self,
        name: &str,
        initialized: bool,
    ) -> Result<(), String> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                var_info.initialized = initialized;
                return Ok(());
            }
        }
        Err(format!("Error: Variable `{}` not found.", name))
    }

    pub fn is_shadowing(&self, name: &str) -> bool {
        let mut found_count = 0;
        for scope in &self.scopes {
            if scope.contains_key(name) {
                found_count += 1;
                if found_count > 1 {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_current_scope_variables(&self) -> Vec<&String> {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.keys().collect()
        } else {
            vec![]
        }
    }

    pub fn get_all_variables(&self) -> Vec<&VariableInfoNew> {
        let mut all_vars = vec![];
        for scope in &self.scopes {
            for var_info in scope.values() {
                all_vars.push(var_info);
            }
        }
        all_vars
    }

    /// Mark a variable as moved. Returns error if already moved.
    pub fn mark_moved(&mut self, name: &str) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                if var_info.ownership == OwnershipState::Moved {
                    return Err(format!(
                        "Error: Use of moved value `{}`. Value was previously moved.",
                        name
                    ));
                }
                var_info.ownership = OwnershipState::Moved;
                return Ok(());
            }
        }
        Err(format!("Error: Variable `{}` not found.", name))
    }

    /// Check if a variable is in a valid (non-moved) state for use.
    pub fn check_not_moved(&self, name: &str) -> Result<(), String> {
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.get(name) {
                if var_info.ownership == OwnershipState::Moved {
                    return Err(format!(
                        "Error: Use of moved value `{}`. Value was previously moved.",
                        name
                    ));
                }
                return Ok(());
            }
        }
        Ok(()) // Not found in scope manager, might be in old symbol table
    }

    /// Get the ownership state of a variable.
    pub fn get_ownership(&self, name: &str) -> Option<&OwnershipState> {
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.get(name) {
                return Some(&var_info.ownership);
            }
        }
        None
    }

    /// Add an immutable borrow to a variable. Fails if mutably borrowed.
    pub fn add_immutable_borrow(&mut self, name: &str) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                match &var_info.ownership {
                    OwnershipState::Moved => {
                        return Err(format!(
                            "Error: Cannot borrow `{}` because it was moved.",
                            name
                        ));
                    }
                    OwnershipState::MutablyBorrowed => {
                        return Err(format!(
                            "Error: Cannot borrow `{}` as immutable because it is also borrowed as mutable.",
                            name
                        ));
                    }
                    OwnershipState::ImmutablyBorrowed(count) => {
                        var_info.ownership = OwnershipState::ImmutablyBorrowed(count + 1);
                        return Ok(());
                    }
                    OwnershipState::Owned => {
                        var_info.ownership = OwnershipState::ImmutablyBorrowed(1);
                        return Ok(());
                    }
                }
            }
        }
        Err(format!("Error: Variable `{}` not found.", name))
    }

    /// Add a mutable borrow to a variable. Fails if any borrows exist.
    pub fn add_mutable_borrow(&mut self, name: &str) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                match &var_info.ownership {
                    OwnershipState::Moved => {
                        return Err(format!(
                            "Error: Cannot borrow `{}` as mutable because it was moved.",
                            name
                        ));
                    }
                    OwnershipState::MutablyBorrowed => {
                        return Err(format!(
                            "Error: Cannot borrow `{}` as mutable more than once at a time.",
                            name
                        ));
                    }
                    OwnershipState::ImmutablyBorrowed(_) => {
                        return Err(format!(
                            "Error: Cannot borrow `{}` as mutable because it is also borrowed as immutable.",
                            name
                        ));
                    }
                    OwnershipState::Owned => {
                        if !var_info.mutable {
                            return Err(format!(
                                "Error: Cannot borrow `{}` as mutable because it is not declared as mutable.",
                                name
                            ));
                        }
                        var_info.ownership = OwnershipState::MutablyBorrowed;
                        return Ok(());
                    }
                }
            }
        }
        Err(format!("Error: Variable `{}` not found.", name))
    }

    fn record_classified_borrow(&mut self, name: &str, mutable: bool) {
        for scope in self.scopes.iter_mut().rev() {
            let Some(variable) = scope.get_mut(name) else {
                continue;
            };
            variable.ownership = if mutable {
                OwnershipState::MutablyBorrowed
            } else {
                match variable.ownership.clone() {
                    OwnershipState::Owned => OwnershipState::ImmutablyBorrowed(1),
                    OwnershipState::ImmutablyBorrowed(count) => {
                        OwnershipState::ImmutablyBorrowed(count + 1)
                    }
                    _ => unreachable!("shared reference classifier rejected conflicting borrow"),
                }
            };
            return;
        }
        unreachable!("shared reference classifier resolved a local source")
    }

    pub fn set_borrow_origin(
        &mut self,
        alias: &str,
        source: String,
        mutable: bool,
    ) -> Result<(), String> {
        let Some(variable) = self
            .scopes
            .last_mut()
            .and_then(|scope| scope.get_mut(alias))
        else {
            return Err(format!("Error: Variable `{alias}` not found."));
        };
        variable.borrowed_from = Some((source, mutable));
        Ok(())
    }

    fn release_borrow(&mut self, name: &str, mutable: bool) {
        for scope in self.scopes.iter_mut().rev() {
            let Some(variable) = scope.get_mut(name) else {
                continue;
            };
            if mutable {
                if variable.ownership == OwnershipState::MutablyBorrowed {
                    variable.ownership = OwnershipState::Owned;
                }
            } else if let OwnershipState::ImmutablyBorrowed(count) = variable.ownership.clone() {
                variable.ownership = if count > 1 {
                    OwnershipState::ImmutablyBorrowed(count - 1)
                } else {
                    OwnershipState::Owned
                };
            }
            return;
        }
    }
}

pub struct SemanticAnalyzer {
    symbol_table: HashMap<String, VariableInfo>,
    compatibility_scope_snapshots: Vec<HashMap<String, VariableInfo>>,
    function_table: FunctionTable,
    closure_binding_scopes: Vec<HashSet<String>>,
    return_contract_stack: Vec<Option<AdmittedFunctionContract>>,
    scope_manager: ScopeManager,
    /// Stack of active generic type parameter sets (e.g., ["T", "U"] for fn<T, U>)
    type_param_scopes: Vec<Vec<String>>,
    /// Whether the current statement is nested beneath an impl block.
    inside_impl: bool,
    struct_registry: StructRegistry,
    enum_registry: EnumRegistry,
    enum_function_contracts: HashMap<String, EnumFunctionContract>,
    reference_function_contracts: HashMap<String, ReferenceFunctionContract>,
    enum_payload_binding_scopes: RefCell<Vec<HashMap<String, Ty>>>,
    struct_execution_stack: Vec<StructExecutionContext>,
    /// Trait registry: trait name -> list of required method names
    trait_registry: HashMap<String, Vec<String>>,
    /// Trait impl registry: type name -> list of implemented trait names
    trait_impls: HashMap<String, Vec<String>>,
    /// Function trait bounds: function name -> [(type_param, [trait_name])]
    function_bounds: HashMap<String, Vec<(String, Vec<String>)>>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut trait_registry = HashMap::new();
        // Built-in protocol used by `for x in ...`.
        trait_registry.insert("IntoIterator".to_string(), vec!["iter".to_string()]);

        Self {
            symbol_table: HashMap::new(),
            compatibility_scope_snapshots: Vec::new(),
            function_table: FunctionTable::new(),
            closure_binding_scopes: vec![HashSet::new()],
            return_contract_stack: Vec::new(),
            scope_manager: ScopeManager::new(),
            type_param_scopes: Vec::new(),
            inside_impl: false,
            struct_registry: StructRegistry::default(),
            enum_registry: EnumRegistry::default(),
            enum_function_contracts: HashMap::new(),
            reference_function_contracts: HashMap::new(),
            enum_payload_binding_scopes: RefCell::new(Vec::new()),
            struct_execution_stack: vec![StructExecutionContext::PreservedContext],
            trait_registry,
            trait_impls: HashMap::new(),
            function_bounds: HashMap::new(),
        }
    }

    /// Check if a name is an in-scope type parameter.
    fn is_type_param(&self, name: &str) -> bool {
        self.type_param_scopes
            .iter()
            .any(|scope| scope.iter().any(|p| p == name))
    }

    fn struct_execution_context(&self) -> StructExecutionContext {
        self.struct_execution_stack
            .last()
            .copied()
            .unwrap_or(StructExecutionContext::PreservedContext)
    }

    fn enum_execution_context(&self) -> EnumExecutionContext {
        match self.struct_execution_context() {
            StructExecutionContext::AdmittedFunction => EnumExecutionContext::AdmittedFunction,
            StructExecutionContext::PreservedContext => EnumExecutionContext::PreservedContext,
        }
    }

    fn local_binding_type(&self, name: &str) -> Option<Ty> {
        self.scope_manager
            .get_variable(name)
            .map(|binding| binding.var_type.clone())
            .or_else(|| {
                self.symbol_table
                    .get(name)
                    .map(|binding| binding.ty.clone())
            })
    }

    fn enum_consumed_names(&self, expression: &Expression) -> Result<Vec<String>, String> {
        self.enum_registry
            .consumed_owned_values(
                expression,
                |name| self.local_binding_type(name),
                |name| {
                    self.enum_function_contracts
                        .get(name)
                        .map(EnumFunctionContract::parameter_types)
                },
            )
            .map_err(|error| error.diagnostic())
    }

    fn enum_payload_binding_type(&self, name: &str) -> Option<Ty> {
        self.enum_payload_binding_scopes
            .borrow()
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned())
    }

    fn push_enum_payload_binding(&self, name: String, ty: Ty) {
        self.enum_payload_binding_scopes
            .borrow_mut()
            .push(HashMap::from([(name, ty)]));
    }

    fn pop_enum_payload_binding(&self) {
        self.enum_payload_binding_scopes.borrow_mut().pop();
    }

    fn apply_enum_match_moves(&mut self, expression: &Expression) -> Result<(), String> {
        let consumed = self.enum_consumed_names(expression)?;
        for name in consumed {
            self.scope_manager.mark_moved(&name)?;
        }
        Ok(())
    }

    fn is_copy_type(&self, ty: &Ty) -> bool {
        self.struct_registry.is_copy_type(ty)
    }

    fn local_reference_source_facts(
        &self,
        expression: &Expression,
    ) -> Option<LocalReferenceSourceFacts> {
        let Expression::Identifier(name) = expression else {
            return None;
        };
        self.scope_manager
            .get_variable(name)
            .map(|variable| LocalReferenceSourceFacts {
                ty: variable.var_type.clone(),
                mutable: variable.mutable,
                initialized: variable.initialized,
                local: variable.scope_level > 0,
                ownership: variable.ownership.clone(),
            })
    }

    fn reference_call_disposition(
        &self,
        name: &str,
        arguments: &[Expression],
    ) -> ReferenceCallDisposition {
        let Some(contract) = self.reference_function_contracts.get(name) else {
            return ReferenceCallDisposition::Preserved;
        };
        let facts = arguments.first().and_then(|argument| {
            let Expression::Borrow {
                expr,
                mutable: true,
            } = argument
            else {
                return None;
            };
            self.local_reference_source_facts(expr)
        });
        classify_reference_call(contract, arguments, facts.as_ref())
    }

    fn readable_identifier_type(&self, name: &str) -> Result<Ty, String> {
        if let Some(ty) = self.enum_payload_binding_type(name) {
            return Ok(ty);
        }
        if let Some(variable) = self.scope_manager.get_variable(name) {
            if !variable.initialized {
                return Err(format!("Error: Use of uninitialized variable `{name}`."));
            }
            if variable.ownership == OwnershipState::MutablyBorrowed {
                return Err(format!("cannot read `{name}` while it is mutably borrowed"));
            }
            return Ok(variable.var_type.clone());
        }
        if let Some(variable) = self.symbol_table.get(name) {
            if !variable.initialized {
                return Err(format!("Error: Use of uninitialized variable `{name}`."));
            }
            return Ok(variable.ty.clone());
        }
        Err(format!("Error: Use of undeclared variable `{name}`."))
    }

    fn infer_supported_field_type(
        &self,
        object: &Expression,
        field: &str,
        array_types: &mut ArrayInferenceCache,
    ) -> Result<Ty, String> {
        let receiver = self
            .infer_and_validate_expression_immutable_with_cache(object, array_types)
            .map_err(|_| "Field access expressions are not supported.".to_string())?;
        let (_, _, field_contract) = self
            .struct_registry
            .resolve_field(&receiver, field, self.struct_execution_context())
            .map_err(|error| {
                if matches!(
                    error,
                    crate::struct_contract::StructContractError::PreserveExistingBehavior
                ) {
                    "Field access expressions are not supported.".to_string()
                } else {
                    error.diagnostic()
                }
            })?;
        Ok(field_contract.ty())
    }

    fn infer_into_iterator_item_type(&self, iterable_type: &Ty) -> Option<Ty> {
        match iterable_type {
            Ty::Array(elem, _) => Some((**elem).clone()),
            Ty::Vec(elem) => Some((**elem).clone()),
            // Keep legacy behavior for old range-start lowering.
            Ty::Int => Some(Ty::Int),
            _ => None,
        }
    }

    fn enter_scope(&mut self) {
        self.compatibility_scope_snapshots
            .push(self.symbol_table.clone());
        self.scope_manager.enter_scope();
        self.closure_binding_scopes.push(HashSet::new());
    }

    fn exit_scope(&mut self) {
        self.scope_manager.exit_scope();
        if self.closure_binding_scopes.len() > 1 {
            self.closure_binding_scopes.pop();
        }
        self.restore_compatibility_scope();
    }

    fn enter_function_scope(&mut self, name: String) {
        self.compatibility_scope_snapshots
            .push(self.symbol_table.clone());
        self.scope_manager.enter_function(name);
        self.closure_binding_scopes.push(HashSet::new());
    }

    fn exit_function_scope(&mut self) {
        self.scope_manager.exit_function();
        if self.closure_binding_scopes.len() > 1 {
            self.closure_binding_scopes.pop();
        }
        self.restore_compatibility_scope();
    }

    fn enter_loop_scope(&mut self) {
        self.compatibility_scope_snapshots
            .push(self.symbol_table.clone());
        self.scope_manager.enter_loop();
        self.closure_binding_scopes.push(HashSet::new());
    }

    fn exit_loop_scope(&mut self) {
        self.scope_manager.exit_loop();
        if self.closure_binding_scopes.len() > 1 {
            self.closure_binding_scopes.pop();
        }
        self.restore_compatibility_scope();
    }

    fn restore_compatibility_scope(&mut self) {
        let bindings = self
            .compatibility_scope_snapshots
            .pop()
            .expect("compatibility scope snapshot must match semantic scope");
        self.symbol_table = bindings;
    }

    fn is_closure_callable(&self, name: &str) -> bool {
        self.scope_manager
            .scopes
            .iter()
            .zip(self.closure_binding_scopes.iter())
            .rev()
            .find_map(|(variables, closures)| {
                variables
                    .contains_key(name)
                    .then_some(closures.contains(name))
            })
            .unwrap_or(false)
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer {
    pub fn analyze(&mut self, ast: Vec<AstNode>) -> Result<(String, Vec<AstNode>), String> {
        self.function_table.clear();
        self.compatibility_scope_snapshots.clear();
        self.closure_binding_scopes.clear();
        self.closure_binding_scopes.push(HashSet::new());
        self.return_contract_stack.clear();
        self.type_param_scopes.clear();
        self.inside_impl = false;
        self.struct_registry = StructRegistry::from_top_level_ast(&ast);
        self.enum_registry = EnumRegistry::from_top_level_ast(&ast);
        self.enum_function_contracts.clear();
        self.reference_function_contracts.clear();
        self.enum_payload_binding_scopes.borrow_mut().clear();
        self.struct_execution_stack.clear();
        self.struct_execution_stack
            .push(StructExecutionContext::PreservedContext);

        // Predeclare all top-level names so forward calls and direct recursion see
        // the same program-wide namespace after module linking.
        for node in &ast {
            if let AstNode::Statement(Statement::Function {
                name,
                parameters,
                return_type,
                type_params,
                ..
            }) = node
            {
                let reference_transport = match classify_reference_function(
                    name,
                    parameters,
                    return_type.as_ref(),
                    type_params,
                ) {
                    ReferenceFunctionDisposition::Supported(contract) => Some(contract),
                    ReferenceFunctionDisposition::ExplicitlyRejected(diagnostic) => {
                        return Err(diagnostic);
                    }
                    ReferenceFunctionDisposition::Preserved => None,
                };
                let enum_transport = if reference_transport.is_none() {
                    self.enum_registry
                        .resolve_function_contract(
                            name,
                            parameters,
                            return_type.as_ref(),
                            type_params,
                            |annotation| {
                                self.struct_registry
                                    .resolve_copy_annotation(annotation)
                                    .map(|contract| (contract.ty, contract.logical_type))
                            },
                        )
                        .map_err(|error| error.diagnostic())?
                } else {
                    None
                };
                let admitted_contract = if let Some(contract) = reference_transport {
                    self.reference_function_contracts
                        .insert(name.clone(), contract.clone());
                    Some(AdmittedFunctionContract {
                        name: contract.name,
                        parameters: contract
                            .parameters
                            .into_iter()
                            .map(|(parameter, contract)| (parameter, contract.ty))
                            .collect(),
                        return_type: contract.result.ty,
                    })
                } else if let Some(contract) = enum_transport {
                    self.enum_function_contracts
                        .insert(name.clone(), contract.clone());
                    Some(AdmittedFunctionContract {
                        name: contract.name,
                        parameters: contract
                            .parameters
                            .into_iter()
                            .map(|(parameter, contract)| (parameter, contract.ty))
                            .collect(),
                        return_type: contract.result.ty,
                    })
                } else if type_params.is_empty() {
                    let copy_contract = match self.struct_registry.resolve_copy_function_contract(
                        name,
                        parameters,
                        return_type.as_ref(),
                        type_params,
                    ) {
                        Ok(contract) => contract,
                        Err(
                            crate::struct_contract::StructContractError::PreserveExistingBehavior,
                        ) => None,
                        Err(error) => return Err(error.diagnostic()),
                    };
                    if let Some(contract) = copy_contract {
                        Some(AdmittedFunctionContract {
                            name: contract.name,
                            parameters: contract
                                .parameters
                                .into_iter()
                                .map(|(parameter, contract)| (parameter, contract.ty))
                                .collect(),
                            return_type: contract.result.ty,
                        })
                    } else {
                        let contract_type: fn(&crate::ast::Type) -> Option<Ty> = if name == "main" {
                            Self::numeric_contract_type
                        } else {
                            Self::helper_contract_type
                        };
                        let parameter_types = parameters
                            .iter()
                            .map(|parameter| {
                                contract_type(&parameter.param_type)
                                    .map(|ty| (parameter.name.clone(), ty))
                            })
                            .collect::<Option<Vec<_>>>();
                        let contract_return = match return_type {
                            Some(ty) => contract_type(ty),
                            None => Some(Ty::Void),
                        };

                        match (parameter_types, contract_return) {
                            (Some(parameters), Some(return_type)) => {
                                Some(AdmittedFunctionContract {
                                    name: name.clone(),
                                    parameters,
                                    return_type,
                                })
                            }
                            _ => None,
                        }
                    }
                } else {
                    None
                };

                let public_return_type = return_type
                    .as_ref()
                    .map(|ty| self.ast_type_to_ty(ty))
                    .unwrap_or(Ty::Void);
                self.function_table.define_function(FunctionInfo {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    return_type: public_return_type,
                    defined_at: None,
                })?;
                if let Some(contract) = admitted_contract {
                    self.function_table.define_admitted_contract(contract);
                }
            }
        }

        for node in &ast {
            match node {
                AstNode::Statement(stmt) => {
                    self.analyze_statement_with_context(stmt, true)?;
                }
                AstNode::Expression(expr) => {
                    self.check_expression_initialization(expr)?;
                    self.infer_and_validate_expression_immutable(expr)?;
                }
            }
        }
        Ok(("Semantic analysis completed successfully".to_string(), ast))
    }

    fn numeric_contract_type(ty: &crate::ast::Type) -> Option<Ty> {
        match ty {
            crate::ast::Type::Named(name) if matches!(name.as_str(), "i32" | "int") => {
                Some(Ty::Int)
            }
            crate::ast::Type::Named(name) if matches!(name.as_str(), "f64" | "float") => {
                Some(Ty::Float)
            }
            _ => None,
        }
    }

    fn helper_contract_type(ty: &crate::ast::Type) -> Option<Ty> {
        Self::numeric_contract_type(ty).or_else(|| match ty {
            crate::ast::Type::Named(name) if name == "bool" => Some(Ty::Bool),
            _ => None,
        })
    }

    fn is_numeric_type(ty: &Ty) -> bool {
        matches!(ty, Ty::Int | Ty::Float)
    }

    fn require_homogeneous_numeric_array(
        &self,
        expected: &Ty,
        actual_types: impl IntoIterator<Item = Ty>,
    ) -> Result<(), String> {
        for actual in actual_types {
            if actual != *expected {
                return Err(format!(
                    "Error: array element type mismatch: expected {}, actual {}.",
                    expected, actual
                ));
            }
        }
        Ok(())
    }

    fn infer_non_generic_array_literal(
        &self,
        elements: &[Expression],
        array_types: &mut ArrayInferenceCache,
    ) -> Result<Ty, String> {
        let Some(first) = elements.first() else {
            return Ok(Ty::Array(Box::new(Ty::Int), 0));
        };

        self.preflight_expression_for_inference_with_cache(first, array_types)?;
        let elem_type = match self.infer_expression_immutable_with_cache(first, array_types) {
            Ok(elem_type) => elem_type,
            Err(first_error) => {
                for element in elements.iter().skip(1) {
                    self.preflight_expression(element)?;
                }
                return Err(first_error);
            }
        };

        if Self::is_numeric_type(&elem_type) {
            let mut remaining_types = Vec::with_capacity(elements.len().saturating_sub(1));
            for element in elements.iter().skip(1) {
                remaining_types.push(
                    self.infer_and_validate_expression_immutable_with_cache(element, array_types)?,
                );
            }
            self.require_homogeneous_numeric_array(&elem_type, remaining_types)?;
        } else if self.struct_registry.is_copy_struct_ty(&elem_type) {
            let mut remaining_types = Vec::with_capacity(elements.len().saturating_sub(1));
            for element in elements.iter().skip(1) {
                remaining_types.push(
                    self.infer_and_validate_expression_immutable_with_cache(element, array_types)?,
                );
            }
            self.struct_registry
                .validate_copy_struct_array_elements(&elem_type, remaining_types)
                .map_err(|error| error.diagnostic())?;
        } else {
            for element in elements.iter().skip(1) {
                self.preflight_expression(element)?;
            }
        }

        Ok(Ty::Array(Box::new(elem_type), elements.len()))
    }

    fn require_value(&self, expr: &Expression) -> Result<Ty, String> {
        let ty = self.infer_and_validate_expression_immutable(expr)?;
        if ty != Ty::Void {
            return Ok(ty);
        }

        if let Expression::FunctionCall { name, .. } = expr {
            Err(format!(
                "Error: void function `{}` cannot be used as a value.",
                name
            ))
        } else {
            Err("Error: Void expression cannot be used as a value.".to_string())
        }
    }

    fn return_type_mismatch(name: &str, expected: &Ty, actual: &Ty) -> String {
        format!(
            "Error: Function `{}` return type mismatch: expected {}, actual {}.",
            name,
            Self::contract_type_name(expected),
            Self::contract_type_name(actual)
        )
    }

    fn contract_type_name(ty: &Ty) -> String {
        match ty {
            Ty::Void => "void".to_string(),
            _ => ty.to_string(),
        }
    }

    fn statement_definitely_returns(statement: &Statement) -> bool {
        match statement {
            Statement::Return(_) => true,
            Statement::Block(block) => Self::control_flow_block_definitely_returns(block),
            Statement::If {
                then_block,
                else_block: Some(else_statement),
                ..
            } => {
                Self::control_flow_block_definitely_returns(then_block)
                    && Self::statement_definitely_returns(else_statement)
            }
            _ => false,
        }
    }

    fn control_flow_block_definitely_returns(block: &Block) -> bool {
        block
            .statements
            .iter()
            .any(Self::statement_definitely_returns)
    }

    fn function_body_definitely_returns(block: &Block) -> bool {
        block.expression.is_some() || Self::control_flow_block_definitely_returns(block)
    }

    fn check_expression_initialization(&self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Identifier(name) => {
                if self.enum_payload_binding_type(name).is_some() {
                    return Ok(());
                }
                // Phase 5: Check for use-after-move
                self.scope_manager.check_not_moved(name)?;

                if let Some(var_info) = self.scope_manager.get_variable(name) {
                    if !var_info.initialized {
                        return Err(format!("Error: Use of uninitialized variable `{}`.", name));
                    }
                } else if let Some(var_info) = self.symbol_table.get(name) {
                    if !var_info.initialized {
                        return Err(format!("Error: Use of uninitialized variable `{}`.", name));
                    }
                } else {
                    return Err(format!("Error: Use of undeclared variable `{}`.", name));
                }
            }
            Expression::Binary { left, right, .. } => {
                self.check_expression_initialization(left)?;
                self.check_expression_initialization(right)?;
            }
            Expression::FunctionCall { name, arguments } => {
                match self.reference_call_disposition(name, arguments) {
                    ReferenceCallDisposition::Supported(_) => return Ok(()),
                    ReferenceCallDisposition::ExplicitlyRejected(message) => {
                        return Err(message);
                    }
                    ReferenceCallDisposition::Preserved => {}
                }
                for arg in arguments {
                    self.check_expression_initialization(arg)?;
                }
            }
            Expression::Print { arguments, .. } => {
                for arg in arguments {
                    self.check_expression_initialization(arg)?;
                }
            }
            Expression::Println { arguments, .. } => {
                for arg in arguments {
                    self.check_expression_initialization(arg)?;
                }
            }
            Expression::Comparison { left, right, .. } => {
                self.check_expression_initialization(left)?;
                self.check_expression_initialization(right)?;
            }
            Expression::Logical { left, right, .. } => {
                self.check_expression_initialization(left)?;
                self.check_expression_initialization(right)?;
            }
            Expression::Unary { operand, .. } => {
                self.check_expression_initialization(operand)?;
            }
            Expression::Borrow { expr, .. } | Expression::Deref(expr) => {
                self.check_expression_initialization(expr)?;
            }
            Expression::EnumVariant { data, .. } => {
                if let Some(data) = data {
                    self.check_expression_initialization(data)?;
                }
            }
            Expression::Match { expr, arms } => {
                if self.enum_registry.is_candidate_match_scrutinee(
                    expr,
                    self.enum_execution_context(),
                    |name| self.local_binding_type(name),
                    |name| {
                        self.function_table
                            .get_admitted_contract(name)
                            .map(|contract| contract.return_type.clone())
                    },
                ) {
                    self.check_expression_initialization(expr)?;
                    for arm in arms {
                        let binding = match &arm.pattern {
                            crate::ast::Pattern::Enum {
                                data: Some(data), ..
                            } => match data.as_ref() {
                                crate::ast::Pattern::Identifier(name) => Some(name.clone()),
                                _ => None,
                            },
                            _ => None,
                        };
                        let has_binding = binding.is_some();
                        if let Some(name) = binding {
                            self.push_enum_payload_binding(name, Ty::Int);
                        }
                        let result = self.check_expression_initialization(&arm.body);
                        if has_binding {
                            self.pop_enum_payload_binding();
                        }
                        result?;
                    }
                }
            }
            _ => {} // Literals don't need initialization checks
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn infer_and_validate_expression(&self, expr: &mut Expression) -> Result<Ty, String> {
        match expr {
            Expression::IntegerLiteral(_) => Ok(Ty::Int),
            Expression::FloatLiteral(_) => Ok(Ty::Float),
            Expression::Identifier(name) => self.readable_identifier_type(name),
            Expression::Binary {
                op, left, right, ..
            } => {
                let lhs_type = self.infer_and_validate_expression(left)?;
                let rhs_type = self.infer_and_validate_expression(right)?;
                infer_binary_type(op.as_str(), &lhs_type, &rhs_type)
            }
            Expression::FunctionCall { name, arguments } => {
                match self.reference_call_disposition(name, arguments) {
                    ReferenceCallDisposition::Supported(contract) => {
                        return self
                            .function_table
                            .validate_admitted_call(name, &[contract.reference_type()]);
                    }
                    ReferenceCallDisposition::ExplicitlyRejected(message) => {
                        return Err(message);
                    }
                    ReferenceCallDisposition::Preserved => {}
                }
                for arg in arguments {
                    self.infer_and_validate_expression(arg)?;
                }
                Ok(Ty::Int)
            }
            Expression::Print {
                format_string,
                arguments,
            } => {
                self.validate_format_string_and_args(format_string, arguments)?;
                Ok(Ty::Int)
            }
            Expression::Println {
                format_string,
                arguments,
            } => {
                self.validate_format_string_and_args(format_string, arguments)?;
                Ok(Ty::Int)
            }
            Expression::Comparison { op, left, right } => {
                let left_type = self.infer_and_validate_expression(left)?;
                let right_type = self.infer_and_validate_expression(right)?;
                self.validate_comparison_operands(op, &left_type, &right_type)?;
                Ok(Ty::Bool)
            }
            Expression::Logical { op, left, right } => {
                let left_type = self.infer_and_validate_expression(left)?;
                let right_type = self.infer_and_validate_expression(right)?;
                self.validate_logical_operands(op, &left_type, &right_type)?;
                Ok(Ty::Bool)
            }
            Expression::Unary { operand, op } => {
                let operand_type = self.infer_and_validate_expression(operand)?;
                self.validate_unary_operation(op, &operand_type)
            }
            // Phase 4 expressions
            Expression::StringLiteral(_) => Ok(Ty::String),
            Expression::MethodCall { object, method, .. } => {
                let obj_ty = self.infer_and_validate_expression(object)?;
                // Phase 6: Option, Result, Vec, HashMap methods
                match &obj_ty {
                    Ty::Option(inner) => match method.as_str() {
                        "is_some" | "is_none" => Ok(Ty::Bool),
                        "unwrap" | "expect" | "unwrap_or" | "unwrap_or_else" => Ok(*inner.clone()),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::Result(ok_ty, err_ty) => match method.as_str() {
                        "is_ok" | "is_err" => Ok(Ty::Bool),
                        "unwrap" | "expect" | "unwrap_or" | "unwrap_or_else" => Ok(*ok_ty.clone()),
                        "unwrap_err" | "expect_err" => Ok(*err_ty.clone()),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::Vec(elem) => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" => Ok(Ty::Bool),
                        "push" | "clear" => Ok(Ty::Void),
                        "pop" | "first" | "last" | "get" => Ok(Ty::Option(elem.clone())),
                        "iter" => Ok(Ty::Vec(elem.clone())),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::Array(elem, size) => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" => Ok(Ty::Bool),
                        "first" | "last" | "get" => Ok(Ty::Option(elem.clone())),
                        "iter" => Ok(Ty::Array(elem.clone(), *size)),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::HashMap(_, val) => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" | "contains_key" => Ok(Ty::Bool),
                        "insert" | "remove" | "clear" => Ok(Ty::Void),
                        "get" => Ok(Ty::Option(val.clone())),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::String => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" | "contains" | "starts_with" | "ends_with" => Ok(Ty::Bool),
                        "to_uppercase" | "to_lowercase" | "trim" | "trim_start" | "trim_end" => {
                            Ok(Ty::String)
                        }
                        "chars" => Ok(Ty::Vec(Box::new(Ty::Int))), // char as int
                        _ => Ok(Ty::Int),                          // Unknown method
                    },
                    _ => Ok(Ty::Int), // Other method calls - stub
                }
            }
            Expression::ArrayLiteral(elements) => {
                let Some(first) = elements.first() else {
                    return Ok(Ty::Array(Box::new(Ty::Int), 0));
                };
                let elem_type = self.infer_and_validate_expression(&mut first.clone())?;
                if self.type_param_scopes.is_empty() && Self::is_numeric_type(&elem_type) {
                    let mut remaining_types = Vec::with_capacity(elements.len().saturating_sub(1));
                    for element in elements.iter().skip(1) {
                        remaining_types
                            .push(self.infer_and_validate_expression(&mut element.clone())?);
                    }
                    self.require_homogeneous_numeric_array(&elem_type, remaining_types)?;
                } else if self.type_param_scopes.is_empty()
                    && self.struct_registry.is_copy_struct_ty(&elem_type)
                {
                    let mut remaining_types = Vec::with_capacity(elements.len().saturating_sub(1));
                    for element in elements.iter().skip(1) {
                        remaining_types
                            .push(self.infer_and_validate_expression(&mut element.clone())?);
                    }
                    self.struct_registry
                        .validate_copy_struct_array_elements(&elem_type, remaining_types)
                        .map_err(|error| error.diagnostic())?;
                }
                Ok(Ty::Array(Box::new(elem_type), elements.len()))
            }
            Expression::ArrayRepeat { value, count } => {
                let elem_type = self.infer_and_validate_expression(value)?;
                Ok(Ty::Array(Box::new(elem_type), *count))
            }
            Expression::IndexAccess { object, index } => {
                let obj_type = self.infer_and_validate_expression(object)?;
                let index_type = self.infer_and_validate_expression(index)?;
                if self.type_param_scopes.is_empty()
                    && matches!(&obj_type, Ty::Array(element, _) if Self::is_numeric_type(element) || self.struct_registry.is_copy_struct_ty(element))
                    && index_type != Ty::Int
                {
                    return Err(format!(
                        "Error: array index type mismatch: expected int, actual {}.",
                        index_type
                    ));
                }
                if is_statically_empty_fixed_array(&obj_type) {
                    return Err("Error: cannot index a zero-length fixed array.".to_string());
                }
                match self
                    .struct_registry
                    .classify_copy_struct_array_index(&obj_type, index)
                {
                    CopyStructArrayIndexDisposition::PreserveExistingBehavior
                    | CopyStructArrayIndexDisposition::Accepted { .. } => {}
                    CopyStructArrayIndexDisposition::NonConstant => {
                        return Err(
                            "fixed Copy-struct array index must be a compile-time integer constant"
                                .to_string(),
                        );
                    }
                    CopyStructArrayIndexDisposition::OutOfBounds { index, count } => {
                        return Err(format!(
                            "fixed Copy-struct array index {index} is outside 0..{count}"
                        ));
                    }
                }
                match obj_type {
                    Ty::Array(elem, _) => Ok(*elem),
                    _ => Err("Cannot index into non-array type".to_string()),
                }
            }
            Expression::FieldAccess { .. }
            | Expression::TupleLiteral(_)
            | Expression::TupleIndex { .. } => Ok(Ty::Int), // Stub
            Expression::StructLiteral { name, .. } => Ok(Ty::Struct(name.clone())),
            // Phase 6: Special handling for Option and Result constructors
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } => {
                match enum_name.as_str() {
                    "Option" => {
                        match variant.as_str() {
                            "Some" => {
                                // Some(value) -> Option<typeof(value)>
                                if let Some(inner_expr) = data {
                                    let inner_ty = self
                                        .infer_and_validate_expression(&mut inner_expr.clone())?;
                                    Ok(Ty::Option(Box::new(inner_ty)))
                                } else {
                                    Err("Some variant requires a value".to_string())
                                }
                            }
                            "None" => {
                                // None -> Option<unknown>, type must be inferred from context
                                // For now, default to Option<Int> - proper inference would need context
                                Ok(Ty::Option(Box::new(Ty::Int)))
                            }
                            _ => Err(format!("Unknown Option variant: {}", variant)),
                        }
                    }
                    "Result" => {
                        match variant.as_str() {
                            "Ok" => {
                                // Ok(value) -> Result<typeof(value), String> (default error type)
                                if let Some(inner_expr) = data {
                                    let inner_ty = self
                                        .infer_and_validate_expression(&mut inner_expr.clone())?;
                                    Ok(Ty::Result(Box::new(inner_ty), Box::new(Ty::String)))
                                } else {
                                    Err("Ok variant requires a value".to_string())
                                }
                            }
                            "Err" => {
                                // Err(error) -> Result<Int, typeof(error)> (default ok type)
                                if let Some(inner_expr) = data {
                                    let inner_ty = self
                                        .infer_and_validate_expression(&mut inner_expr.clone())?;
                                    Ok(Ty::Result(Box::new(Ty::Int), Box::new(inner_ty)))
                                } else {
                                    Err("Err variant requires a value".to_string())
                                }
                            }
                            _ => Err(format!("Unknown Result variant: {}", variant)),
                        }
                    }
                    _ if self.enum_execution_context()
                        == EnumExecutionContext::AdmittedFunction =>
                    {
                        let actual = data
                            .as_deref_mut()
                            .map(|payload| self.infer_and_validate_expression(payload))
                            .transpose()?;
                        let resolved = self
                            .enum_registry
                            .resolve_constructor(
                                enum_name,
                                variant,
                                data.is_some(),
                                self.enum_execution_context(),
                            )
                            .map_err(|error| error.diagnostic())?;
                        self.enum_registry
                            .validate_constructor_payload(&resolved, actual.as_ref())
                            .map_err(|error| error.diagnostic())?;
                        Ok(resolved.contract.ty())
                    }
                    _ => Ok(Ty::Enum(enum_name.clone())),
                }
            }
            Expression::Match { expr, arms } => {
                let scrutinee = self.infer_and_validate_expression(expr)?;
                let patterns = self
                    .enum_registry
                    .resolve_match_patterns(&scrutinee, expr, arms, self.enum_execution_context())
                    .map_err(|error| error.diagnostic())?;
                let mut result_types = Vec::with_capacity(arms.len());
                for (arm, binding) in arms.iter_mut().zip(patterns.payload_bindings.iter()) {
                    if let Some(binding) = binding {
                        self.push_enum_payload_binding(binding.name.clone(), binding.ty.clone());
                    }
                    let result = self.infer_and_validate_expression(&mut arm.body);
                    if binding.is_some() {
                        self.pop_enum_payload_binding();
                    }
                    result_types.push(result?);
                }
                let consumed = self.enum_consumed_names(expr)?;
                self.enum_registry
                    .resolve_match_with_consumed(
                        &scrutinee,
                        expr,
                        arms,
                        &result_types,
                        &consumed,
                        self.enum_execution_context(),
                    )
                    .map(|resolved| resolved.result)
                    .map_err(|error| error.diagnostic())
            }
            // Phase 5: Borrow and Deref
            Expression::Borrow { expr, mutable } => {
                let facts = self.local_reference_source_facts(expr);
                if facts.is_none() {
                    self.infer_and_validate_expression(expr)?;
                }
                match classify_local_borrow(expr, *mutable, facts.as_ref()) {
                    LocalReferenceDisposition::Supported(contract) => Ok(contract.reference_type()),
                    LocalReferenceDisposition::ExplicitlyRejected(message) => Err(message),
                    LocalReferenceDisposition::Preserved => unreachable!(
                        "borrow expressions are fully classified by the local reference contract"
                    ),
                }
            }
            Expression::Deref(expr) => {
                let inner_ty = self.infer_and_validate_expression(expr)?;
                match classify_local_dereference(&inner_ty) {
                    LocalReferenceDisposition::Supported(contract) => Ok(contract.pointee),
                    LocalReferenceDisposition::ExplicitlyRejected(message) => Err(message),
                    LocalReferenceDisposition::Preserved => unreachable!(
                        "dereference expressions are fully classified by the local reference contract"
                    ),
                }
            }
            // Phase 7: Closures
            Expression::Closure { .. } => Ok(Ty::Int), // Closure type inference stub
        }
    }

    fn preflight_expression(&self, expr: &Expression) -> Result<(), String> {
        let mut array_types = ArrayInferenceCache::new();
        self.preflight_expression_with_array_mode(
            expr,
            false,
            &mut array_types,
            self.struct_execution_context(),
        )
    }

    fn preflight_expression_for_inference_with_cache(
        &self,
        expr: &Expression,
        array_types: &mut ArrayInferenceCache,
    ) -> Result<(), String> {
        self.preflight_expression_with_array_mode(
            expr,
            self.type_param_scopes.is_empty(),
            array_types,
            self.struct_execution_context(),
        )
    }

    fn preflight_expression_with_array_mode(
        &self,
        expr: &Expression,
        interleave_array_inference: bool,
        array_types: &mut ArrayInferenceCache,
        struct_context: StructExecutionContext,
    ) -> Result<(), String> {
        match expr {
            Expression::TupleLiteral(_) | Expression::TupleIndex { .. } => {
                return Err("Tuple expressions are not supported.".to_string());
            }
            Expression::FunctionCall { name, arguments } => {
                if !self.is_closure_callable(name)
                    && self
                        .function_table
                        .get_admitted_contract(name)
                        .is_some_and(|contract| contract.return_type == Ty::Void)
                {
                    return Err(format!(
                        "Error: void function `{}` cannot be used as a value.",
                        name
                    ));
                }
                for argument in arguments {
                    self.preflight_expression_with_array_mode(
                        argument,
                        interleave_array_inference,
                        array_types,
                        struct_context,
                    )?;
                }
            }
            Expression::Binary { left, right, .. }
            | Expression::Comparison { left, right, .. }
            | Expression::Logical { left, right, .. } => {
                self.preflight_expression_with_array_mode(
                    left,
                    interleave_array_inference,
                    array_types,
                    struct_context,
                )?;
                self.preflight_expression_with_array_mode(
                    right,
                    interleave_array_inference,
                    array_types,
                    struct_context,
                )?;
            }
            Expression::MethodCall {
                object, arguments, ..
            } => {
                self.preflight_expression_with_array_mode(
                    object,
                    interleave_array_inference,
                    array_types,
                    struct_context,
                )?;
                for argument in arguments {
                    self.preflight_expression_with_array_mode(
                        argument,
                        false,
                        array_types,
                        struct_context,
                    )?;
                }
            }
            Expression::Print { arguments, .. } | Expression::Println { arguments, .. } => {
                for argument in arguments {
                    self.preflight_expression_with_array_mode(
                        argument,
                        false,
                        array_types,
                        struct_context,
                    )?;
                }
            }
            Expression::Unary { operand, .. } => {
                self.preflight_expression_with_array_mode(
                    operand,
                    interleave_array_inference,
                    array_types,
                    struct_context,
                )?;
            }
            Expression::ArrayLiteral(elements) => {
                if interleave_array_inference {
                    let cache_key = expr as *const Expression;
                    if !array_types.contains_key(&cache_key) {
                        let ty = self.infer_non_generic_array_literal(elements, array_types)?;
                        array_types.insert(cache_key, ty);
                    }
                } else {
                    for element in elements {
                        self.preflight_expression_with_array_mode(
                            element,
                            false,
                            array_types,
                            struct_context,
                        )?;
                    }
                }
            }
            Expression::ArrayRepeat { value, .. } => {
                self.preflight_expression_with_array_mode(
                    value,
                    interleave_array_inference,
                    array_types,
                    struct_context,
                )?;
            }
            Expression::IndexAccess { object, index } => {
                self.preflight_expression_with_array_mode(
                    object,
                    interleave_array_inference,
                    array_types,
                    struct_context,
                )?;
                self.preflight_expression_with_array_mode(
                    index,
                    interleave_array_inference,
                    array_types,
                    struct_context,
                )?;
            }
            Expression::FieldAccess { object, field } => {
                self.preflight_expression_with_array_mode(
                    object,
                    false,
                    array_types,
                    struct_context,
                )?;
                if struct_context != StructExecutionContext::AdmittedFunction {
                    return Err("Field access expressions are not supported.".to_string());
                }
                self.infer_supported_field_type(object, field, array_types)?;
            }
            Expression::StructLiteral { name, fields } => {
                if self
                    .struct_registry
                    .resolve_construction(name, fields, struct_context)
                    .is_ok()
                {
                    return Ok(());
                }
                for (_, value) in fields {
                    self.preflight_expression_with_array_mode(
                        value,
                        false,
                        array_types,
                        StructExecutionContext::PreservedContext,
                    )?;
                }
                return Err("Struct construction expressions are not supported.".to_string());
            }
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } => {
                if let Some(value) = data {
                    let payload_is_inferred = matches!(
                        (enum_name.as_str(), variant.as_str()),
                        ("Option", "Some") | ("Result", "Ok") | ("Result", "Err")
                    );
                    self.preflight_expression_with_array_mode(
                        value,
                        interleave_array_inference && payload_is_inferred,
                        array_types,
                        struct_context,
                    )?;
                }
                if matches!(struct_context, StructExecutionContext::AdmittedFunction)
                    && !matches!(enum_name.as_str(), "Option" | "Result")
                {
                    self.enum_registry
                        .resolve_constructor(
                            enum_name,
                            variant,
                            data.is_some(),
                            EnumExecutionContext::AdmittedFunction,
                        )
                        .map_err(|error| error.diagnostic())?;
                }
            }
            Expression::Match { expr, arms } => {
                self.preflight_expression_with_array_mode(
                    expr,
                    false,
                    array_types,
                    struct_context,
                )?;
                for arm in arms {
                    self.preflight_expression_with_array_mode(
                        &arm.body,
                        false,
                        array_types,
                        struct_context,
                    )?;
                }
                let context = match struct_context {
                    StructExecutionContext::AdmittedFunction => {
                        EnumExecutionContext::AdmittedFunction
                    }
                    StructExecutionContext::PreservedContext => {
                        EnumExecutionContext::PreservedContext
                    }
                };
                if !self.enum_registry.is_candidate_match_scrutinee(
                    expr,
                    context,
                    |name| self.local_binding_type(name),
                    |name| {
                        self.function_table
                            .get_admitted_contract(name)
                            .map(|contract| contract.return_type.clone())
                    },
                ) {
                    return Err("Match expressions are not supported.".to_string());
                }
            }
            Expression::Borrow { expr, .. } | Expression::Deref(expr) => {
                self.preflight_expression_with_array_mode(
                    expr,
                    interleave_array_inference,
                    array_types,
                    struct_context,
                )?;
            }
            Expression::IntegerLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::Identifier(_) => {}
            Expression::Closure { body, .. } => {
                self.preflight_expression_with_array_mode(
                    body,
                    false,
                    array_types,
                    StructExecutionContext::PreservedContext,
                )?;
            }
        }

        Ok(())
    }

    fn preflight_block_syntax(&self, block: &Block) -> Result<(), String> {
        for statement in &block.statements {
            self.preflight_statement_syntax(statement)?;
        }
        if let Some(expression) = &block.expression {
            self.preflight_expression(expression)?;
        }
        Ok(())
    }

    fn preflight_statement_syntax(&self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Let { value, .. } | Statement::Return(value) => {
                if let Some(expression) = value {
                    self.preflight_expression(expression)?;
                }
            }
            Statement::Expression(expression) => self.preflight_expression(expression)?,
            Statement::Assignment { target, value } => {
                self.preflight_expression(target)?;
                self.preflight_expression(value)?;
            }
            Statement::Block(block) => self.preflight_block_syntax(block)?,
            Statement::Function { body, .. } => self.preflight_block_syntax(body)?,
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.preflight_expression(condition)?;
                self.preflight_block_syntax(then_block)?;
                if let Some(else_statement) = else_block {
                    self.preflight_statement_syntax(else_statement)?;
                }
            }
            Statement::While { condition, body } => {
                self.preflight_expression(condition)?;
                self.preflight_block_syntax(body)?;
            }
            Statement::For { iterable, body, .. } => {
                self.preflight_expression(iterable)?;
                self.preflight_block_syntax(body)?;
            }
            Statement::Loop { body } => self.preflight_block_syntax(body)?,
            Statement::ImplBlock { methods, .. } => {
                for method in methods {
                    self.preflight_statement_syntax(method)?;
                }
            }
            Statement::TraitDef { methods, .. } => {
                for method in methods {
                    if let Some(body) = &method.body {
                        self.preflight_block_syntax(body)?;
                    }
                }
            }
            Statement::Break
            | Statement::Continue
            | Statement::StructDef { .. }
            | Statement::EnumDef { .. }
            | Statement::ModDecl { .. }
            | Statement::UseImport { .. } => {}
        }
        Ok(())
    }

    fn infer_and_validate_expression_immutable(&self, expr: &Expression) -> Result<Ty, String> {
        let mut array_types = ArrayInferenceCache::new();
        self.infer_and_validate_expression_immutable_with_cache(expr, &mut array_types)
    }

    fn infer_and_validate_expression_immutable_with_cache(
        &self,
        expr: &Expression,
        array_types: &mut ArrayInferenceCache,
    ) -> Result<Ty, String> {
        if self.type_param_scopes.is_empty()
            && let Expression::ArrayLiteral(elements) = expr
        {
            let cache_key = expr as *const Expression;
            if let Some(ty) = array_types.get(&cache_key) {
                return Ok(ty.clone());
            }
            let ty = self.infer_non_generic_array_literal(elements, array_types)?;
            array_types.insert(cache_key, ty.clone());
            return Ok(ty);
        }
        self.preflight_expression_for_inference_with_cache(expr, array_types)?;
        self.infer_expression_immutable_with_cache(expr, array_types)
    }

    fn infer_discarded_expression_immutable(&self, expr: &Expression) -> Result<Ty, String> {
        if let Expression::FunctionCall { name, .. } = expr
            && !self.is_closure_callable(name)
            && self
                .function_table
                .get_admitted_contract(name)
                .is_some_and(|contract| contract.return_type == Ty::Void)
        {
            let mut array_types = ArrayInferenceCache::new();
            return self.infer_expression_immutable_with_cache(expr, &mut array_types);
        }

        self.infer_and_validate_expression_immutable(expr)
    }

    fn infer_expression_immutable_with_cache(
        &self,
        expr: &Expression,
        array_types: &mut ArrayInferenceCache,
    ) -> Result<Ty, String> {
        match expr {
            Expression::IntegerLiteral(_) => Ok(Ty::Int),
            Expression::FloatLiteral(_) => Ok(Ty::Float),
            Expression::Identifier(name) => self.readable_identifier_type(name),
            Expression::Binary {
                op, left, right, ..
            } => {
                let lhs_type =
                    self.infer_and_validate_expression_immutable_with_cache(left, array_types)?;
                let rhs_type =
                    self.infer_and_validate_expression_immutable_with_cache(right, array_types)?;
                infer_binary_type(op.as_str(), &lhs_type, &rhs_type)
            }
            Expression::FunctionCall { name, arguments } => {
                let direct_mutable_call = match self.reference_call_disposition(name, arguments) {
                    ReferenceCallDisposition::Supported(contract) => Some(contract),
                    ReferenceCallDisposition::ExplicitlyRejected(message) => {
                        return Err(message);
                    }
                    ReferenceCallDisposition::Preserved => None,
                };
                let mut argument_types = Vec::with_capacity(arguments.len());
                if let Some(contract) = direct_mutable_call {
                    argument_types.push(contract.reference_type());
                } else {
                    for arg in arguments {
                        argument_types.push(
                            self.infer_and_validate_expression_immutable_with_cache(
                                arg,
                                array_types,
                            )?,
                        );
                    }
                }

                if self.is_closure_callable(name) {
                    Ok(Ty::Int)
                } else if self.function_table.get_admitted_contract(name).is_some() {
                    self.function_table
                        .validate_admitted_call(name, &argument_types)
                } else if self.function_table.get_function(name).is_some() {
                    Ok(Ty::Int)
                } else {
                    Err(format!("Error: Function `{}` is not defined.", name))
                }
            }
            Expression::Print {
                format_string,
                arguments,
            } => {
                self.validate_format_string_and_args_immutable(format_string, arguments)?;
                Ok(Ty::Int)
            }
            Expression::Println {
                format_string,
                arguments,
            } => {
                self.validate_format_string_and_args_immutable(format_string, arguments)?;
                Ok(Ty::Int)
            }
            Expression::Comparison { op, left, right } => {
                let left_type =
                    self.infer_and_validate_expression_immutable_with_cache(left, array_types)?;
                let right_type =
                    self.infer_and_validate_expression_immutable_with_cache(right, array_types)?;
                self.validate_comparison_operands(op, &left_type, &right_type)?;
                Ok(Ty::Bool)
            }
            Expression::Logical { op, left, right } => {
                let left_type =
                    self.infer_and_validate_expression_immutable_with_cache(left, array_types)?;
                let right_type =
                    self.infer_and_validate_expression_immutable_with_cache(right, array_types)?;
                self.validate_logical_operands(op, &left_type, &right_type)?;
                Ok(Ty::Bool)
            }
            Expression::Unary { operand, op } => {
                let operand_type =
                    self.infer_and_validate_expression_immutable_with_cache(operand, array_types)?;
                self.validate_unary_operation(op, &operand_type)
            }
            // Phase 4 expressions
            Expression::StringLiteral(_) => Ok(Ty::String),
            Expression::MethodCall { object, method, .. } => {
                let obj_ty =
                    self.infer_and_validate_expression_immutable_with_cache(object, array_types)?;
                // Phase 6: Option, Result, Vec, HashMap methods
                match &obj_ty {
                    Ty::Option(inner) => match method.as_str() {
                        "is_some" | "is_none" => Ok(Ty::Bool),
                        "unwrap" | "expect" | "unwrap_or" | "unwrap_or_else" => Ok(*inner.clone()),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::Result(ok_ty, err_ty) => match method.as_str() {
                        "is_ok" | "is_err" => Ok(Ty::Bool),
                        "unwrap" | "expect" | "unwrap_or" | "unwrap_or_else" => Ok(*ok_ty.clone()),
                        "unwrap_err" | "expect_err" => Ok(*err_ty.clone()),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::Vec(elem) => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" => Ok(Ty::Bool),
                        "push" | "clear" => Ok(Ty::Void),
                        "pop" | "first" | "last" | "get" => Ok(Ty::Option(elem.clone())),
                        "iter" => Ok(Ty::Vec(elem.clone())),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::Array(elem, size) => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" => Ok(Ty::Bool),
                        "first" | "last" | "get" => Ok(Ty::Option(elem.clone())),
                        "iter" => Ok(Ty::Array(elem.clone(), *size)),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::HashMap(_, val) => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" | "contains_key" => Ok(Ty::Bool),
                        "insert" | "remove" | "clear" => Ok(Ty::Void),
                        "get" => Ok(Ty::Option(val.clone())),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::String => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" | "contains" | "starts_with" | "ends_with" => Ok(Ty::Bool),
                        "to_uppercase" | "to_lowercase" | "trim" | "trim_start" | "trim_end" => {
                            Ok(Ty::String)
                        }
                        "chars" => Ok(Ty::Vec(Box::new(Ty::Int))), // char as int
                        _ => Ok(Ty::Int),                          // Unknown method
                    },
                    _ => Ok(Ty::Int), // Other method calls - stub
                }
            }
            Expression::ArrayLiteral(elements) => {
                if self.type_param_scopes.is_empty() {
                    let cache_key = expr as *const Expression;
                    if let Some(ty) = array_types.get(&cache_key) {
                        return Ok(ty.clone());
                    }
                    let ty = self.infer_non_generic_array_literal(elements, array_types)?;
                    array_types.insert(cache_key, ty.clone());
                    return Ok(ty);
                }
                let Some(first) = elements.first() else {
                    return Ok(Ty::Array(Box::new(Ty::Int), 0));
                };
                let elem_type =
                    self.infer_and_validate_expression_immutable_with_cache(first, array_types)?;
                Ok(Ty::Array(Box::new(elem_type), elements.len()))
            }
            Expression::ArrayRepeat { value, count } => {
                let elem_type =
                    self.infer_and_validate_expression_immutable_with_cache(value, array_types)?;
                Ok(Ty::Array(Box::new(elem_type), *count))
            }
            Expression::IndexAccess { object, index } => {
                let obj_type =
                    self.infer_and_validate_expression_immutable_with_cache(object, array_types)?;
                let index_type =
                    self.infer_and_validate_expression_immutable_with_cache(index, array_types)?;
                if self.type_param_scopes.is_empty()
                    && matches!(&obj_type, Ty::Array(element, _) if Self::is_numeric_type(element) || self.struct_registry.is_copy_struct_ty(element))
                    && index_type != Ty::Int
                {
                    return Err(format!(
                        "Error: array index type mismatch: expected int, actual {}.",
                        index_type
                    ));
                }
                if is_statically_empty_fixed_array(&obj_type) {
                    return Err("Error: cannot index a zero-length fixed array.".to_string());
                }
                match self
                    .struct_registry
                    .classify_copy_struct_array_index(&obj_type, index)
                {
                    CopyStructArrayIndexDisposition::PreserveExistingBehavior
                    | CopyStructArrayIndexDisposition::Accepted { .. } => {}
                    CopyStructArrayIndexDisposition::NonConstant => {
                        return Err(
                            "fixed Copy-struct array index must be a compile-time integer constant"
                                .to_string(),
                        );
                    }
                    CopyStructArrayIndexDisposition::OutOfBounds { index, count } => {
                        return Err(format!(
                            "fixed Copy-struct array index {index} is outside 0..{count}"
                        ));
                    }
                }
                match obj_type {
                    Ty::Array(elem, _) => Ok(*elem),
                    _ => Err("Cannot index into non-array type".to_string()),
                }
            }
            Expression::FieldAccess { object, field } => {
                self.infer_supported_field_type(object, field, array_types)
            }
            Expression::TupleLiteral(_) | Expression::TupleIndex { .. } => Ok(Ty::Int), // Stub
            Expression::StructLiteral { name, fields } => {
                let resolved = self
                    .struct_registry
                    .resolve_construction(name, fields, self.struct_execution_context())
                    .map_err(|error| error.diagnostic())?;
                let mut actual_types = Vec::with_capacity(fields.len());
                for (source_index, (_, value)) in fields.iter().enumerate() {
                    let expected =
                        resolved.contract.fields[resolved.source_to_declaration[source_index]].ty();
                    let actual = if matches!(
                        (value, &expected),
                        (Expression::ArrayLiteral(elements), Ty::Array(_, 0)) if elements.is_empty()
                    ) {
                        expected
                    } else {
                        self.infer_and_validate_expression_immutable_with_cache(value, array_types)?
                    };
                    actual_types.push(actual);
                }
                self.struct_registry
                    .validate_construction_types(&resolved, &actual_types)
                    .map_err(|error| error.diagnostic())?;
                Ok(Ty::Struct(resolved.contract.name))
            }
            // Phase 6: Special handling for Option and Result constructors
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } => match enum_name.as_str() {
                "Option" => match variant.as_str() {
                    "Some" => {
                        if let Some(inner_expr) = data {
                            let inner_ty = self
                                .infer_and_validate_expression_immutable_with_cache(
                                    inner_expr,
                                    array_types,
                                )?;
                            Ok(Ty::Option(Box::new(inner_ty)))
                        } else {
                            Err("Some variant requires a value".to_string())
                        }
                    }
                    "None" => Ok(Ty::Option(Box::new(Ty::Int))),
                    _ => Err(format!("Unknown Option variant: {}", variant)),
                },
                "Result" => match variant.as_str() {
                    "Ok" => {
                        if let Some(inner_expr) = data {
                            let inner_ty = self
                                .infer_and_validate_expression_immutable_with_cache(
                                    inner_expr,
                                    array_types,
                                )?;
                            Ok(Ty::Result(Box::new(inner_ty), Box::new(Ty::String)))
                        } else {
                            Err("Ok variant requires a value".to_string())
                        }
                    }
                    "Err" => {
                        if let Some(inner_expr) = data {
                            let inner_ty = self
                                .infer_and_validate_expression_immutable_with_cache(
                                    inner_expr,
                                    array_types,
                                )?;
                            Ok(Ty::Result(Box::new(Ty::Int), Box::new(inner_ty)))
                        } else {
                            Err("Err variant requires a value".to_string())
                        }
                    }
                    _ => Err(format!("Unknown Result variant: {}", variant)),
                },
                _ if self.enum_execution_context() == EnumExecutionContext::AdmittedFunction => {
                    let actual = data
                        .as_deref()
                        .map(|payload| {
                            self.infer_and_validate_expression_immutable_with_cache(
                                payload,
                                array_types,
                            )
                        })
                        .transpose()?;
                    let resolved = self
                        .enum_registry
                        .resolve_constructor(
                            enum_name,
                            variant,
                            data.is_some(),
                            self.enum_execution_context(),
                        )
                        .map_err(|error| error.diagnostic())?;
                    self.enum_registry
                        .validate_constructor_payload(&resolved, actual.as_ref())
                        .map_err(|error| error.diagnostic())?;
                    Ok(resolved.contract.ty())
                }
                _ => Ok(Ty::Enum(enum_name.clone())),
            },
            Expression::Match { expr, arms } => {
                let scrutinee =
                    self.infer_and_validate_expression_immutable_with_cache(expr, array_types)?;
                let patterns = self
                    .enum_registry
                    .resolve_match_patterns(&scrutinee, expr, arms, self.enum_execution_context())
                    .map_err(|error| error.diagnostic())?;
                let mut result_types = Vec::with_capacity(arms.len());
                for (arm, binding) in arms.iter().zip(patterns.payload_bindings.iter()) {
                    if let Some(binding) = binding {
                        self.push_enum_payload_binding(binding.name.clone(), binding.ty.clone());
                    }
                    let result = self
                        .infer_and_validate_expression_immutable_with_cache(&arm.body, array_types);
                    if binding.is_some() {
                        self.pop_enum_payload_binding();
                    }
                    result_types.push(result?);
                }
                let consumed = self.enum_consumed_names(expr)?;
                self.enum_registry
                    .resolve_match_with_consumed(
                        &scrutinee,
                        expr,
                        arms,
                        &result_types,
                        &consumed,
                        self.enum_execution_context(),
                    )
                    .map(|resolved| resolved.result)
                    .map_err(|error| error.diagnostic())
            }
            // Phase 5: Borrow and Deref
            Expression::Borrow { expr, mutable } => {
                let facts = self.local_reference_source_facts(expr);
                if facts.is_none() {
                    self.infer_and_validate_expression_immutable_with_cache(expr, array_types)?;
                }
                match classify_local_borrow(expr, *mutable, facts.as_ref()) {
                    LocalReferenceDisposition::Supported(contract) => Ok(contract.reference_type()),
                    LocalReferenceDisposition::ExplicitlyRejected(message) => Err(message),
                    LocalReferenceDisposition::Preserved => unreachable!(
                        "borrow expressions are fully classified by the local reference contract"
                    ),
                }
            }
            Expression::Deref(expr) => {
                let inner_ty =
                    self.infer_and_validate_expression_immutable_with_cache(expr, array_types)?;
                match classify_local_dereference(&inner_ty) {
                    LocalReferenceDisposition::Supported(contract) => Ok(contract.pointee),
                    LocalReferenceDisposition::ExplicitlyRejected(message) => Err(message),
                    LocalReferenceDisposition::Preserved => unreachable!(
                        "dereference expressions are fully classified by the local reference contract"
                    ),
                }
            }
            // Phase 7: Closures
            Expression::Closure { .. } => Ok(Ty::Int), // Closure type inference stub
        }
    }

    #[allow(dead_code)]
    fn validate_format_string_and_args(
        &self,
        format_string: &str,
        arguments: &[Expression],
    ) -> Result<(), String> {
        let placeholder_count = format_string.matches("{}").count();

        if placeholder_count != arguments.len() {
            return Err(format!(
                "Error: Format string has {} placeholders but {} arguments were provided.",
                placeholder_count,
                arguments.len()
            ));
        }

        for (i, arg) in arguments.iter().enumerate() {
            let arg_type = self.infer_and_validate_expression_immutable(arg)?;
            if !self.is_printable_type(&arg_type) {
                return Err(format!(
                    "Error: Argument {} of type `{}` is not printable.",
                    i + 1,
                    arg_type
                ));
            }
        }

        Ok(())
    }

    fn validate_format_string_and_args_immutable(
        &self,
        format_string: &str,
        arguments: &[Expression],
    ) -> Result<(), String> {
        let placeholder_count = format_string.matches("{}").count();

        if placeholder_count != arguments.len() {
            return Err(format!(
                "Error: Format string has {} placeholders but {} arguments were provided.",
                placeholder_count,
                arguments.len()
            ));
        }

        for (i, arg) in arguments.iter().enumerate() {
            let arg_type = self.infer_and_validate_expression_immutable(arg)?;
            if !self.is_printable_type(&arg_type) {
                return Err(format!(
                    "Error: Argument {} of type `{}` is not printable.",
                    i + 1,
                    arg_type
                ));
            }
        }

        Ok(())
    }

    fn is_printable_type(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Int | Ty::Float | Ty::Bool | Ty::String)
    }

    fn validate_comparison_operands(
        &self,
        _op: &ComparisonOp,
        left_type: &Ty,
        right_type: &Ty,
    ) -> Result<(), String> {
        if left_type == right_type
            || (left_type == &Ty::Int && right_type == &Ty::Float)
            || (left_type == &Ty::Float && right_type == &Ty::Int)
        {
            Ok(()) // Allow same-type comparisons and int/float comparisons
        } else {
            Err(format!(
                "Error: Cannot compare types `{}` and `{}`.",
                left_type, right_type
            ))
        }
    }

    fn validate_logical_operands(
        &self,
        _op: &LogicalOp,
        left_type: &Ty,
        right_type: &Ty,
    ) -> Result<(), String> {
        if left_type != &Ty::Bool {
            return Err(format!(
                "Error: Left operand of logical operation must be boolean, found: {}",
                left_type
            ));
        }
        if right_type != &Ty::Bool {
            return Err(format!(
                "Error: Right operand of logical operation must be boolean, found: {}",
                right_type
            ));
        }
        Ok(())
    }

    fn validate_unary_operation(&self, op: &UnaryOp, operand_type: &Ty) -> Result<Ty, String> {
        match op {
            UnaryOp::Not => {
                if operand_type == &Ty::Bool {
                    Ok(Ty::Bool)
                } else {
                    Err(format!(
                        "Error: Logical NOT operator requires boolean operand, found: {}",
                        operand_type
                    ))
                }
            }
            UnaryOp::Negate => {
                if operand_type == &Ty::Int || operand_type == &Ty::Float {
                    Ok(operand_type.clone())
                } else {
                    Err(format!(
                        "Error: Unary minus operator requires numeric operand, found: {}",
                        operand_type
                    ))
                }
            }
        }
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        self.analyze_statement_with_context(stmt, false)
    }

    fn analyze_statement_with_context(
        &mut self,
        stmt: &Statement,
        is_top_level: bool,
    ) -> Result<(), String> {
        match stmt {
            Statement::Let {
                name,
                mutable,
                type_annotation,
                value,
            } => {
                if self.scope_manager.variable_exists_in_current_scope(name) {
                    return Err(format!(
                        "Error: Variable `{}` is already defined in this scope.",
                        name
                    ));
                }

                let disposition = type_annotation.as_ref().map_or(
                    BindingAnnotationDisposition::PreserveExistingBehavior,
                    |annotation| classify_binding_annotation(annotation, value.is_some()),
                );

                if value.is_none()
                    && let BindingAnnotationDisposition::ExistingExplicitRejection(kind) =
                        disposition
                {
                    return Err(format!(
                        "Error: Variable `{}` uses an unsupported {} for an uninitialized binding.",
                        name,
                        kind.topology()
                    ));
                }

                let inferred_type = if let Some(val) = value {
                    self.check_expression_initialization(val)?;
                    let inferred = self.require_value(val)?;
                    if self.type_param_scopes.is_empty() && !self.inside_impl {
                        if let Some(contract) = type_annotation.as_ref().and_then(|annotation| {
                            self.struct_registry
                                .typed_empty_copy_struct_array_contract(annotation, val)
                        }) {
                            contract.ty()
                        } else {
                            type_annotation
                                .as_ref()
                                .and_then(|annotation| {
                                    typed_empty_numeric_array_contract(annotation, val)
                                })
                                .map_or(inferred, |contract| contract.ty())
                        }
                    } else {
                        inferred
                    }
                } else {
                    Ty::Int
                };

                if let Some(initializer) = value {
                    self.struct_registry
                        .validate_copy_struct_array_binding(
                            type_annotation.as_ref(),
                            &inferred_type,
                        )
                        .map_err(|error| error.diagnostic())?;
                    self.struct_registry
                        .validate_direct_binding_initializer(initializer, &inferred_type)
                        .map_err(|error| error.diagnostic())?;
                    if let Ty::Struct(struct_name) = &inferred_type {
                        self.struct_registry
                            .validate_binding_annotation(struct_name, type_annotation.as_ref())
                            .map_err(|error| error.diagnostic())?;
                    }
                    if let Ty::Enum(enum_name) = &inferred_type {
                        self.enum_registry
                            .validate_binding(enum_name, type_annotation.as_ref(), *mutable)
                            .map_err(|error| error.diagnostic())?;
                    }
                }

                if value.is_some()
                    && let BindingAnnotationDisposition::ExistingExplicitRejection(kind) =
                        disposition
                {
                    return Err(format!(
                        "Error: Variable `{}` uses an unsupported {} for an initialized binding.",
                        name,
                        kind.topology()
                    ));
                }

                let reference_annotation = if matches!(inferred_type, Ty::Reference(_, _)) {
                    type_annotation.as_ref().map_or(
                        LocalReferenceDisposition::Preserved,
                        |annotation| {
                            classify_local_reference_annotation(annotation, value.is_some())
                        },
                    )
                } else {
                    LocalReferenceDisposition::Preserved
                };
                if let LocalReferenceDisposition::ExplicitlyRejected(message) = reference_annotation
                {
                    return Err(message);
                }

                let binding_type = if value.is_some()
                    && let BindingAnnotationDisposition::MatchesExistingContractShape(contract) =
                        disposition
                    && (self.type_param_scopes.is_empty() || contract.is_numeric_scalar())
                {
                    let expected_type = contract.ty();
                    if inferred_type != expected_type {
                        return Err(format!(
                            "Error: Variable `{}` type annotation mismatch: expected {}, actual {}.",
                            name, expected_type, inferred_type
                        ));
                    }
                    expected_type
                } else if value.is_some()
                    && let LocalReferenceDisposition::Supported(contract) = reference_annotation
                {
                    let expected_type = contract.reference_type();
                    if inferred_type != expected_type {
                        return Err(format!(
                            "Error: Variable `{}` type annotation mismatch: expected {}, actual {}.",
                            name, expected_type, inferred_type
                        ));
                    }
                    expected_type
                } else {
                    inferred_type
                };

                if let Some(initializer) = value {
                    classify_mutable_reference_binding(initializer, &binding_type)?;
                }

                // Phase 5: Track ownership transfers and borrows.
                let mut borrow_origin = None;
                if let Some(val_expr) = value {
                    self.apply_enum_match_moves(val_expr)?;
                    match val_expr {
                        // Move semantics: let x = y (non-Copy type moves)
                        Expression::Identifier(source_name) => {
                            if !self.is_copy_type(&binding_type) {
                                self.scope_manager.mark_moved(source_name)?;
                            }
                        }
                        // Borrow checking: let r = &x or let r = &mut x
                        Expression::Borrow {
                            expr,
                            mutable: is_mut_borrow,
                        } => {
                            if let Expression::Identifier(source_name) = expr.as_ref() {
                                self.scope_manager
                                    .record_classified_borrow(source_name, *is_mut_borrow);
                                borrow_origin = Some((source_name.clone(), *is_mut_borrow));
                            }
                        }
                        _ => {}
                    }
                }

                self.scope_manager.define_variable(
                    name.clone(),
                    binding_type.clone(),
                    *mutable,
                    value.is_some(),
                )?;
                if let Some((source, mutable)) = borrow_origin {
                    self.scope_manager
                        .set_borrow_origin(name, source, mutable)?;
                }

                // Also add to old symbol table for backward compatibility
                let var_info = VariableInfo {
                    name: name.clone(),
                    ty: binding_type.clone(),
                    mutable: *mutable,
                    initialized: value.is_some(),
                };
                self.symbol_table.insert(name.clone(), var_info);

                if matches!(value, Some(Expression::Closure { .. })) {
                    self.closure_binding_scopes
                        .last_mut()
                        .expect("global closure-binding scope must exist")
                        .insert(name.clone());
                }

                Ok(())
            }
            Statement::Assignment { target, value } => {
                self.check_expression_initialization(value)?;
                let rhs = self.require_value(value)?;
                let inside_admitted_function = self.scope_manager.is_in_function()
                    && self.type_param_scopes.is_empty()
                    && !self.inside_impl;
                let mutable_reference_facts = if let Expression::Deref(reference) = target
                    && let Expression::Identifier(name) = reference.as_ref()
                {
                    self.scope_manager.get_variable(name).map(|variable| {
                        MutableReferenceAssignmentFacts {
                            ty: variable.var_type.clone(),
                            initialized: variable.initialized,
                            local: variable.scope_level > 0,
                            ownership: variable.ownership.clone(),
                        }
                    })
                } else {
                    None
                };
                match classify_mutable_reference_assignment(
                    target,
                    mutable_reference_facts.as_ref(),
                    &rhs,
                    inside_admitted_function,
                ) {
                    MutableReferenceAssignmentDisposition::Supported(_) => {
                        self.apply_enum_match_moves(value)?;
                        return Ok(());
                    }
                    MutableReferenceAssignmentDisposition::ExplicitlyRejected(message) => {
                        return Err(message);
                    }
                    MutableReferenceAssignmentDisposition::Preserved => {}
                }
                let facts = if let Expression::Identifier(name) = target {
                    self.scope_manager.get_variable(name).map(|variable| {
                        ScalarAssignmentTargetFacts {
                            ty: variable.var_type.clone(),
                            mutable: variable.mutable,
                            initialized: variable.initialized,
                            local: variable.scope_level > 0,
                            ownership: variable.ownership.clone(),
                        }
                    })
                } else {
                    None
                };
                match classify_scalar_assignment(
                    Some(target),
                    facts.as_ref(),
                    &rhs,
                    inside_admitted_function,
                ) {
                    ScalarAssignmentDisposition::Supported(_) => {
                        self.apply_enum_match_moves(value)?;
                        Ok(())
                    }
                    ScalarAssignmentDisposition::ExplicitlyRejected(message) => Err(message),
                    ScalarAssignmentDisposition::PreserveExistingBehavior => {
                        unreachable!("explicit assignment must receive a classifier disposition")
                    }
                }
            }
            Statement::Return(expr) => {
                let contract = self
                    .return_contract_stack
                    .last()
                    .and_then(|entry| entry.as_ref())
                    .cloned();
                if let Some(contract) = contract {
                    let actual = if let Some(val) = expr {
                        self.check_expression_initialization(val)?;
                        self.infer_and_validate_expression_immutable(val)?
                    } else {
                        Ty::Void
                    };
                    if actual != contract.return_type {
                        return Err(Self::return_type_mismatch(
                            &contract.name,
                            &contract.return_type,
                            &actual,
                        ));
                    }
                } else if let Some(val) = expr {
                    self.check_expression_initialization(val)?;
                    self.infer_and_validate_expression_immutable(val)?;
                }
                if let Some(val) = expr {
                    self.apply_enum_match_moves(val)?;
                }
                Ok(())
            }
            Statement::Function {
                name,
                parameters,
                body,
                return_type: _,
                type_params,
                trait_bounds,
            } => {
                // Phase 5: Register generic type parameters in scope
                if !type_params.is_empty() {
                    self.type_param_scopes.push(type_params.clone());
                }
                // Phase 5: Store trait bounds for this function
                if !trait_bounds.is_empty() {
                    self.function_bounds
                        .insert(name.clone(), trait_bounds.clone());
                }

                let return_contract = if is_top_level {
                    self.function_table.get_admitted_contract(name).cloned()
                } else {
                    None
                };
                self.return_contract_stack.push(return_contract.clone());
                let struct_context = if is_top_level && type_params.is_empty() && !self.inside_impl
                {
                    StructExecutionContext::AdmittedFunction
                } else {
                    StructExecutionContext::PreservedContext
                };
                self.struct_execution_stack.push(struct_context);

                // Enter a new scope for the function body
                self.enter_function_scope(name.clone());

                let body_result = (|| {
                    // Declare parameters as variables in the function scope
                    for (index, param) in parameters.iter().enumerate() {
                        let param_type = return_contract.as_ref().map_or_else(
                            || self.ast_type_to_ty(&param.param_type),
                            |contract| contract.parameters[index].1.clone(),
                        );
                        self.scope_manager.define_variable(
                            param.name.clone(),
                            param_type.clone(),
                            false,
                            true, // Parameters are always initialized
                        )?;
                        // Also add to old symbol table for backward compatibility
                        let var_info = VariableInfo {
                            name: param.name.clone(),
                            ty: param_type,
                            mutable: false,
                            initialized: true,
                        };
                        self.symbol_table.insert(param.name.clone(), var_info);
                    }

                    self.analyze_block(body)?;

                    if let Some(contract) = &return_contract {
                        if let Some(tail) = &body.expression {
                            let actual = self.infer_and_validate_expression_immutable(tail)?;
                            if actual != contract.return_type {
                                return Err(Self::return_type_mismatch(
                                    &contract.name,
                                    &contract.return_type,
                                    &actual,
                                ));
                            }
                        }

                        if contract.return_type != Ty::Void
                            && !Self::function_body_definitely_returns(body)
                        {
                            return Err(format!(
                                "Error: Function `{}` must return {} on all paths.",
                                contract.name,
                                Self::contract_type_name(&contract.return_type)
                            ));
                        }
                    }

                    Ok(())
                })();

                // Exit the function scope
                self.exit_function_scope();
                self.return_contract_stack.pop();
                self.struct_execution_stack.pop();

                // Pop generic type parameters
                if !type_params.is_empty() {
                    self.type_param_scopes.pop();
                }

                body_result
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.check_expression_initialization(condition)?;
                let condition_type = self.infer_and_validate_expression_immutable(condition)?;

                if condition_type != Ty::Bool {
                    return Err(format!(
                        "Error: If condition must be boolean, found: {}",
                        condition_type
                    ));
                }
                self.apply_enum_match_moves(condition)?;

                self.enter_scope();
                let then_result = self.analyze_block(then_block);
                self.exit_scope();
                then_result?;

                if let Some(else_stmt) = else_block {
                    self.enter_scope();
                    let else_result = self.analyze_statement(else_stmt);
                    self.exit_scope();
                    else_result?;
                }

                Ok(())
            }
            Statement::While { condition, body } => {
                self.check_expression_initialization(condition)?;
                let condition_type = self.infer_and_validate_expression_immutable(condition)?;

                if condition_type != Ty::Bool {
                    return Err(format!(
                        "Error: While condition must be boolean, found: {}",
                        condition_type
                    ));
                }
                self.apply_enum_match_moves(condition)?;

                self.enter_loop_scope();
                let body_result = self.analyze_block(body);
                self.exit_loop_scope();
                body_result?;

                Ok(())
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                self.check_expression_initialization(iterable)?;
                let iterable_type = self.infer_and_validate_expression_immutable(iterable)?;
                let loop_var_type = self
                    .infer_into_iterator_item_type(&iterable_type)
                    .ok_or_else(|| {
                        format!(
                            "Error: For-loop iterable must implement IntoIterator (supported: arrays, Vec, integer range start), found: {}",
                            iterable_type
                        )
                    })?;
                self.apply_enum_match_moves(iterable)?;

                self.enter_loop_scope();
                let body_result = (|| {
                    self.scope_manager.define_variable(
                        variable.clone(),
                        loop_var_type,
                        false,
                        true,
                    )?;
                    self.analyze_block(body)
                })();
                self.exit_loop_scope();
                body_result?;

                Ok(())
            }
            Statement::Loop { body } => {
                self.enter_loop_scope();
                let body_result = self.analyze_block(body);
                self.exit_loop_scope();
                body_result?;
                Ok(())
            }
            Statement::Break => {
                if !self.scope_manager.can_break_continue() {
                    return Err("Error: Break statement outside of loop.".to_string());
                }
                Ok(())
            }
            Statement::Continue => {
                if !self.scope_manager.can_break_continue() {
                    return Err("Error: Continue statement outside of loop.".to_string());
                }
                Ok(())
            }
            Statement::Expression(expr) => {
                self.check_expression_initialization(expr)?;
                self.infer_discarded_expression_immutable(expr)?;
                self.apply_enum_match_moves(expr)?;
                // Phase 5: Track moves for non-Copy function call arguments
                self.track_expression_moves(expr)?;
                // Phase 5: Check trait bounds at function call sites
                self.check_trait_bounds_at_call(expr)?;
                Ok(())
            }
            Statement::Block(block) => {
                self.enter_scope();
                let block_result = self.analyze_block(block);
                self.exit_scope();
                block_result?;
                Ok(())
            }
            // Phase 4/5: type definitions
            Statement::StructDef { type_params, .. } | Statement::EnumDef { type_params, .. } => {
                if !type_params.is_empty() {
                    self.type_param_scopes.push(type_params.clone());
                }
                if !type_params.is_empty() {
                    self.type_param_scopes.pop();
                }
                Ok(())
            }
            Statement::TraitDef {
                name,
                type_params,
                methods,
            } => {
                // Register trait in registry with its required method names
                let required_methods: Vec<String> = methods
                    .iter()
                    .filter(|m| m.body.is_none()) // Only methods without default impl are required
                    .map(|m| m.name.clone())
                    .collect();
                self.trait_registry.insert(name.clone(), required_methods);
                if !type_params.is_empty() {
                    self.type_param_scopes.push(type_params.clone());
                }
                self.struct_execution_stack
                    .push(StructExecutionContext::PreservedContext);
                let default_body_result = (|| {
                    for method in methods {
                        if let Some(body) = &method.body {
                            self.preflight_block_syntax(body)?;
                        }
                    }
                    Ok(())
                })();
                self.struct_execution_stack.pop();
                if !type_params.is_empty() {
                    self.type_param_scopes.pop();
                }
                default_body_result
            }
            Statement::ImplBlock {
                type_name,
                methods,
                type_params,
                trait_name,
            } => {
                if !type_params.is_empty() {
                    self.type_param_scopes.push(type_params.clone());
                }
                let previous_inside_impl = self.inside_impl;
                self.inside_impl = true;
                let impl_result = (|| {
                    // Analyze method bodies
                    for method in methods {
                        self.analyze_statement(method)?;
                    }
                    // Phase 5: Check trait completeness if this is an impl Trait for Type
                    if let Some(trait_name) = trait_name {
                        // Register that this type implements this trait
                        self.trait_impls
                            .entry(type_name.clone())
                            .or_default()
                            .push(trait_name.clone());
                        // Check all required methods are implemented
                        if let Some(required_methods) = self.trait_registry.get(trait_name) {
                            let implemented: Vec<String> = methods
                                .iter()
                                .filter_map(|m| {
                                    if let Statement::Function { name, .. } = m {
                                        Some(name.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            for required in required_methods {
                                if !implemented.contains(required) {
                                    return Err(format!(
                                        "Error: Method `{}` is required by trait `{}` but not implemented for `{}`.",
                                        required, trait_name, type_name
                                    ));
                                }
                            }
                        }
                    }
                    Ok(())
                })();
                self.inside_impl = previous_inside_impl;
                if !type_params.is_empty() {
                    self.type_param_scopes.pop();
                }
                impl_result
            }
            // Phase 7: Module system
            Statement::ModDecl { .. } | Statement::UseImport { .. } => {
                // Module declarations and imports are resolved during module linking.
                Ok(())
            }
        }
    }

    fn ast_type_to_ty(&self, ty: &crate::ast::Type) -> Ty {
        match ty {
            crate::ast::Type::Named(name) => match name.as_str() {
                "i32" | "int" => Ty::Int,
                "f64" | "float" => Ty::Float,
                "bool" => Ty::Bool,
                "String" => Ty::String,
                other => {
                    // Phase 5: Check if this is a generic type parameter
                    if self.is_type_param(other) {
                        Ty::TypeParam(other.to_string())
                    } else {
                        Ty::Struct(other.to_string())
                    }
                }
            },
            crate::ast::Type::Array(elem, size) => {
                Ty::Array(Box::new(self.ast_type_to_ty(elem)), *size)
            }
            crate::ast::Type::Tuple(types) => {
                Ty::Tuple(types.iter().map(|t| self.ast_type_to_ty(t)).collect())
            }
            crate::ast::Type::Reference(inner, mutable) => {
                Ty::Reference(Box::new(self.ast_type_to_ty(inner)), *mutable)
            }
            // Phase 6: Standard library types Option<T>, Result<T, E>, Vec<T>, HashMap<K, V>
            crate::ast::Type::Generic(name, type_args) => {
                match name.as_str() {
                    "Option" if type_args.len() == 1 => {
                        let inner_ty = self.ast_type_to_ty(&type_args[0]);
                        Ty::Option(Box::new(inner_ty))
                    }
                    "Result" if type_args.len() == 2 => {
                        let ok_ty = self.ast_type_to_ty(&type_args[0]);
                        let err_ty = self.ast_type_to_ty(&type_args[1]);
                        Ty::Result(Box::new(ok_ty), Box::new(err_ty))
                    }
                    "Vec" if type_args.len() == 1 => {
                        let elem_ty = self.ast_type_to_ty(&type_args[0]);
                        Ty::Vec(Box::new(elem_ty))
                    }
                    "HashMap" if type_args.len() == 2 => {
                        let key_ty = self.ast_type_to_ty(&type_args[0]);
                        let val_ty = self.ast_type_to_ty(&type_args[1]);
                        Ty::HashMap(Box::new(key_ty), Box::new(val_ty))
                    }
                    _ => {
                        // Other generic types - treat as type parameter for now
                        Ty::TypeParam(name.clone())
                    }
                }
            }
        }
    }

    /// Check trait bounds at function call sites.
    /// If a function has bounds like T: Display, verify the argument type implements Display.
    fn check_trait_bounds_at_call(&self, expr: &Expression) -> Result<(), String> {
        if let Expression::FunctionCall { name, arguments } = expr {
            if let Some(bounds) = self.function_bounds.get(name) {
                // For each bound, check that the corresponding argument's type
                // has the required trait implementation.
                // Simple heuristic: map type params to argument types by position.
                // Simple heuristic: map type params to argument types by position.
                for (param_name, required_traits) in bounds {
                    // Find which argument corresponds to this type param
                    // by looking at which param position uses this type param
                    if let Some((i, arg)) = arguments.iter().enumerate().next() {
                        // If the arg is at position i, and we know the function
                        // param at position i has type `param_name`, check bounds
                        let arg_type = self.infer_and_validate_expression_immutable(arg)?;
                        let type_name = match &arg_type {
                            Ty::Struct(name) => Some(name.clone()),
                            Ty::Enum(name) => Some(name.clone()),
                            _ => None,
                        };
                        if let Some(type_name) = type_name {
                            let impls = self.trait_impls.get(&type_name);
                            for required_trait in required_traits {
                                let has_impl = impls
                                    .map(|impls| impls.contains(required_trait))
                                    .unwrap_or(false);
                                if !has_impl {
                                    return Err(format!(
                                        "Error: Type `{}` does not implement trait `{}` required by `{}`.",
                                        type_name, required_trait, name
                                    ));
                                }
                            }
                        }
                        // Only check the first matching arg for simplicity
                        let _ = i;
                        let _ = param_name;
                    }
                }
            }
        }
        Ok(())
    }

    /// Track moves caused by non-Copy arguments in function calls and other expressions.
    fn track_expression_moves(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    if let Expression::Identifier(arg_name) = arg {
                        let arg_type = self.infer_and_validate_expression_immutable(arg)?;
                        if !matches!(arg_type, Ty::Enum(_)) && !self.is_copy_type(&arg_type) {
                            self.scope_manager.mark_moved(arg_name)?;
                        }
                    }
                }
            }
            Expression::MethodCall { arguments, .. } => {
                for arg in arguments {
                    if let Expression::Identifier(arg_name) = arg {
                        let arg_type = self.infer_and_validate_expression_immutable(arg)?;
                        if !matches!(arg_type, Ty::Enum(_)) && !self.is_copy_type(&arg_type) {
                            self.scope_manager.mark_moved(arg_name)?;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn analyze_block(&mut self, block: &Block) -> Result<(), String> {
        for stmt in &block.statements {
            self.analyze_statement(stmt)?;
        }

        if let Some(expr) = &block.expression {
            self.check_expression_initialization(expr)?;
            self.infer_and_validate_expression_immutable(expr)?;
            self.apply_enum_match_moves(expr)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, Block, Expression, Parameter, Statement, Type};

    #[test]
    fn public_function_info_and_table_api_remain_source_compatible() {
        let mut table = FunctionTable::new();
        let info = FunctionInfo {
            name: "identity".to_string(),
            parameters: vec![Parameter {
                name: "value".to_string(),
                param_type: Type::Named("i32".to_string()),
            }],
            return_type: Ty::Int,
            defined_at: None,
        };

        table.define_function(info).expect("define function");
        assert_eq!(table.validate_call("identity", &[Ty::Int]), Ok(Ty::Int));
        assert!(table.validate_call("identity", &[]).is_err());
    }

    #[test]
    fn for_loop_over_array_iter_infers_element_type() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "arr".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::ArrayLiteral(vec![
                    Expression::IntegerLiteral(1),
                    Expression::IntegerLiteral(2),
                ])),
            }),
            AstNode::Statement(Statement::For {
                variable: "x".to_string(),
                iterable: Expression::MethodCall {
                    object: Box::new(Expression::Identifier("arr".to_string())),
                    method: "iter".to_string(),
                    arguments: vec![],
                },
                body: Block {
                    statements: vec![Statement::Expression(Expression::Println {
                        format_string: "{}".to_string(),
                        arguments: vec![Expression::Identifier("x".to_string())],
                    })],
                    expression: None,
                },
            }),
        ];

        assert!(analyzer.analyze(ast).is_ok());
    }

    #[test]
    fn println_accepts_string_arguments() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "name".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::StringLiteral("Aero".to_string())),
            }),
            AstNode::Statement(Statement::Expression(Expression::Println {
                format_string: "Hello, {}".to_string(),
                arguments: vec![Expression::Identifier("name".to_string())],
            })),
        ];

        assert!(analyzer.analyze(ast).is_ok());
    }
}
