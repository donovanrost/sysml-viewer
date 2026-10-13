package InheritedBounds {
    part def Assembly { part base [2..5]; part refined :> base; part fixed [3] :> base; }
}
