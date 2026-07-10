theory Topology_Syntax
  imports Main
begin

datatype port_kind =
    Sig | PortP | PortPN | OpGate | Gnd | BusIn | BusMid | BusOut

datatype fact =
    NodeFact nat port_kind
  | EdgeFact nat nat nat real bool

definition pure_fact_file :: "fact list ⇒ bool" where
  "pure_fact_file facts ⟷ (∀f ∈ set facts. case f of NodeFact _ _ ⇒ True | EdgeFact _ _ _ _ _ ⇒ True)"

lemma pure_fact_file_trivial: "pure_fact_file facts"
  unfolding pure_fact_file_def by simp

end
