// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-runtime/src/builtins.rs
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
//! Built-in functions and types for the Typhon language.

use crate::errors::RuntimeError;
use crate::object::Value;

/// Implementation of the built-in print function.
pub fn print(args: &[Value]) -> Result<Value, RuntimeError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{arg}");
    }
    println!();
    Ok(Value::None)
}

/// Implementation of the built-in len function.
pub fn len(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::generic(format!(
            "len() takes exactly 1 argument ({} given)",
            args.len()
        )));
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::List(l) => Ok(Value::Int(l.borrow().len() as i64)),
        Value::Dict(d) => Ok(Value::Int(d.borrow().len() as i64)),
        _ => Err(RuntimeError::type_error(
            "str, list, or dict",
            args[0].get_type().to_string(),
            String::from("Object has no len()"),
        )),
    }
}

/// Implementation of the built-in type function.
pub fn type_of(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::generic(format!(
            "type() takes exactly 1 argument ({} given)",
            args.len()
        )));
    }

    Ok(Value::String(args[0].get_type().to_string()))
}

/// Implementation of the built-in str function.
pub fn to_string(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::generic(format!(
            "str() takes exactly 1 argument ({} given)",
            args.len()
        )));
    }

    Ok(Value::String(args[0].to_string()))
}

/// Implementation of the built-in int function.
pub fn to_int(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::generic(format!(
            "int() takes exactly 1 argument ({} given)",
            args.len()
        )));
    }

    match &args[0] {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::String(s) => s
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| RuntimeError::value_error(format!("Invalid literal for int(): '{s}'"))),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        _ => Err(RuntimeError::type_error(
            "int, float, str, or bool",
            args[0].get_type().to_string(),
            String::from("Cannot convert to int"),
        )),
    }
}

/// Implementation of the built-in float function.
pub fn to_float(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::generic(format!(
            "float() takes exactly 1 argument ({} given)",
            args.len()
        )));
    }

    match &args[0] {
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::String(s) => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| RuntimeError::value_error(format!("Invalid literal for float(): '{s}'"))),
        Value::Bool(b) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        _ => Err(RuntimeError::type_error(
            "int, float, str, or bool",
            args[0].get_type().to_string(),
            String::from("Cannot convert to float"),
        )),
    }
}

/// Implementation of the built-in bool function.
pub fn to_bool(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::generic(format!(
            "bool() takes exactly 1 argument ({} given)",
            args.len()
        )));
    }

    let is_truthy = match &args[0] {
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::Bool(b) => *b,
        Value::String(s) => !s.is_empty(),
        Value::List(l) => !l.borrow().is_empty(),
        Value::Dict(d) => !d.borrow().is_empty(),
        Value::None => false,
        _ => true, // Other objects are truthy
    };

    Ok(Value::Bool(is_truthy))
}
