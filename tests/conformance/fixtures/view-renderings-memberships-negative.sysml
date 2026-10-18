package ViewRenderingMembershipsNegative {
    rendering def Rendering;
    view def Parent { render rendering displayed : Rendering; }
    view def Other { render rendering another : Rendering; }
    view def SameName :> Parent { render rendering displayed : Rendering; }
    view sameName : Parent { render rendering displayed : Rendering; }
    view def ProtectedParent { protected render rendering protectedRendering : Rendering; }
    view def ProtectedCombined :> ProtectedParent, Parent;
    view def Reexport { public import Parent::displayed; }
    view def ImportedCombined :> Reexport, Other;
    view importedCombined : ImportedCombined;
    view def ValidPeer :> Parent;
}
