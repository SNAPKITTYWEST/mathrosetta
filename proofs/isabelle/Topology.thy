theory Topology
  imports Topology_Syntax
begin

record node =
  nid :: nat
  kind :: port_kind
  voltage :: real
  is_floating :: bool

record edge =
  eid :: nat
  src :: nat
  dst :: nat
  conductance :: real
  is_bus :: bool

record topology =
  nodes :: "node list"
  edges :: "edge list"

definition build_topology :: "fact list ⇒ topology" where
  "build_topology facts ≡
     ⦇nodes = map (λf. case f of
                           NodeFact n k ⇒ ⦇nid = n, kind = k, voltage = 0.0, is_floating = False⦇)
                    (filter (λf. case f of NodeFact _ _ ⇒ True | _ ⇒ False) facts),
          edges = map (λf. case f of
                             EdgeFact e s d g b ⇒ ⦇eid = e, src = s, dst = d, conductance = g, is_bus = b⦇)
                     (filter (λf. case f of EdgeFact _ _ _ _ _ ⇒ True | _ ⇒ False) facts)⦇"

definition well_formed_netlist :: "fact list ⇒ bool" where
  "well_formed_netlist facts ≡
     let T = build_topology facts;
         non_gnd_nodes = [n ← nodes T. kind n ≠ Gnd];
         edge_srcs = set (map src (edges T));
         edge_dsts = set (map dst (edges T));
         all_connected = edge_srcs ∪ edge_dsts
     in ∀n ∈ set non_gnd_nodes. nid n ∈ all_connected"

definition exo_cell_conductance :: "bool ⇒ bool ⇒ real" where
  "exo_cell_conductance p_on pn_on ≡
     (if p_on then 1/50 else 1/1e12) + (if pn_on then 1/50 else 1/1e12)"

definition bus_segment_conductance :: "real" where
  "bus_segment_conductance ≡ 1 / (0.1 * 500)"

definition annotated_with_va_params :: "fact list ⇒ bool" where
  "annotated_with_va_params facts ≡
     let T = build_topology facts
     in ∀e ∈ set (edges T).
          (is_bus e ⟶ conductance e = bus_segment_conductance) ∧
          (¬ is_bus e ⟶
             (∃p_on pn_on. conductance e = exo_cell_conductance p_on pn_on))"

end
