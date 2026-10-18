package StateParallelNegative {
    state def Driver {
        action outside;
        state def Outgoing parallel { action ownedAction; }
        transition first Outgoing::ownedAction then outside;
        state def Incoming parallel { action ownedAction; }
        transition first outside then Incoming::ownedAction;
        state outgoing parallel { action ownedAction; }
        transition first outgoing.ownedAction then outside;
        state incoming parallel { action ownedAction; }
        transition first outside then incoming.ownedAction;
        state def Local parallel {
            state a;
            state b;
            transition first a then b;
        }
        state local parallel {
            state a;
            transition first a then a;
        }
        state performed parallel { perform action local references outside; }
        transition first performed.local then outside;
        state subactions parallel { entry action entering; do action running; exit action leaving; }
        transition first subactions.entering then outside;
        transition first outside then subactions.leaving;
        state def ValidPeer parallel { state local; }
    }
}
