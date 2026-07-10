/-- Inductive reachability on topology -/
namespace Sovereign

def Adj (T : Topology) (u v : Nat) : Prop :=
  ∃ e ∈ T.edges, e.src = u ∧ e.dst = v

inductive Reachable (T : Topology) : Nat → Nat → Prop
| edge : Adj T u v → Reachable T u v
| trans : Reachable T u v → Reachable T v w → Reachable T u w

def TopologyEquivalent (A B : Topology) : Prop :=
  A.nodes.toFinset = B.nodes.toFinset ∧
  A.edges.toFinset = B.edges.toFinset

theorem topologyEquivalent_refl (T : Topology) : TopologyEquivalent T T := by
  constructor <;> simp [Finset.ext_iff]

def ReachabilityPreserved (A B : Topology) : Prop :=
  ∀ u v, Reachable A u v ↔ Reachable B u v

theorem reachabilityPreserved_refl (T : Topology) : ReachabilityPreserved T T := by
  intro u v
  constructor <;> intro h <;> exact h

end Sovereign
