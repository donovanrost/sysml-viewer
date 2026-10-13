package AliasChain {
    alias Last for Middle;
    alias Middle for Original;
    part def Original;
    part instance : Last;
}
