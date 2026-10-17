package ScopedEnds {
    part def Leaf { port p; }
    part def Branch { part leaf : Leaf; }
    part host {
        part a : Branch;
        part b : Branch;
        part c;
        connection missingFirst connect absent to b;
        connection missingLast connect a to b.missing;
        connection deepMissing connect a.leaf.missing to b.leaf.p;
        connection missingMiddle connect a.absent.p to b.leaf.p;
        connection bodyError connect a.leaf.p to b.leaf.p {
            part unrelated : Missing;
        }
        connection triple connect (a, b.missing, c);
        connection endBound connect [missingBound] a to b;
        connection healthy connect a.leaf.p to b.leaf.p;
    }
    part ambiguousHost {
        part duplicate;
        part duplicate;
        part good;
        connection wire connect duplicate to good;
    }
}

package FlowEnds {
    item def Fluid;
    part def Vessel { out item outlet : Fluid; in item inlet : Fluid; }
    part pipes {
        part a : Vessel;
        part b : Vessel;
        flow broken from a.missing to b.inlet;
        flow healthy from a.outlet to b.inlet;
    }
    action run { action a; then action b; }
}
