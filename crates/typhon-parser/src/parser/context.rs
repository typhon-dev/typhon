//! Parser context management for parsing different constructs
//!
//! This module provides context tracking structures used during parsing
//! to track things like current scope, parent nodes, and language constructs.

use typhon_ast::nodes::NodeID;

/// The type of construct currently being parsed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextType {
    /// Assignment statement context
    Assignment,
    /// Class definition context
    Class,
    /// Conditional context (if, elif, else)
    Conditional,
    /// Exception handling context (try, except, finally)
    Exception,
    /// Expression context
    Expression,
    /// Function definition context
    Function,
    /// Default context
    Global,
    /// Import statement context
    Import,
    /// Loop context (while, for)
    Loop,
}

/// String literal type being parsed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringType {
    /// Regular string literal
    Regular,
    /// Format string literal (f-string)
    Format,
    /// Template string literal (t-string)
    Template,
}

/// Function or method modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FunctionModifiers {
    /// Whether the function is async
    pub is_async: bool,
    /// Whether the function has decorators
    pub has_decorator: bool,
}

/// Identifier visibility type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierType {
    /// Regular identifier (no special prefix/style)
    Regular,
    /// Private identifier (single leading underscore)
    Private,
    /// Mangled identifier (double leading underscores)
    Mangled,
    /// Constant identifier (all uppercase)
    Constant,
}

/// Context flags to track specific parsing states
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextFlags {
    /// Function-related modifiers
    pub fn_modifiers: FunctionModifiers,
    /// Type of string being parsed, if in a string context
    pub string_type: Option<StringType>,
    /// Type of identifier being parsed, if in an identifier context
    pub id_type: Option<IdentifierType>,
    /// Whether we're parsing a type annotation
    pub in_type_annotation: bool,
    /// Whether we're parsing a default argument
    pub in_default_arg: bool,
}

/// A parsing context tracks state for a specific construct being parsed
#[derive(Debug, Clone, Copy)]
pub struct Context {
    /// The type of construct being parsed
    pub context_type: ContextType,
    /// Optional parent node ID
    pub parent: Option<NodeID>,
    /// Additional flags for specific parsing conditions
    pub flags: ContextFlags,
    /// Starting indentation level
    pub indent_level: usize,
}

impl Context {
    /// Create a new context
    #[must_use]
    pub fn new(context_type: ContextType, parent: Option<NodeID>, indent_level: usize) -> Self {
        Self { context_type, parent, flags: ContextFlags::default(), indent_level }
    }

    /// Create a new context with the given flags
    #[must_use]
    pub const fn with_flags(
        context_type: ContextType,
        parent: Option<NodeID>,
        indent_level: usize,
        flags: ContextFlags,
    ) -> Self {
        Self { context_type, parent, flags, indent_level }
    }

    /// Check if this is a function context
    #[inline]
    #[must_use]
    pub fn is_function(&self) -> bool { self.context_type == ContextType::Function }

    /// Check if this is a class context
    #[inline]
    #[must_use]
    pub fn is_class(&self) -> bool { self.context_type == ContextType::Class }

    /// Check if this is a private context
    #[inline]
    #[must_use]
    pub const fn is_private(&self) -> bool {
        matches!(self.flags.id_type, Some(IdentifierType::Private))
    }

    /// Check if this is a template string context
    #[inline]
    #[must_use]
    pub const fn is_template_string(&self) -> bool {
        matches!(self.flags.string_type, Some(StringType::Template))
    }

    /// Check if this is a format string context
    #[inline]
    #[must_use]
    pub const fn is_format_string(&self) -> bool {
        matches!(self.flags.string_type, Some(StringType::Format))
    }
}

/// A context stack manages nested parsing contexts
#[derive(Default, Debug, Clone)]
pub struct ContextStack {
    /// Stack of active contexts
    stack: Vec<Context>,
}

impl ContextStack {
    /// Create a new empty context stack
    #[must_use]
    pub fn new() -> Self {
        let stack = vec![Context::new(ContextType::Global, None, 0)];

        Self { stack }
    }

    /// Push a new context onto the stack
    pub fn push(&mut self, context: Context) { self.stack.push(context); }

    /// Pop the current context off the stack
    pub fn pop(&mut self) -> Option<Context> {
        // Always keep at least the global context
        if self.stack.len() <= 1 {
            return None;
        }
        self.stack.pop()
    }

    /// Get the current context
    ///
    /// ## Panics
    ///
    /// TODO: add context
    #[must_use]
    pub fn current(&self) -> &Context {
        self.stack.last().expect("Context stack should never be empty")
    }

    /// Get a mutable reference to the current context
    ///
    /// ## Panics
    ///
    /// TODO: add context
    #[must_use]
    pub fn current_mut(&mut self) -> &mut Context {
        self.stack.last_mut().expect("Context stack should never be empty")
    }

    /// Check if the current context is of the given type
    #[inline]
    #[must_use]
    pub fn in_context(&self, context_type: ContextType) -> bool {
        self.current().context_type == context_type
    }

    /// Check if we are in any of the given contexts
    #[inline]
    #[must_use]
    pub fn in_any_context(&self, context_types: &[ContextType]) -> bool {
        context_types.contains(&self.current().context_type)
    }

    /// Check if we're currently in a loop context
    #[inline]
    #[must_use]
    pub fn in_loop(&self) -> bool { self.in_context(ContextType::Loop) }

    /// Check if we're currently in a function context
    #[inline]
    #[must_use]
    pub fn in_function(&self) -> bool { self.in_context(ContextType::Function) }

    /// Check if we're currently in a class context
    #[inline]
    #[must_use]
    pub fn in_class(&self) -> bool { self.in_context(ContextType::Class) }

    /// Check if we're currently in a template string context
    #[inline]
    #[must_use]
    pub fn in_template_string(&self) -> bool { self.current().is_template_string() }

    /// Check if we're currently in a format string context
    #[inline]
    #[must_use]
    pub fn in_format_string(&self) -> bool { self.current().is_format_string() }

    /// Check if we're currently in a private context
    #[inline]
    #[must_use]
    pub fn in_private_context(&self) -> bool { self.current().is_private() }

    /// Find the nearest parent context of the given type
    #[must_use]
    pub fn find_parent_context(&self, context_type: ContextType) -> Option<&Context> {
        self.stack.iter().rev().find(|ctx| ctx.context_type == context_type)
    }

    /// Get the current indentation level
    #[inline]
    #[must_use]
    pub fn current_indent_level(&self) -> usize { self.current().indent_level }
}
