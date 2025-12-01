//! Type definitions and type utilities for the Typhon type system.
//!
//! This module defines the core types supported by Typhon, including primitive types,
//! user-defined class types, function types, generic types, and compound types like
//! unions and tuples.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

use crate::common::SourceInfo;
use crate::typesystem::{TypeError, TypeErrorKind};

/// A unique identifier for a type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u64);

impl TypeId {
    /// Creates a new type ID.
    pub fn new(id: u64) -> Self {
        TypeId(id)
    }

    /// Returns the next type ID.
    pub fn next(&self) -> Self {
        TypeId(self.0 + 1)
    }
}

/// A type in the Typhon type system.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// A primitive type (int, float, bool, str, None).
    Primitive(PrimitiveType),

    /// A class type.
    Class(ClassType),

    /// A function type.
    Function(Rc<FunctionType>),

    /// A union of types.
    Union(UnionType),

    /// A tuple type.
    Tuple(TupleType),

    /// A list type.
    List(ListType),

    /// A type variable.
    TypeVar(TypeVar),

    /// A generic type.
    GenericInstance(Rc<GenericInstance>),

    /// The Any type (equivalent to Python's typing.Any).
    Any,

    /// The None type.
    None,

    /// The Never type (for functions that never return, or empty branches).
    Never,
}

impl Type {
    /// Attempts to get a reference to the inner FunctionType if this is a Function variant.
    pub fn as_function_type(&self) -> Option<&FunctionType> {
        match self {
            Type::Function(func_type) => Some(func_type),
            _ => None,
        }
    }

    /// Attempts to get a reference to the inner FunctionType if this is a Function variant.
    /// Returns an error with appropriate message if it's not a Function.
    pub fn try_as_function_type(
        &self,
        source_info: Option<SourceInfo>,
    ) -> Result<&FunctionType, TypeError> {
        self.as_function_type().ok_or_else(|| {
            TypeError::new(
                TypeErrorKind::TypeMismatch {
                    expected: "function".to_string(),
                    actual: self.to_string(),
                },
                source_info,
            )
        })
    }

    /// Creates a new primitive type.
    pub fn primitive(kind: PrimitiveTypeKind) -> Self {
        Type::Primitive(PrimitiveType::new(kind))
    }

    /// Creates a new class type.
    pub fn class(name: String, source_info: Option<SourceInfo>) -> Self {
        Type::Class(ClassType::new(name, source_info))
    }

    /// Creates a new function type.
    pub fn function(
        parameters: Vec<ParameterType>,
        return_type: Rc<Type>,
        source_info: Option<SourceInfo>,
    ) -> Self {
        Type::Function(Rc::new(FunctionType::new(parameters, return_type, source_info)))
    }

    /// Creates a new union type.
    pub fn union(types: Vec<Rc<Type>>) -> Self {
        Type::Union(UnionType::new(types))
    }

    /// Creates a new tuple type.
    pub fn tuple(element_types: Vec<Rc<Type>>) -> Self {
        Type::Tuple(TupleType::new(element_types))
    }

    /// Creates a new list type.
    pub fn list(element_type: Rc<Type>) -> Self {
        Type::List(ListType::new(element_type))
    }

    /// Creates a new type variable.
    pub fn type_var(name: String, constraints: Vec<Rc<Type>>) -> Self {
        Type::TypeVar(TypeVar::new(name, constraints))
    }

    /// Returns whether the type is a concrete type (not a type variable).
    pub fn is_concrete(&self) -> bool {
        !matches!(self, Type::TypeVar(_))
    }

    /// Returns the source information for the type, if available.
    pub fn source_info(&self) -> Option<SourceInfo> {
        match self {
            Type::Primitive(p) => p.source_info,
            Type::Class(c) => c.source_info,
            Type::Function(f) => f.source_info,
            Type::Union(u) => u.source_info,
            Type::Tuple(t) => t.source_info,
            Type::List(l) => l.source_info,
            Type::TypeVar(v) => v.source_info,
            Type::GenericInstance(g) => g.source_info,
            Type::Any | Type::None | Type::Never => None,
        }
    }
}

/// Implementation to convert from FunctionType to Type
impl From<Rc<FunctionType>> for Type {
    fn from(func_type: Rc<FunctionType>) -> Self {
        Type::Function(func_type)
    }
}

