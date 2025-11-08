// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-runtime/src/object.rs
// SPDX-FileType: SOURCE
// SPDX-License-Identifier: Apache-2.0
// -------------------------------------------------------------------------
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -------------------------------------------------------------------------
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
            ObjectType::Int => write!(f, "int"),
            ObjectType::Float => write!(f, "float"),
            ObjectType::Bool => write!(f, "bool"),
            ObjectType::String => write!(f, "str"),
            ObjectType::List => write!(f, "list"),
            ObjectType::Dict => write!(f, "dict"),
            ObjectType::Function => write!(f, "function"),
            ObjectType::Class => write!(f, "class"),
            ObjectType::Instance => write!(f, "instance"),
            ObjectType::None => write!(f, "NoneType"),
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
    pub fn get_type(&self) -> ObjectType {
        match self {
            Value::Int(_) => ObjectType::Int,
            Value::Float(_) => ObjectType::Float,
            Value::Bool(_) => ObjectType::Bool,
            Value::String(_) => ObjectType::String,
            Value::List(_) => ObjectType::List,
            Value::Dict(_) => ObjectType::Dict,
            Value::Function(_) => ObjectType::Function,
            Value::Class(_) => ObjectType::Class,
            Value::Instance(_) => ObjectType::Instance,
            Value::None => ObjectType::None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(fl) => write!(f, "{fl}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::String(s) => write!(f, "{s}"),
            Value::List(l) => {
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
            Value::Dict(d) => {
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
            Value::Function(func) => write!(f, "<function {}>", func.name),
            Value::Class(class) => write!(f, "<class {}>", class.name),
            Value::Instance(instance) => write!(f, "<{} instance>", instance.class.name),
            Value::None => write!(f, "None"),
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
    // More fields would be added in a real implementation
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
