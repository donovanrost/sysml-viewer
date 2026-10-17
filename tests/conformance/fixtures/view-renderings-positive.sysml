package ViewRenderingsPositive {
    rendering def Rendering;
    view def Empty;
    view def Parent { render rendering displayed : Rendering; }
    view def Child :> Parent;
    view instance : Parent;
    view def Left :> Parent;
    view def Right :> Parent;
    view def Diamond :> Left, Right;
    view diamond : Diamond;
}