/// Implementation to convert from FunctionType to Type (non-reference counted version)
impl From<FunctionType> for Type {
    fn from(func_type: FunctionType) -> Self {
        Type::Function(Rc::new(func_type))
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(p) => write!(f, "{p}"),
            Type::Class(c) => write!(f, "{}", c.name),
            Type::Function(func) => write!(f, "{func}"),
            Type::Union(u) => write!(f, "{u}"),
            Type::Tuple(t) => write!(f, "{t}"),
            Type::List(l) => write!(f, "{l}"),
            Type::TypeVar(v) => write!(f, "{}", v.name),
            Type::GenericInstance(g) => write!(f, "{g}"),
            Type::Any => write!(f, "Any"),
            Type::None => write!(f, "None"),
            Type::Never => write!(f, "Never"),
        }
    }
}

/// A primitive type.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveType {
    /// The kind of primitive type.
    pub kind: PrimitiveTypeKind,
    /// Source information.
    pub source_info: Option<SourceInfo>,
}

impl PrimitiveType {
    /// Creates a new primitive type.
    pub fn new(kind: PrimitiveTypeKind) -> Self {
        Self { kind, source_info: None }
    }

    /// Creates a new primitive type with source information.
    pub fn with_source_info(kind: PrimitiveTypeKind, source_info: SourceInfo) -> Self {
        Self { kind, source_info: Some(source_info) }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            PrimitiveTypeKind::Int => write!(f, "int"),
            PrimitiveTypeKind::Float => write!(f, "float"),
            PrimitiveTypeKind::Bool => write!(f, "bool"),
            PrimitiveTypeKind::Str => write!(f, "str"),
            PrimitiveTypeKind::Bytes => write!(f, "bytes"),
        }
    }
}

/// The kind of primitive type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveTypeKind {
    /// Integer type.
    Int,
    /// Float type.
    Float,
    /// Boolean type.
    Bool,
    /// String type.
    Str,
    /// Bytes type.
    Bytes,
}

/// A class type.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassType {
    /// Name of the class.
    pub name: String,
    /// Fields of the class, mapping field names to their types.
    pub fields: HashMap<String, Rc<Type>>,
    /// Methods of the class, mapping method names to their types.
    pub methods: HashMap<String, Rc<FunctionType>>,
    /// Base classes.
    pub bases: Vec<Rc<Type>>,
    /// Generic parameters.
    pub generic_params: Vec<GenericParam>,
    /// Source information.
    pub source_info: Option<SourceInfo>,
}

impl ClassType {
    /// Creates a new class type.
    pub fn new(name: String, source_info: Option<SourceInfo>) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            methods: HashMap::new(),
            bases: Vec::new(),
            generic_params: Vec::new(),
            source_info,
        }
    }

    /// Adds a field to the class.
    pub fn add_field(&mut self, name: String, ty: Rc<Type>) {
        self.fields.insert(name, ty);
    }

    /// Adds a method to the class.
    pub fn add_method(&mut self, name: String, ty: Rc<FunctionType>) {
        self.methods.insert(name, ty);
    }

    /// Adds a base class.
    pub fn add_base(&mut self, base: Rc<Type>) {
        self.bases.push(base);
    }

    /// Adds a generic parameter.
    pub fn add_generic_param(&mut self, param: GenericParam) {
        self.generic_params.push(param);
    }

    /// Returns whether the class has generic parameters.
    pub fn is_generic(&self) -> bool {
        !self.generic_params.is_empty()
    }
}

/// A parameter type for a function.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterType {
    /// Name of the parameter.
    pub name: Option<String>,
    /// Type of the parameter.
    pub ty: Rc<Type>,
    /// Whether the parameter is optional.
    pub optional: bool,
    /// Default value, if any.
    pub default_value: Option<String>, // Serialized representation of the default value
}

impl ParameterType {
    /// Creates a new parameter type.
    pub fn new(name: Option<String>, ty: Rc<Type>, optional: bool) -> Self {
        Self { name, ty, optional, default_value: None }
    }

    /// Sets the default value for the parameter.
    pub fn with_default(mut self, default_value: String) -> Self {
        self.default_value = Some(default_value);
        self
    }
}

impl fmt::Display for ParameterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}: {}", name, self.ty)?;
        } else {
            write!(f, "{}", self.ty)?;
        }
        if self.optional {
            write!(f, " = ...")?;
        }
        Ok(())
    }
}

/// A function type.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    /// Parameters of the function.
    pub parameters: Vec<ParameterType>,
    /// Return type of the function.
    pub return_type: Rc<Type>,
    /// Source information.
    pub source_info: Option<SourceInfo>,
}

