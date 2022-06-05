use std::fmt::Debug;

// TODO maybe abstract programs over a universe i.e. only over types?

#[derive(Default, Debug, Clone)]
pub struct Program {
    // rules: Vec<Rule>,
    // questions: Vec<Question>,
    variables: Vec<Ident>,
    // predicates: Vec<Ident>,
    unions: Vec<(Ref, Ref)>,

    constants: Vec<Literal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ref {
    Ident(usize),
    Constant(usize),
}

#[derive(Debug, Clone)]
pub enum ConstantOrIdent {
    Ident(Ident),
    Constant(Literal),
}

impl Program {
    // TODO maybe it makes sense to split this into separate statements
    pub fn lower(&mut self, s: Statement) -> Result<(), String> {
        match s {
            Statement::Variable(id) => self.variables.push(id),
            Statement::Unify(l, r) => {
                let l = self.reify(l)?;
                let r = self.reify(r)?;
                self.unions.push((l, r));
            }
            Statement::Possible(id) => {
                println!("{:?}", self.satisfied_values(id));
            }
        }
        Ok(())
    }
    // Converts a constant or ident into a reference of a literal.
    fn reify(&mut self, v: ConstantOrIdent) -> Result<Ref, String> {
        match v {
            ConstantOrIdent::Ident(v_id) => Ok(Ref::Ident(
                self.variables
                    .iter()
                    .position(|id| id == &v_id)
                    .ok_or(format!("Unbound variable {:?}", v_id))?,
            )),
            ConstantOrIdent::Constant(l_val) => {
                let idx = if let Some(idx) = self.constants.iter().position(|val| val == &l_val) {
                    idx
                } else {
                    let idx = self.constants.len();
                    self.constants.push(l_val);
                    idx
                };

                Ok(Ref::Constant(idx))
            }
        }
    }

    // TODO maybe make a way to shrink it to lowest possible index?

    pub fn satisfied_values(
        &self,
        of: Ident,
    ) -> Possibilities<impl IntoIterator<Item = Literal> + Debug + '_> {
        let Some(idx) =  self.ref_for(of) else { return Possibilities::Unbound; };
        // have the index of the reified_variable
        let mut work = vec![idx];
        //let concrete = None;
        loop {
            let Some(v) = work.pop() else { break };
        }
        todo!();
        Possibilities::Values(vec![])
    }
    fn ref_for(&self, i: Ident) -> Option<usize> {
        self.variables.iter().position(|id| id == &i)
    }
    fn unified_vars(&self, l: usize, r: usize) -> bool {
        self.unions.contains(&(Ref::Ident(l), Ref::Ident(r)))
    }
}

#[derive(Debug, PartialEq)]
pub enum Possibilities<I> {
    Unbound,
    Values(I),
    Inconsistent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident(pub String);

pub enum Statement {
    Variable(Ident),
    Unify(ConstantOrIdent, ConstantOrIdent),
    Possible(Ident),
}

macro_rules! impl_constant {
  ($($name: ident = $t: ty $(,)?)+) => {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Literal {
      $(
        $name($t),
      )+
    }
  }
}
impl_constant!(I32 = i32,);
