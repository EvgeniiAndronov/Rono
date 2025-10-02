use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ChifType {
    Int,
    Float,
    Str,
    Bool,
    Nil,
    Array(Box<ChifType>, Vec<usize>), // type, dimensions
    List(Box<ChifType>, Vec<usize>),  // type, dimensions
    Map(Box<ChifType>, Box<ChifType>), // key_type, value_type
    Struct(String),                   // struct name
    Pointer(Box<ChifType>),
}

#[derive(Debug, Clone)]
pub enum ChifValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Nil,
    Array(Vec<ChifValue>),
    List(Vec<ChifValue>),
    Map(HashMap<String, ChifValue>),
    Struct(String, HashMap<String, ChifValue>),
    Pointer(Box<ChifValue>),
    Reference(String), // Reference to a variable name
}

impl fmt::Display for ChifType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChifType::Int => write!(f, "int"),
            ChifType::Float => write!(f, "float"),
            ChifType::Str => write!(f, "str"),
            ChifType::Bool => write!(f, "bool"),
            ChifType::Nil => write!(f, "nil"),
            ChifType::Array(inner, dims) => {
                write!(f, "array[{}]", inner)?;
                for dim in dims {
                    write!(f, "[{}]", dim)?;
                }
                Ok(())
            }
            ChifType::List(inner, dims) => {
                write!(f, "list[{}]", inner)?;
                for _ in dims {
                    write!(f, "[]")?;
                }
                Ok(())
            }
            ChifType::Map(key, value) => write!(f, "map[{}:{}]", key, value),
            ChifType::Struct(name) => write!(f, "{}", name),
            ChifType::Pointer(inner) => write!(f, "pointer[{}]", inner),
        }
    }
}

impl fmt::Display for ChifValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChifValue::Int(i) => write!(f, "{}", i),
            ChifValue::Float(fl) => write!(f, "{}", fl),
            ChifValue::Str(s) => write!(f, "{}", s),
            ChifValue::Bool(b) => write!(f, "{}", b),
            ChifValue::Nil => write!(f, "nil"),
            ChifValue::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            ChifValue::List(list) => {
                write!(f, "[")?;
                for (i, val) in list.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            ChifValue::Map(map) => {
                write!(f, "{{")?;
                for (i, (key, val)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{}\": {}", key, val)?;
                }
                write!(f, "}}")
            }
            ChifValue::Struct(name, fields) => {
                write!(f, "{} {{ ", name)?;
                for (i, (key, val)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", key, val)?;
                }
                write!(f, " }}")
            }
            ChifValue::Pointer(val) => write!(f, "&{}", val),
            ChifValue::Reference(var_name) => write!(f, "&{}", var_name),
        }
    }
}

impl ChifValue {
    pub fn get_type(&self) -> ChifType {
        match self {
            ChifValue::Int(_) => ChifType::Int,
            ChifValue::Float(_) => ChifType::Float,
            ChifValue::Str(_) => ChifType::Str,
            ChifValue::Bool(_) => ChifType::Bool,
            ChifValue::Nil => ChifType::Nil,
            ChifValue::Array(arr) => {
                if let Some(first) = arr.first() {
                    ChifType::Array(Box::new(first.get_type()), vec![arr.len()])
                } else {
                    ChifType::Array(Box::new(ChifType::Nil), vec![0])
                }
            }
            ChifValue::List(list) => {
                if let Some(first) = list.first() {
                    ChifType::List(Box::new(first.get_type()), vec![])
                } else {
                    ChifType::List(Box::new(ChifType::Nil), vec![])
                }
            }
            ChifValue::Map(map) => {
                if let Some((_, val)) = map.iter().next() {
                    ChifType::Map(Box::new(ChifType::Str), Box::new(val.get_type()))
                } else {
                    ChifType::Map(Box::new(ChifType::Str), Box::new(ChifType::Nil))
                }
            }
            ChifValue::Struct(name, _) => ChifType::Struct(name.clone()),
            ChifValue::Pointer(val) => ChifType::Pointer(Box::new(val.get_type())),
            ChifValue::Reference(_) => ChifType::Pointer(Box::new(ChifType::Nil)),
        }
    }
}