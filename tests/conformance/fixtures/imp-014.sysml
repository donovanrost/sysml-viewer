package AliasHeritage {
    part def Parent { part def Nested; protected alias renamed for Nested; private alias hidden for Nested; }
    part def Child :> Parent;
}
