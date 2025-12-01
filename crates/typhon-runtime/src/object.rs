//! Object system for the Typhon runtime.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Type tags for Typhon objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    /// Integer value.
    Int,
    /// Float value.
    Float,
    /// Boolean value.
    Bool,
    /// String value.
    String,
    /// List value.
    List,
    /// Dictionary value.
    Dict,
    /// Function value.
    Function,
    /// Class value.
    Class,
    /// Instance of a class.
    Instance,
    /// None value.
    None,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::Bool => write!(f, "bool"),
            Self::String => write!(f, "str"),
            Self::List => write!(f, "list"),
            Self::Dict => write!(f, "dict"),
            Self::Function => write!(f, "function"),
            Self::Class => write!(f, "class"),
            Self::Instance => write!(f, "instance"),
            Self::None => write!(f, "NoneType"),
        }
    }
}

/// A value in the Typhon runtime.
#[derive(Debug, Clone)]
pub enum Value {
    /// Integer value.
    Int(i64),
    /// Float value.
    Float(f64),
    /// Boolean value.
    Bool(bool),
    /// String value.
    String(String),
    /// List value.
    List(Rc<RefCell<Vec<Value>>>),
    /// Dictionary value.
    Dict(Rc<RefCell<HashMap<String, Value>>>),
    /// Function value.
    Function(Rc<Function>),
    /// Class value.
    Class(Rc<Class>),
    /// Instance of a class.
    Instance(Rc<Instance>),
    /// None value.
    None,
}

impl Value {
    /// Get the type of the value.
    #[must_use]
    pub const fn get_type(&self) -> ObjectType {
        match self {
            Self::Int(_) => ObjectType::Int,
            Self::Float(_) => ObjectType::Float,
            Self::Bool(_) => ObjectType::Bool,
            Self::String(_) => ObjectType::String,
            Self::List(_) => ObjectType::List,
            Self::Dict(_) => ObjectType::Dict,
            Self::Function(_) => ObjectType::Function,
            Self::Class(_) => ObjectType::Class,
            Self::Instance(_) => ObjectType::Instance,
            Self::None => ObjectType::None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(i) => write!(f, "{i}"),
            Self::Float(fl) => write!(f, "{fl}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::String(s) => write!(f, "{s}"),
            Self::List(l) => {
                let l = l.borrow();
                write!(f, "[")?;
                let mut first = true;
                for item in l.iter() {
                    if first {
                        first = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Self::Dict(d) => {
                let d = d.borrow();
                write!(f, "{{")?;
                let mut first = true;
                for (k, v) in d.iter() {
                    if first {
                        first = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            Self::Function(func) => write!(f, "<function {}>", func.name),
            Self::Class(class) => write!(f, "<class {}>", class.name),
            Self::Instance(instance) => write!(f, "<{} instance>", instance.class.name),
            Self::None => write!(f, "None"),
        }
    }
}

/// A function in the Typhon runtime.
#[derive(Debug)]
pub struct Function {
    /// Name of the function.
    pub name: String,
    /// Code of the function.
    pub code: Vec<u8>,
}

/// A class in the Typhon runtime.
#[derive(Debug)]
pub struct Class {
    /// Name of the class.
    pub name: String,
    /// Methods of the class.
    pub methods: HashMap<String, Rc<Function>>,
    /// Base classes.
    pub bases: Vec<Rc<Class>>,
}

/// An instance of a class in the Typhon runtime.
#[derive(Debug)]
pub struct Instance {
    /// Class of the instance.
    pub class: Rc<Class>,
    /// Attributes of the instance.
    pub attributes: RefCell<HashMap<String, Value>>,
}
