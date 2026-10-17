package KerMLEnds {
    class Leaf { feature p; }
    class Branch { feature leaf : Leaf; }
    feature a : Branch;
    feature b : Branch;
    connector broken from a.leaf.missing to b.leaf.p;
    connector bodyError from a.leaf.p to b.leaf.p {
        feature unrelated : Missing;
    }
    connector bound from [missingBound] a to b;
    connector triple (a, b.missing, a.leaf.p);
    connector healthy from a.leaf.p to b.leaf.p;
}
