package ViewRenderingsOwnedNegative {
    rendering def Rendering;
    view def Duplicate {
        render rendering firstRendering : Rendering;
        render rendering secondRendering : Rendering;
    }
    view duplicate {
        render rendering firstRendering : Rendering;
        render rendering secondRendering : Rendering;
    }
    view def ValidPeer { render rendering singleRendering : Rendering; }
}
