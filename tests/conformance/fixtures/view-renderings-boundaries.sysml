package ViewRenderingBoundaries {
    rendering def Rendering;
    view def Parent { render rendering displayed : Rendering; }
    view def Other { render rendering another : Rendering; }
    view def Override :> Parent {
        render rendering displayed :>> Parent::displayed;
    }
    view def Renamed :> Parent {
        render rendering replacement :>> Parent::displayed;
    }
    view def Transitive :> Renamed {
        render rendering replacementAgain :>> Renamed::replacement;
    }
    view instance : Transitive;
    view def PrivateParent { private render rendering hidden : Rendering; }
    view def PrivateChild :> PrivateParent, Parent;
    view def ProtectedParent { protected render rendering visible : Rendering; }
    view def ProtectedChild :> ProtectedParent;
    view def Reexport { public import Parent::displayed; }
    view def Imported :> Reexport;
    view def PrivateReexport { private import Parent::displayed; }
    view def PrivateImported :> PrivateReexport, Other;
    view def OwnImport {
        public import Parent::displayed;
        render rendering local : Rendering;
    }
    view def Ordinary { rendering ordinary : Rendering; }
    view def OrdinaryChild :> Ordinary, Parent;
}
