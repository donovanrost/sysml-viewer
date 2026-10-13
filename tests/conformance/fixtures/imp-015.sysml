package AliasConflict {
    part def Parent { part x; }
    part def Child :> Parent { alias x for Parent::x; }
}
