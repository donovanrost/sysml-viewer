package ViewRenderingIncomplete {
    rendering def Rendering;
    view def Parent { render rendering displayed : Rendering; }
    view def Other { render rendering another : Rendering; }
    view def MissingBase :> Parent, Other, Missing;
    view def MissingChild :> MissingBase;
    view missing : MissingChild;
    view def MissingRedefinition :> Parent, Other {
        render rendering replacement :>> Missing::displayed;
    }
    view def MissingRedefinitionChild :> MissingRedefinition;
    view def MissingImport { public import Missing::*; }
    view def MissingImportChild :> MissingImport, Parent, Other;
    view def CycleA :> CycleB, Parent;
    view def CycleB :> CycleA, Other;
    view def DefiniteOwned :> Missing {
        render rendering one : Rendering;
        render rendering two : Rendering;
    }
    view definiteOwned : Missing {
        render rendering one : Rendering;
        render rendering two : Rendering;
    }
    view def CompleteInvalidPeer :> Parent, Other;
    view completeInvalidPeer : CompleteInvalidPeer;
    view def ValidPeer :> Parent;
}
