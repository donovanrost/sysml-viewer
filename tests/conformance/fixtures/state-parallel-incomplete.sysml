package StateParallelIncomplete {
    state def Driver {
        action outside;
        state def Unknown parallel {
            state local;
            transition first MissingSource then MissingTarget;
        }
        state unknown parallel {
            state local;
            transition first MissingSource then MissingTarget;
        }
        state sourceBroken parallel { action ownedAction; }
        transition first sourceBroken.ownedAction.missing then outside;
        state targetBroken parallel { action ownedAction; }
        transition first outside then targetBroken.ownedAction.missing;
        state middleBroken parallel { action ownedAction; }
        transition first middleBroken.missing.ownedAction then outside;
        state rootBroken parallel { action ownedAction; }
        transition first MissingRoot.rootBroken.ownedAction then outside;
        state sourceKnown parallel { action ownedAction; }
        transition first sourceKnown.ownedAction then MissingTarget;
        state targetKnown parallel { action ownedAction; }
        transition first MissingSource then targetKnown.ownedAction;
        state def SourceKnown parallel { action ownedAction; }
        transition first SourceKnown::ownedAction then MissingTarget;
        state def TargetKnown parallel { action ownedAction; }
        transition first MissingSource then TargetKnown::ownedAction;
        state def CompleteInvalidPeer parallel { action ownedAction; }
        transition first CompleteInvalidPeer::ownedAction then outside;
        state def Left parallel { action collision; }
        state def Right parallel { action collision; }
        state ambiguous parallel {
            private import Left::collision;
            private import Right::collision;
            transition first collision then collision;
        }
        state def ValidPeer parallel { state local; }
    }
}
