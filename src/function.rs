use std::{fmt, mem::transmute, str::FromStr};

use nanbox::{NanBox, NanBoxable};

use crate::{lex::Span, ops::Primitive, Ident};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FunctionId {
    Named(Ident),
    Anonymous(Span),
    FormatString(Span),
    Primitive(Primitive),
}

impl From<Ident> for FunctionId {
    fn from(name: Ident) -> Self {
        Self::Named(name)
    }
}

impl From<Primitive> for FunctionId {
    fn from(op: Primitive) -> Self {
        Self::Primitive(op)
    }
}

impl fmt::Display for FunctionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionId::Named(name) => write!(f, "`{name}`"),
            FunctionId::Anonymous(span) => write!(f, "fn from {span}"),
            FunctionId::FormatString(span) => write!(f, "format string from {span}"),
            FunctionId::Primitive(id) => write!(f, "{id}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Function {
    Code(u32),
    Primitive(Primitive),
    Selector(Selector),
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::Code(start) => write!(f, "({start})"),
            Function::Primitive(prim) => write!(f, "{prim}"),
            Function::Selector(sel) => write!(f, "{sel}"),
        }
    }
}

impl NanBoxable for Function {
    unsafe fn from_nan_box(n: NanBox) -> Self {
        let [a, b, c, d, e, f]: [u8; 6] = NanBoxable::from_nan_box(n);
        let start = u32::from_le_bytes([b, c, d, e]);
        match a {
            0 => Function::Code(start),
            1 => Function::Primitive(transmute([b, c])),
            2 => Function::Selector(Selector([b, c, d, e, f])),
            _ => unreachable!(),
        }
    }
    fn into_nan_box(self) -> NanBox {
        match self {
            Function::Code(start) => {
                let [b, c, d, e] = start.to_le_bytes();
                NanBoxable::into_nan_box([0, b, c, d, e])
            }
            Function::Primitive(prim) => {
                let [b, c]: [u8; 2] = unsafe { transmute(prim) };
                NanBoxable::into_nan_box([1, b, c, 0, 0])
            }
            Function::Selector(sel) => {
                let [b, c, d, e, f] = sel.0;
                NanBoxable::into_nan_box([2, b, c, d, e, f])
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Selector([u8; 5]);

impl Selector {
    pub fn min_inputs(&self) -> u8 {
        self.0.iter().max().copied().unwrap()
    }
    pub fn outputs(&self) -> u8 {
        self.0.iter().position(|&i| i == 0).unwrap_or(5) as u8
    }
    pub fn get(&self, index: u8) -> u8 {
        self.0[index as usize]
    }
    pub fn output_indices(&self) -> impl Iterator<Item = u8> + '_ {
        self.0
            .iter()
            .copied()
            .take_while(|&i| i != 0)
            .map(|i| i - 1)
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in self.0 {
            if i == 0 {
                break;
            }
            write!(f, "{}", (b'a' + i - 1) as char)?;
        }
        Ok(())
    }
}

impl FromStr for Selector {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || s.len() > 5 || s.chars().any(|c| !('a'..='e').contains(&c)) {
            return Err(());
        }
        let mut inner = [0; 5];
        for (i, c) in s.chars().enumerate() {
            inner[i] = c as u8 - b'a' + 1;
        }
        Ok(Self(inner))
    }
}
