package ProtectedFixture {
    part def Parent {
        protected part def HiddenType;
        private part def PrivateType;
    }
    part def Child :> Parent { part good : HiddenType; }
    part outside : Parent::HiddenType;
    part def PrivateChild :> Parent { part bad : PrivateType; }
}
