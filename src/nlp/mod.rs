use crate::MathIR;
use crate::{Variable, Domain, AssumptionSet, Constant};

pub struct NaturalParser;

impl NaturalParser {
    pub fn parse(input: &str) -> Result<MathIR, String> {
        let lowered = input.trim().to_lowercase();

        if let Some(result) = try_phrase_map(&lowered) {
            return Ok(result);
        }

        Err(format!("Cannot parse natural language: {}", input))
    }
}

fn try_phrase_map(input: &str) -> Option<MathIR> {
    match input {
        "topology preservation" => Some(topology_preservation()),
        "reachability preservation" => Some(reachability_preservation()),
        "conduction soundness" => Some(conduction_soundness()),
        "exo execution condition" => Some(exo_execution_condition()),
        "symmetry preservation" => Some(symmetry_preservation()),
        "reflexivity" => Some(reflexivity()),
        "transitivity" => Some(transitivity()),
        "commutativity" => Some(commutativity()),
        "associativity" => Some(associativity()),
        "distributivity" => Some(distributivity()),
        "pythagorean identity" => Some(pythagorean_identity()),
        _ => None,
    }
}

fn forall(var: Variable, body: MathIR) -> MathIR {
    let domain = Box::new(var.domain.clone());
    MathIR::ForAll(var, domain, Box::new(body))
}

fn topology_preservation() -> MathIR {
    forall(var_t(), forall(var_e(), MathIR::Iff(
        Box::new(mem_e_t()),
        Box::new(mem_e_compile_t()),
    )))
}

fn reachability_preservation() -> MathIR {
    forall(var_t(), forall(var_a(), forall(var_b(), MathIR::Iff(
        Box::new(reachable_t_a_b()),
        Box::new(reachable_compile_t_a_b()),
    ))))
}

fn conduction_soundness() -> MathIR {
    forall(var_t(), forall(var_o(), MathIR::Iff(
        Box::new(execute_o_t()),
        Box::new(conducts_t_o()),
    )))
}

fn exo_execution_condition() -> MathIR {
    forall(var_t(), forall(var_o(), MathIR::Iff(
        Box::new(execute_o_t()),
        Box::new(MathIR::And(vec![
            reachable_t_o(),
            conducts_t_o(),
            sigma_t_eq_1(),
        ])),
    )))
}

fn symmetry_preservation() -> MathIR {
    forall(var_t(), MathIR::Iff(
        Box::new(MathIR::Fn { name: "Symmetric".into(), args: vec![MathIR::Var(Box::new(var_t()))] }),
        Box::new(MathIR::Fn { name: "Symmetric".into(), args: vec![MathIR::Fn { name: "Compile".into(), args: vec![MathIR::Var(Box::new(var_t()))] }] }),
    ))
}

fn reflexivity() -> MathIR {
    forall(var_t(), MathIR::Fn { name: "Reflexive".into(), args: vec![MathIR::Var(Box::new(var_t()))] })
}

fn transitivity() -> MathIR {
    forall(var_t(), MathIR::Fn { name: "Transitive".into(), args: vec![MathIR::Var(Box::new(var_t()))] })
}

fn commutativity() -> MathIR {
    forall(var_a(), forall(var_b(), MathIR::Iff(
        Box::new(MathIR::Fn { name: "R".into(), args: vec![MathIR::Var(Box::new(var_a())), MathIR::Var(Box::new(var_b()))] }),
        Box::new(MathIR::Fn { name: "R".into(), args: vec![MathIR::Var(Box::new(var_b())), MathIR::Var(Box::new(var_a()))] }),
    )))
}

fn associativity() -> MathIR {
    forall(var_a(), forall(var_b(), forall(var_c(), MathIR::Iff(
        Box::new(MathIR::Fn { name: "R".into(), args: vec![
            MathIR::Fn { name: "R".into(), args: vec![MathIR::Var(Box::new(var_a())), MathIR::Var(Box::new(var_b()))] },
            MathIR::Var(Box::new(var_c())),
        ] }),
        Box::new(MathIR::Fn { name: "R".into(), args: vec![
            MathIR::Var(Box::new(var_a())),
            MathIR::Fn { name: "R".into(), args: vec![MathIR::Var(Box::new(var_b())), MathIR::Var(Box::new(var_c()))] },
        ] }),
    ))))
}

fn distributivity() -> MathIR {
    forall(var_a(), forall(var_b(), forall(var_c(), MathIR::Iff(
        Box::new(MathIR::Fn { name: "R".into(), args: vec![
            MathIR::Var(Box::new(var_a())),
            MathIR::Fn { name: "S".into(), args: vec![MathIR::Var(Box::new(var_b())), MathIR::Var(Box::new(var_c()))] },
        ] }),
        Box::new(MathIR::Fn { name: "S".into(), args: vec![
            MathIR::Fn { name: "R".into(), args: vec![MathIR::Var(Box::new(var_a())), MathIR::Var(Box::new(var_b()))] },
            MathIR::Fn { name: "R".into(), args: vec![MathIR::Var(Box::new(var_a())), MathIR::Var(Box::new(var_c()))] },
        ] }),
    ))))
}