impl FunctionType {
    /// Creates a new function type.
    pub fn new(
        parameters: Vec<ParameterType>,
        return_type: Rc<Type>,
        source_info: Option<SourceInfo>,
    ) -> Self {
        Self { parameters, return_type, source_info }
    }
}

impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for param in &self.parameters {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{param}")?;
            first = false;
        }
        write!(f, ") -> {}", self.return_type)
    }
}

/// A union type.
#[derive(Debug, Clone, PartialEq)]
pub struct UnionType {
    /// Types in the union.
    pub types: Vec<Rc<Type>>,
    /// Source information.
    pub source_info: Option<SourceInfo>,
}

impl UnionType {
    /// Creates a new union type.
    pub fn new(types: Vec<Rc<Type>>) -> Self {
        Self { types, source_info: None }
    }

    /// Creates a new union type with source information.
    pub fn with_source_info(types: Vec<Rc<Type>>, source_info: SourceInfo) -> Self {
        Self { types, source_info: Some(source_info) }
    }

    /// Flattens nested union types.
    pub fn flatten(&self) -> Self {
        let mut flattened = Vec::new();

        for ty in &self.types {
            if let Type::Union(union) = ty.as_ref() {
                let nested_flattened = union.flatten();
                flattened.extend(nested_flattened.types.iter().cloned());
            } else {
                flattened.push(ty.clone());
            }
        }

        Self { types: flattened, source_info: self.source_info }
    }

    /// Simplifies the union by removing duplicates and handling special cases.
    pub fn simplify(&self) -> Rc<Type> {
        let flattened = self.flatten();

        // Remove duplicates
        let mut unique_types = Vec::new();
        let mut seen = HashSet::new();

        for ty in flattened.types {
            let type_str = format!("{ty}");
            if !seen.contains(&type_str) {
                seen.insert(type_str);
                unique_types.push(ty);
            }
        }

        // Handle special cases
        if unique_types.is_empty() {
            return Rc::new(Type::Never);
        } else if unique_types.len() == 1 {
            return unique_types[0].clone();
        }

        // Check for Any type
        for ty in &unique_types {
            if matches!(ty.as_ref(), Type::Any) {
                return Rc::new(Type::Any);
            }
        }

        Rc::new(Type::Union(UnionType { types: unique_types, source_info: flattened.source_info }))
    }
}

impl fmt::Display for UnionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.types.is_empty() {
            return write!(f, "Never");
        }

        let mut first = true;
        for ty in &self.types {
            if !first {
                write!(f, " | ")?;
            }
            write!(f, "{ty}")?;
            first = false;
        }
        Ok(())
    }
}

/// A tuple type.
#[derive(Debug, Clone, PartialEq)]
pub struct TupleType {
    /// Element types of the tuple.
    pub element_types: Vec<Rc<Type>>,
    /// Source information.
    pub source_info: Option<SourceInfo>,
}

impl TupleType {
    /// Creates a new tuple type.
    pub fn new(element_types: Vec<Rc<Type>>) -> Self {
        Self { element_types, source_info: None }
    }

    /// Creates a new tuple type with source information.
    pub fn with_source_info(element_types: Vec<Rc<Type>>, source_info: SourceInfo) -> Self {
        Self { element_types, source_info: Some(source_info) }
    }
}

impl fmt::Display for TupleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for ty in &self.element_types {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{ty}")?;
            first = false;
        }
        if self.element_types.len() == 1 {
            write!(f, ",")?;
        }
        write!(f, ")")
    }
}

/// A list type.
#[derive(Debug, Clone, PartialEq)]
pub struct ListType {
    /// Element type of the list.
    pub element_type: Rc<Type>,
    /// Source information.
    pub source_info: Option<SourceInfo>,
}

impl ListType {
    /// Creates a new list type.
    pub fn new(element_type: Rc<Type>) -> Self {
        Self { element_type, source_info: None }
    }

    /// Creates a new list type with source information.
    pub fn with_source_info(element_type: Rc<Type>, source_info: SourceInfo) -> Self {
        Self { element_type, source_info: Some(source_info) }
    }
}

impl fmt::Display for ListType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "list[{}]", self.element_type)
    }
}

/// A type variable.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeVar {
    /// Name of the type variable.
    pub name: String,
    /// Constraints on the type variable.
    pub constraints: Vec<Rc<Type>>,
    /// Source information.
    pub source_info: Option<SourceInfo>,
}

