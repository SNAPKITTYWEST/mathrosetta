/-- Build topology from pure facts -/
namespace Sovereign

structure Node where
  id : Nat
  kind : PortKind
  voltage : Rat
  floating : Bool
deriving Repr, DecidableEq

structure Edge where
  id : Nat
  src : Nat
  dst : Nat
  conductance : Rat
  isBus : Bool
deriving Repr, DecidableEq

structure Topology where
  nodes : List Node
  edges : List Edge
deriving Repr, DecidableEq

def buildTopology (facts : List Fact) : Topology :=
  { nodes := facts.filterMap (fun f =>
      match f with
      | Fact.node n k => some { id := n, kind := k, voltage := 0, floating := false }
      | _ => none),
    edges := facts.filterMap (fun f =>
      match f with
      | Fact.edge e s d g b => some { id := e, src := s, dst := d, conductance := g, isBus := b }
      | _ => none) }

def wellFormedNetlist (facts : List Fact) : Prop :=
  let T := buildTopology facts
  let nonGnd := T.nodes.filter (fun n => !(n.kind == PortKind.gnd))
  let connected : Finset Nat :=
    (T.edges.map (fun e => e.src) ++ T.edges.map (fun e => e.dst)).toFinset
  ∀ n ∈ nonGnd, n.id ∈ connected

def exoCellConductance (pOn pnOn : Bool) : Rat :=
  (if pOn then 1 / 50 else 1 / 1_000_000_000_000) +
  (if pnOn then 1 / 50 else 1 / 1_000_000_000_000)

def busSegmentConductance : Rat := 1 / 50

def annotatedWithVaParams (facts : List Fact) : Prop :=
  let T := buildTopology facts
  ∀ e ∈ T.edges,
    (e.isBus → e.conductance = busSegmentConductance) ∧
    (¬e.isBus → ∃ (pOn pnOn : Bool), e.conductance = exoCellConductance pOn pnOn)

end Sovereign