fn pythagorean_identity() -> MathIR {
    forall(var_x(), MathIR::Eq(
        Box::new(MathIR::Add(vec![
            MathIR::Pow(
                Box::new(MathIR::Fn { name: "sin".into(), args: vec![MathIR::Var(Box::new(var_x()))] }),
                Box::new(MathIR::Const(Constant::Int(2))),
            ),
            MathIR::Pow(
                Box::new(MathIR::Fn { name: "cos".into(), args: vec![MathIR::Var(Box::new(var_x()))] }),
                Box::new(MathIR::Const(Constant::Int(2))),
            ),
        ])),
        Box::new(MathIR::Const(Constant::Int(1))),
    ))
}

fn var_t() -> Variable { Variable { id: "T".into(), domain: Domain::UserDefined("Topology".into()), assumptions: AssumptionSet::default() } }
fn var_e() -> Variable { Variable { id: "e".into(), domain: Domain::UserDefined("Edge".into()), assumptions: AssumptionSet::default() } }
fn var_a() -> Variable { Variable { id: "a".into(), domain: Domain::UserDefined("Vertex".into()), assumptions: AssumptionSet::default() } }
fn var_b() -> Variable { Variable { id: "b".into(), domain: Domain::UserDefined("Vertex".into()), assumptions: AssumptionSet::default() } }
fn var_c() -> Variable { Variable { id: "c".into(), domain: Domain::UserDefined("Vertex".into()), assumptions: AssumptionSet::default() } }
fn var_o() -> Variable { Variable { id: "o".into(), domain: Domain::UserDefined("Operator".into()), assumptions: AssumptionSet::default() } }
fn var_x() -> Variable { Variable { id: "x".into(), domain: Domain::Real, assumptions: AssumptionSet::default() } }

fn mem_e_t() -> MathIR {
    MathIR::Fn { name: "Mem".into(), args: vec![
        MathIR::Var(Box::new(var_e())),
        MathIR::Var(Box::new(var_t())),
    ] }
}

fn mem_e_compile_t() -> MathIR {
    MathIR::Fn { name: "Mem".into(), args: vec![
        MathIR::Var(Box::new(var_e())),
        MathIR::Fn { name: "Compile".into(), args: vec![MathIR::Var(Box::new(var_t()))] },
    ] }
}

fn reachable_t_a_b() -> MathIR {
    MathIR::Fn { name: "Reachable".into(), args: vec![
        MathIR::Var(Box::new(var_t())),
        MathIR::Var(Box::new(var_a())),
        MathIR::Var(Box::new(var_b())),
    ] }
}

fn reachable_compile_t_a_b() -> MathIR {
    MathIR::Fn { name: "Reachable".into(), args: vec![
        MathIR::Fn { name: "Compile".into(), args: vec![MathIR::Var(Box::new(var_t()))] },
        MathIR::Var(Box::new(var_a())),
        MathIR::Var(Box::new(var_b())),
    ] }
}

fn execute_o_t() -> MathIR {
    MathIR::Fn { name: "Execute".into(), args: vec![
        MathIR::Var(Box::new(var_o())),
        MathIR::Var(Box::new(var_t())),
    ] }
}

fn conducts_t_o() -> MathIR {
    MathIR::Fn { name: "Conducts".into(), args: vec![
        MathIR::Var(Box::new(var_t())),
        MathIR::Var(Box::new(var_o())),
    ] }
}

fn reachable_t_o() -> MathIR {
    MathIR::Fn { name: "Reachable".into(), args: vec![
        MathIR::Var(Box::new(var_t())),
        MathIR::Var(Box::new(var_o())),
    ] }
}

fn sigma_t_eq_1() -> MathIR {
    MathIR::Eq(
        Box::new(MathIR::Fn { name: "Sigma".into(), args: vec![MathIR::Var(Box::new(var_t()))] }),
        Box::new(MathIR::Const(Constant::Int(1))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_preservation() {
        let result = NaturalParser::parse("topology preservation").unwrap();
        match result {
            MathIR::ForAll(_, _, _) => {}
            _ => panic!("Expected ForAll"),
        }
    }

    #[test]
    fn test_reachability_preservation() {
        let result = NaturalParser::parse("reachability preservation").unwrap();
        match result {
            MathIR::ForAll(_, _, _) => {}
            _ => panic!("Expected ForAll"),
        }
    }

    #[test]
    fn test_conduction_soundness() {
        let result = NaturalParser::parse("conduction soundness").unwrap();
        match result {
            MathIR::ForAll(_, _, _) => {}
            _ => panic!("Expected ForAll"),
        }
    }

    #[test]
    fn test_exo_execution_condition() {
        let result = NaturalParser::parse("exo execution condition").unwrap();
        match result {
            MathIR::ForAll(_, _, body) => {
                match *body {
                    MathIR::ForAll(_, _, inner) => {
                        match *inner {
                            MathIR::Iff(_, _) => {}
                            _ => panic!("Expected Iff"),
                        }
                    }
                    _ => panic!("Expected ForAll"),
                }
            }
            _ => panic!("Expected ForAll"),
        }
    }

    #[test]
    fn test_pythagorean_identity() {
        let result = NaturalParser::parse("pythagorean identity").unwrap();
        match result {
            MathIR::ForAll(_, _, _) => {}
            _ => panic!("Expected ForAll"),
        }
    }

    #[test]
    fn test_unknown_phrase() {
        let result = NaturalParser::parse("unknown phrase xyz");
        assert!(result.is_err());
    }
}