impl TypeVar {
    /// Creates a new type variable.
    pub fn new(name: String, constraints: Vec<Rc<Type>>) -> Self {
        Self { name, constraints, source_info: None }
    }

    /// Creates a new type variable with source information.
    pub fn with_source_info(
        name: String,
        constraints: Vec<Rc<Type>>,
        source_info: SourceInfo,
    ) -> Self {
        Self { name, constraints, source_info: Some(source_info) }
    }

    /// Returns whether the type variable has constraints.
    pub fn has_constraints(&self) -> bool {
        !self.constraints.is_empty()
    }
}

/// A generic parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    /// Name of the generic parameter.
    pub name: String,
    /// Constraints on the generic parameter.
    pub constraints: Vec<Rc<Type>>,
    /// Source information.
    pub source_info: Option<SourceInfo>,
}

impl GenericParam {
    /// Creates a new generic parameter.
    pub fn new(name: String, constraints: Vec<Rc<Type>>) -> Self {
        Self { name, constraints, source_info: None }
    }

    /// Creates a new generic parameter with source information.
    pub fn with_source_info(
        name: String,
        constraints: Vec<Rc<Type>>,
        source_info: SourceInfo,
    ) -> Self {
        Self { name, constraints, source_info: Some(source_info) }
    }

    /// Returns whether the generic parameter has constraints.
    pub fn has_constraints(&self) -> bool {
        !self.constraints.is_empty()
    }
}

/// A generic type instance.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericInstance {
    /// Base type.
    pub base: Rc<Type>,
    /// Type arguments.
    pub type_args: Vec<Rc<Type>>,
    /// Source information.
    pub source_info: Option<SourceInfo>,
}

impl GenericInstance {
    /// Creates a new generic type instance.
    pub fn new(base: Rc<Type>, type_args: Vec<Rc<Type>>) -> Self {
        Self { base, type_args, source_info: None }
    }

    /// Creates a new generic type instance with source information.
    pub fn with_source_info(
        base: Rc<Type>,
        type_args: Vec<Rc<Type>>,
        source_info: SourceInfo,
    ) -> Self {
        Self { base, type_args, source_info: Some(source_info) }
    }
}

impl fmt::Display for GenericInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base)?;
        write!(f, "[")?;
        let mut first = true;
        for arg in &self.type_args {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{arg}")?;
            first = false;
        }
        write!(f, "]")
    }
}

/// The type environment.
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// Mapping of variable names to types.
    variables: HashMap<String, Rc<Type>>,
    /// Type definitions.
    type_defs: HashMap<String, Rc<Type>>,
    /// Parent environment.
    pub parent: Option<Rc<TypeEnv>>,
    /// Next type ID.
    next_id: TypeId,
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeEnv {
    /// Creates a new type environment.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            type_defs: HashMap::new(),
            parent: None,
            next_id: TypeId::new(0),
        }
    }

    /// Creates a new child environment.
    pub fn new_child(parent: Rc<TypeEnv>) -> Self {
        Self {
            variables: HashMap::new(),
            type_defs: HashMap::new(),
            parent: Some(parent.clone()),
            next_id: parent.next_id,
        }
    }

    /// Adds a variable to the environment.
    pub fn add_variable(&mut self, name: String, ty: Rc<Type>) {
        self.variables.insert(name, ty);
    }

    /// Adds a type definition to the environment.
    pub fn add_type_def(&mut self, name: String, ty: Rc<Type>) {
        self.type_defs.insert(name, ty);
    }

    /// Gets a variable from the environment.
    pub fn get_variable(&self, name: &str) -> Option<Rc<Type>> {
        self.variables
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.get_variable(name)))
    }

    /// Gets a type definition from the environment.
    pub fn get_type_def(&self, name: &str) -> Option<Rc<Type>> {
        self.type_defs
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.get_type_def(name)))
    }

    /// Gets the next type ID and increments the counter.
    pub fn next_type_id(&mut self) -> TypeId {
        let id = self.next_id;
        self.next_id = self.next_id.next();
        id
    }

    /// Creates builtin types in the environment.
    pub fn create_builtins(&mut self) {
        // Primitive types
        let int_type = Rc::new(Type::primitive(PrimitiveTypeKind::Int));
        let float_type = Rc::new(Type::primitive(PrimitiveTypeKind::Float));
        let bool_type = Rc::new(Type::primitive(PrimitiveTypeKind::Bool));
        let str_type = Rc::new(Type::primitive(PrimitiveTypeKind::Str));
        let bytes_type = Rc::new(Type::primitive(PrimitiveTypeKind::Bytes));
        let none_type = Rc::new(Type::None);
        let any_type = Rc::new(Type::Any);

        // Add to environment
        self.add_type_def("int".to_string(), int_type);
        self.add_type_def("float".to_string(), float_type);
        self.add_type_def("bool".to_string(), bool_type);
        self.add_type_def("str".to_string(), str_type);
        self.add_type_def("bytes".to_string(), bytes_type);
        self.add_type_def("None".to_string(), none_type);
        self.add_type_def("Any".to_string(), any_type);

        // Create container types
        let list_class = ClassType::new("list".to_string(), None);
        let dict_class = ClassType::new("dict".to_string(), None);
        let set_class = ClassType::new("set".to_string(), None);

        // Add to environment
        self.add_type_def("list".to_string(), Rc::new(Type::Class(list_class)));
        self.add_type_def("dict".to_string(), Rc::new(Type::Class(dict_class)));
        self.add_type_def("set".to_string(), Rc::new(Type::Class(set_class)));
    }
}

