use std::fmt::{self, Debug, Display};

// TODO maybe abstract programs over a universe i.e. only over types?

#[derive(Default, Debug, Clone)]
pub struct Program {
    // rules: Vec<Rule>,
    // questions: Vec<Question>,
    variables: Vec<(Ident, usize)>,
    // predicates: Vec<Ident>,
    constraints: Vec<(ConstraintKind, Ref, Ref)>,

    type_values: Vec<Ty>,
}

pub trait Visitor {
    fn walk_ref(&mut self, r: &Ref, env: &Program);
    fn walk_ty(&mut self, ty: &Ty, env: &Program);
}

pub struct TyUnifier {
    ty: Ty,
    /// Visited Identifiers
    visited: Vec<usize>,
}

/// Unify a TyKind while walking down an ident
impl Visitor for TyUnifier {
    fn walk_ref(&mut self, r: &Ref, env: &Program) {
        match r {
            Ref::Ty(box t) => self.walk_ty(t, env),
            Ref::Ident(idx) => {
                if self.visited.contains(idx) {
                    return;
                }
                self.visited.push(*idx);
                for item in env.unified_items(*idx) {
                    self.walk_ref(item, env)
                }
                self.walk_ty(&env.type_values[env.variables[*idx].1], env);
            }
        }
    }
    fn walk_ty(&mut self, ty: &Ty, env: &Program) {
        match &ty.kind {
            TyKind::Param(p) => self.walk_ref(&Ref::Ident(p.unwrap()), env),
            _ => self.ty = self.ty.unify(ty, env),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ref {
    Ident(usize),
    Ty(Box<Ty>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LowerableIdent {
    Idx(usize),
    Ident(Ident),
}

impl LowerableIdent {
    fn lower(self, env: &Program) -> Self {
        // TODO maybe just return instead
        let LowerableIdent::Ident(id) = self else {
            panic!("already lowered {:?}", self);
        };
        let Some(idx) = env.ref_for(id.clone()) else {
            panic!("Type parameter {} not defined", id.0);
        };
        LowerableIdent::Idx(idx)
    }
    fn unwrap(&self) -> usize {
        let Self::Idx(i) = self else {
            panic!("Unexpected failed unwrap {:?}", self);
        };
        *i
    }
}

// TODO maybe there needs to be some way to make this modifiable
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TyKind {
    Dynamic,

    Number,
    Text,
    Bool,

    /// An unbound type which can be substituted for another type.
    Param(LowerableIdent),

    Tuple(Box<Ty>, Box<Ty>),

    Enum(Box<Ty>, Box<Ty>),

    /*
    Func(Ref, Ref),
    */
    // TODO maybe include reason here?
    /// Uninhabited type, for example when unifying text and an int.
    Never,
}

impl TyKind {
    fn lower(self, env: &Program) -> Self {
        use TyKind::*;
        match self {
            Param(li) => Param(li.lower(env)),
            Tuple(box l, box r) => Tuple(box l.lower(env), box r.lower(env)),
            Enum(box l, box r) => Enum(box l.lower(env), box r.lower(env)),

            Dynamic | Never | Number | Bool | Text => self,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Constraint {
    None,

    /// Equality of this type to a specific value
    Eq(Literal),

    /// Inequality of this type to another value
    Neq(Literal),
    // And(Box<Constraint>, Box<Constraint>),
}

impl Constraint {
    fn unify(&self, other: &Self) -> Result<Self, ()> {
        use Constraint::*;
        let result = match (self, other) {
            (None, o) | (o, None) => o.clone(),
            (Eq(l), Eq(r)) if l == r => Eq(l.clone()),

            (Neq(l), Neq(r)) => {
                if l == r {
                    Neq(l.clone())
                } else {
                    todo!()
                }
            }

            (Eq(e), Neq(n)) | (Neq(n), Eq(e)) => {
                if e == n {
                    return Err(());
                } else {
                    Eq(e.clone())
                }
            }

            _ => return Err(()),
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ty {
    kind: TyKind,
    constraints: Constraint,
}

impl Ty {
    /// Represents any valid value
    pub fn all() -> Self {
        Self {
            kind: TyKind::Dynamic,
            constraints: Constraint::None,
        }
    }
    pub fn none() -> Self {
        Self {
            kind: TyKind::Never,
            constraints: Constraint::None,
        }
    }
    pub fn from_literal(lit: Literal) -> Self {
        Self {
            kind: lit.kind(),
            constraints: Constraint::Eq(lit),
        }
    }

    pub fn lower(self, env: &Program) -> Self {
        let Self { kind, constraints } = self;
        Self {
            kind: kind.lower(env),
            constraints,
        }
    }

    // TODO need to include env here
    /// Unify this type with another type
    fn unify(&self, other: &Self, env: &Program) -> Self {
        use TyKind::*;
        // Order is important here, need to handle params first
        let kind = match (&self.kind, &other.kind) {
            (Dynamic, other) | (other, Dynamic) => other.clone(),
            (Param(a), Param(b)) if a == b => Param(a.clone()),
            (Param(p), _) => {
                let p = p.unwrap();
                let mut param_t = TyUnifier {
                    ty: env.type_values[env.variables[p].1].clone(),
                    visited: vec![],
                };
                param_t.walk_ref(&Ref::Ident(p), env);
                return param_t.ty.unify(other, env);
            }
            (_, Param(p)) => {
                let p = p.unwrap();
                let mut param_t = TyUnifier {
                    ty: env.type_values[env.variables[p].1].clone(),
                    visited: vec![],
                };
                param_t.walk_ref(&Ref::Ident(p), env);
                return self.unify(&param_t.ty, env);
            }

            (Enum(box a, box b), _) => {
                let a = a.unify(other, env);
                let b = b.unify(other, env);
                match (&a.kind, &b.kind) {
                    (v, Never) => return a,
                    (Never, v) => return b,
                    (x, y) => {
                        if a == b {
                            return a;
                        } else {
                            Enum(box a, box b)
                        }
                    }
                }
            }
            (o, Enum(_, _)) => return other.unify(self, env),

            (Tuple(box l, box r), Tuple(box a, box b)) => {
                TyKind::Tuple(box l.unify(a, env), box r.unify(b, env))
            }
            (Tuple(_, _), _) | (_, Tuple(_, _)) => return Self::none(),

            // If parameters are equal we do not need to do anythign more.
            (Dynamic | Never | Text | Bool | Number, Dynamic | Never | Text | Bool | Number) => {
                if self.kind != other.kind {
                    return Self::none();
                } else {
                    self.kind.clone()
                }
            }
        };
        let Ok(constraints) = self.constraints.unify(&other.constraints) else {
            return Self::none();
        };
        Self { kind, constraints }
    }
}

impl From<TyKind> for Ty {
    fn from(kind: TyKind) -> Self {
        Ty {
            kind,
            constraints: Constraint::None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LiteralOrIdent {
    Ident(Ident),
    Literal(Literal),
}

impl Program {
    // TODO maybe it makes sense to split this into separate statements
    pub fn lower(&mut self, s: Statement) -> Result<(), String> {
        match s {
            Statement::Variable(id, ty) => {
                let t_idx = self.type_values.len();
                self.type_values.push(ty.lower(self));
                self.variables.push((id, t_idx));
            }
            Statement::Constrain(c, l, r) => {
                use LiteralOrIdent::Literal as Lit;
                match (c, &l, &r) {
                    (ConstraintKind::Eq, Lit(l), Lit(r)) => {
                        if l == r {
                            return Ok(());
                        } else {
                            return Err(String::from("Cannot unify unequal constants"));
                        }
                    }
                    (ConstraintKind::Neq, Lit(l), Lit(r)) => {
                        if l != r {
                            return Ok(());
                        } else {
                            return Err(String::from("Cannot unify unequal constants"));
                        }
                    }
                    _ => {}
                }
                let l = self.reify(l)?;
                let r = self.reify(r)?;
                self.constraints.push((c, l, r));
            }
            Statement::Possible(id) => {
                let possible_tys = self.satisfied_values(id.clone());
                println!("{} : {}", id.0, TyAndProgram(possible_tys, self));
            }
        }
        Ok(())
    }
    // Converts a constant or ident into a reference of a literal.
    fn reify(&mut self, v: LiteralOrIdent) -> Result<Ref, String> {
        match v {
            LiteralOrIdent::Ident(v_id) => Ok(Ref::Ident(
                self.variables
                    .iter()
                    .position(|(id, _)| id == &v_id)
                    .ok_or(format!("Unbound variable {:?}", v_id))?,
            )),
            LiteralOrIdent::Literal(l_val) => Ok(Ref::Ty(box Ty::from_literal(l_val))),
        }
    }

    // TODO maybe make a way to shrink it to lowest possible index?

    pub fn satisfied_values(&self, of: Ident) -> Ty {
        let Some(idx) =  self.ref_for(of) else { return Ty::all(); };

        let ty = self.type_values[self.variables[idx].1].clone();
        let mut valid = TyUnifier {
            ty,
            visited: vec![],
        };
        valid.walk_ref(&Ref::Ident(idx), &self);
        valid.ty
    }
    fn ref_for(&self, i: Ident) -> Option<usize> {
        self.variables.iter().position(|(id, _)| id == &i)
    }
    fn unified_items(&self, i: usize) -> impl Iterator<Item = &'_ Ref> + '_ {
        let i = Ref::Ident(i);
        self.constraints.iter().filter_map(move |(c, l, r)| {
            if *l == i {
                Some(r)
            } else if *r == i {
                Some(l)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident(pub String);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConstraintKind {
    Eq,
    Neq,
}

pub enum Statement {
    Variable(Ident, Ty),
    Constrain(ConstraintKind, LiteralOrIdent, LiteralOrIdent),
    Possible(Ident),
}

macro_rules! impl_constant {
  ($($name: ident = $t: ty : $ty_kind: expr $(,)?)+) => {
    /// A raw literal expressible in the AST.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Literal { $( $name($t), )+ }

    impl Literal {
      const fn kind(&self) -> TyKind {
        match self {
          $( Self::$name(..) => $ty_kind, )+
        }
      }
    }
  }
}
impl_constant!(
    I32 = i32: TyKind::Number,
    Str = String: TyKind::Text,
    Bool = bool: TyKind::Bool,
);

impl Display for TyAndProgram<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&TyKindAndProgram(self.0.kind.clone(), self.1), f)?;
        if matches!(self.0.constraints, Constraint::None) {
            return Ok(());
        }
        write!(f, " where ");
        Display::fmt(&self.0.constraints, f)
    }
}

// TODO in theory could optimize this to take references to types instead.
pub struct TyAndProgram<'a>(Ty, &'a Program);
pub struct TyKindAndProgram<'a>(TyKind, &'a Program);

impl Display for TyKindAndProgram<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TyKind::*;
        match &self.0 {
            Number => write!(f, "Number"),
            Text => write!(f, "Text"),
            Bool => write!(f, "Bool"),

            Never => write!(f, "!"),
            Dynamic => write!(f, "*"),
            Param(LowerableIdent::Idx(i)) => {
                let (ident, param_type_idx) = &self.1.variables[*i];
                write!(
                    f,
                    "<{}: {}>",
                    ident.0,
                    TyAndProgram(self.1.type_values[*param_type_idx].clone(), &self.1)
                )
            }
            Tuple(box a, box b) => {
                write!(
                    f,
                    "({}, {})",
                    TyAndProgram(a.clone(), &self.1),
                    TyAndProgram(b.clone(), &self.1),
                )
            }
            Enum(box a, box b) => write!(
                f,
                "{} | {}",
                TyAndProgram(a.clone(), &self.1),
                TyAndProgram(b.clone(), &self.1),
            ),
            v => panic!("Unimplemented {:?}", v),
        }
    }
}

impl Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Constraint::*;
        match self {
            None => write!(f, "true"),
            Eq(l) => write!(f, "= {:?}", l),
            Neq(v) => write!(f, "!= {:?}", v),
        }
    }
}
