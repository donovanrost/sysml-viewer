package ViewRenderingsInheritedNegative {
    rendering def Rendering;
    view def First { render rendering firstRendering : Rendering; }
    view def Second { render rendering secondRendering : Rendering; }
    view def Combined :> First, Second;
    view combined : Combined;
    view def LocalAndInherited :> First {
        render rendering localRendering : Rendering;
    }
    view localAndInherited : First {
        render rendering localRendering : Rendering;
    }
}
