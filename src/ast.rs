use std::fmt::{self, Debug, Display};

// TODO maybe abstract programs over a universe i.e. only over types?

#[derive(Default, Debug, Clone)]
pub struct Program {
    // rules: Vec<Rule>,
    // questions: Vec<Question>,
    variables: Vec<(Ident, usize)>,
    // predicates: Vec<Ident>,
    unions: Vec<(Ref, Ref)>,

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
                self.ty = self.ty.unify(&env.type_values[env.variables[*idx].1], &env);
            }
        }
    }
    fn walk_ty(&mut self, ty: &Ty, env: &Program) {
        let kind = match (&self.ty.kind, &ty.kind) {
            (TyKind::Dynamic, v) | (v, TyKind::Dynamic) => v,
            (a, b) if a == b => a,
            // Need to handle case of unifying types with parameters later
            (TyKind::Param(a), TyKind::Param(b)) => {
                assert_ne!(a, b);
                todo!()
            }
            _ => todo!(),
        }
        .clone();
        let Ok(constraints) = self.ty.constraints.unify(&ty.constraints) else {
            self.ty = Ty::none();
            return
        };
        self.ty = Ty { kind, constraints };
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ref {
    Ident(usize),
    Ty(Box<Ty>),
}

// TODO maybe there needs to be some way to make this modifiable
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TyKind {
    Dynamic,

    Number,
    Text,
    Bool,

    /// An unbound type which can be substituted for another type.
    Param(usize),

    Tuple(Ref, Ref),

    /*
    Enum(Ref, Ref),
    Func(Ref, Ref),
    */
    // TODO maybe include reason here?
    /// Uninhabited type, for example when unifying text and an int.
    Never,
}

impl TyKind {
    fn subtype(&self, other: &Self, env: &Program) -> Self {
        use TyKind::*;
        match (self, other) {
            (Dynamic, v) | (v, Dynamic) => v.clone(),
            (a, b) if a == b => a.clone(),
            (Never, _) | (_, Never) => TyKind::Never,

            // Incomplete things
            (Param(a), Param(b)) => todo!(),
            (a, b) => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Constraint {
    None,

    /// Equality of this type to a specific value
    Eq(Literal),
}

impl Constraint {
    fn unify(&self, other: &Self) -> Result<Self, ()> {
        use Constraint::*;
        let result = match (self, other) {
            (None, o) | (o, None) => o.clone(),
            (Eq(l), Eq(r)) if l == r => Eq(l.clone()),
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
    fn all() -> Self {
        Self {
            kind: TyKind::Dynamic,
            constraints: Constraint::None,
        }
    }
    fn none() -> Self {
        Self {
            kind: TyKind::Never,
            constraints: Constraint::None,
        }
    }
    fn from_literal(lit: Literal) -> Self {
        let kind = lit.kind();
        Self {
            kind,
            constraints: Constraint::Eq(lit),
        }
    }

    // TODO need to include env here
    /// Unify this type with another type
    fn unify(&self, other: &Self, env: &Program) -> Self {
        let kind = match (&self.kind, &other.kind) {
            (TyKind::Dynamic, other) | (other, TyKind::Dynamic) => other,
            (a, b) if a != b => return Self::none(),
            (a, b) => a,
        }
        .clone();
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
            Statement::Variable(id, ty_kind) => {
                let t_idx = self.type_values.len();
                self.type_values.push(ty_kind.into());
                self.variables.push((id, t_idx));
            }
            Statement::Unify(l, r) => {
                match (&l, &r) {
                    (LiteralOrIdent::Literal(l), LiteralOrIdent::Literal(r)) if l != r => {
                        return Err(String::from("Cannot unify unequal constants"))
                    }
                    _ => {}
                }
                let l = self.reify(l)?;
                let r = self.reify(r)?;
                self.unions.push((l, r));
            }
            Statement::Possible(id) => {
                println!("{} : {}", id.0.clone(), self.satisfied_values(id));
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
        self.unions.iter().filter_map(move |(l, r)| {
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

pub enum Statement {
    Variable(Ident, TyKind),
    Unify(LiteralOrIdent, LiteralOrIdent),
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
impl_constant!(I32 = i32: TyKind::Number, Str = String: TyKind::Text);

impl Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.kind, f)?;
        if matches!(self.constraints, Constraint::None) {
            return Ok(());
        }
        write!(f, " where ");
        Display::fmt(&self.constraints, f)
    }
}

impl Display for TyKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TyKind::*;
        match self {
            Number => write!(f, "Number"),
            Text => write!(f, "Text"),
            Bool => write!(f, "Bool"),

            Never => write!(f, "!"),
            Dynamic => write!(f, "*"),
            _ => todo!(),
        }
    }
}

impl Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Constraint::*;
        match self {
            None => write!(f, "true"),
            Eq(l) => write!(f, "= {:?}", l),
        }
    }
}