/// Type compatibility result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCompatibility {
    /// Types are identical.
    Identical,
    /// Types are compatible (subtype relationship).
    Compatible,
    /// Types are incompatible.
    Incompatible,
}

impl TypeCompatibility {
    /// Returns whether the compatibility is identical or compatible.
    pub fn is_compatible(self) -> bool {
        matches!(self, TypeCompatibility::Identical | TypeCompatibility::Compatible)
    }

    /// Returns whether the compatibility is identical.
    pub fn is_identical(self) -> bool {
        matches!(self, TypeCompatibility::Identical)
    }
}

/// Checks if a type is compatible with another type.
pub fn check_type_compatibility(left: &Type, right: &Type) -> TypeCompatibility {
    // Same type means identical
    if left == right {
        return TypeCompatibility::Identical;
    }

    match (left, right) {
        // Any is compatible with everything
        (Type::Any, _) | (_, Type::Any) => TypeCompatibility::Compatible,

        // None is compatible with optional types
        (Type::None, Type::Union(union)) => {
            for ty in &union.types {
                if matches!(ty.as_ref(), Type::None) {
                    return TypeCompatibility::Compatible;
                }
            }
            TypeCompatibility::Incompatible
        }

        // None is not compatible with other types
        (Type::None, _) => TypeCompatibility::Incompatible,

        // Never is compatible with everything (as a subtype)
        (Type::Never, _) => TypeCompatibility::Compatible,

        // Primitive types
        (Type::Primitive(left_prim), Type::Primitive(right_prim)) => {
            if left_prim.kind == right_prim.kind {
                TypeCompatibility::Identical
            } else {
                // Special case for numeric types
                match (left_prim.kind, right_prim.kind) {
                    (PrimitiveTypeKind::Int, PrimitiveTypeKind::Float) => {
                        TypeCompatibility::Compatible
                    }
                    _ => TypeCompatibility::Incompatible,
                }
            }
        }

        // Class types
        (Type::Class(left_class), Type::Class(right_class)) => {
            if left_class.name == right_class.name {
                TypeCompatibility::Identical
            } else {
                // Check inheritance
                for base in &left_class.bases {
                    if check_type_compatibility(base, right).is_compatible() {
                        return TypeCompatibility::Compatible;
                    }
                }
                TypeCompatibility::Incompatible
            }
        }

        // Function types
        (Type::Function(left_func), Type::Function(right_func)) => {
            // Check return types
            let return_compat =
                check_type_compatibility(&left_func.return_type, &right_func.return_type);
            if !return_compat.is_compatible() {
                return TypeCompatibility::Incompatible;
            }

            // Check parameters
            if left_func.parameters.len() != right_func.parameters.len() {
                return TypeCompatibility::Incompatible;
            }

            let mut all_identical = return_compat.is_identical();

            for (left_param, right_param) in left_func.parameters.iter().zip(&right_func.parameters)
            {
                let param_compat = check_type_compatibility(&left_param.ty, &right_param.ty);
                if !param_compat.is_compatible() {
                    return TypeCompatibility::Incompatible;
                }
                all_identical = all_identical && param_compat.is_identical();
            }

            if all_identical { TypeCompatibility::Identical } else { TypeCompatibility::Compatible }
        }

        // Union types
        (Type::Union(left_union), Type::Union(right_union)) => {
            let mut all_compatible = true;
            let mut all_covered = true;

            // Check if all types in left_union are compatible with some type in right_union
            for left_ty in &left_union.types {
                let mut type_covered = false;
                for right_ty in &right_union.types {
                    if check_type_compatibility(left_ty, right_ty).is_compatible() {
                        type_covered = true;
                        break;
                    }
                }
                all_covered = all_covered && type_covered;
            }

            // Check if all types in right_union are compatible with some type in left_union
            for right_ty in &right_union.types {
                let mut type_covered = false;
                for left_ty in &left_union.types {
                    if check_type_compatibility(left_ty, right_ty).is_compatible() {
                        type_covered = true;
                        break;
                    }
                }
                all_compatible = all_compatible && type_covered;
            }

            if all_compatible && all_covered {
                TypeCompatibility::Identical
            } else if all_covered {
                TypeCompatibility::Compatible
            } else {
                TypeCompatibility::Incompatible
            }
        }

        // Union with non-union
        (Type::Union(union), other) => {
            for ty in &union.types {
                if check_type_compatibility(ty, other).is_compatible() {
                    return TypeCompatibility::Compatible;
                }
            }
            TypeCompatibility::Incompatible
        }
        (other, Type::Union(union)) => {
            for ty in &union.types {
                if check_type_compatibility(other, ty).is_compatible() {
                    return TypeCompatibility::Compatible;
                }
            }
            TypeCompatibility::Incompatible
        }

        // Tuple types
        (Type::Tuple(left_tuple), Type::Tuple(right_tuple)) => {
            if left_tuple.element_types.len() != right_tuple.element_types.len() {
                return TypeCompatibility::Incompatible;
            }

            let mut all_identical = true;

            for (left_ty, right_ty) in
                left_tuple.element_types.iter().zip(&right_tuple.element_types)
            {
                let elem_compat = check_type_compatibility(left_ty, right_ty);
                if !elem_compat.is_compatible() {
                    return TypeCompatibility::Incompatible;
                }
                all_identical = all_identical && elem_compat.is_identical();
            }

            if all_identical { TypeCompatibility::Identical } else { TypeCompatibility::Compatible }
        }

        // List types
        (Type::List(left_list), Type::List(right_list)) => {
            let elem_compat =
                check_type_compatibility(&left_list.element_type, &right_list.element_type);
            if elem_compat.is_identical() {
                TypeCompatibility::Identical
            } else if elem_compat.is_compatible() {
                TypeCompatibility::Compatible
            } else {
                TypeCompatibility::Incompatible
            }
        }

        // TypeVar
        (Type::TypeVar(var), other) => {
            if var.constraints.is_empty() {
                TypeCompatibility::Compatible
            } else {
                for constraint in &var.constraints {
                    if check_type_compatibility(constraint, other).is_compatible() {
                        return TypeCompatibility::Compatible;
                    }
                }
                TypeCompatibility::Incompatible
            }
        }
        (other, Type::TypeVar(var)) => {
            if var.constraints.is_empty() {
                TypeCompatibility::Compatible
            } else {
                for constraint in &var.constraints {
                    if check_type_compatibility(other, constraint).is_compatible() {
                        return TypeCompatibility::Compatible;
                    }
                }
                TypeCompatibility::Incompatible
            }
        }

        // Generic instance
        (Type::GenericInstance(left_gen), Type::GenericInstance(right_gen)) => {
            // Check base types
            let base_compat = check_type_compatibility(&left_gen.base, &right_gen.base);
            if !base_compat.is_compatible() {
                return TypeCompatibility::Incompatible;
            }

            // Check type arguments
            if left_gen.type_args.len() != right_gen.type_args.len() {
                return TypeCompatibility::Incompatible;
            }

            let mut all_identical = base_compat.is_identical();

            for (left_arg, right_arg) in left_gen.type_args.iter().zip(&right_gen.type_args) {
                let arg_compat = check_type_compatibility(left_arg, right_arg);
                if !arg_compat.is_compatible() {
                    return TypeCompatibility::Incompatible;
                }
                all_identical = all_identical && arg_compat.is_identical();
            }

            if all_identical { TypeCompatibility::Identical } else { TypeCompatibility::Compatible }
        }

        // Default: incompatible
        _ => TypeCompatibility::Incompatible,
    }
}
