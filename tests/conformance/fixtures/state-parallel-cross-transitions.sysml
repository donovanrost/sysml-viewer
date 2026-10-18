package StateParallelCrossTransitions {
    private import StateParallelCross::*;
    state def Driver {
        action outside;
        transition first Outgoing::local then outside;
        transition first outside then incoming.local;
    }
}
