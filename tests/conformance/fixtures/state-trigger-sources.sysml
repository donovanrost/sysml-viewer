package StateTriggerSources {
    item def Signal;
    state def Driver {
        state a;
        state b;
        action ordinary;
        state region { state nested; perform action work references nested; }
        perform action invoked references a;
        transition performed first invoked accept payload : Signal then b;
        transition performChain first region.work accept payload : Signal then b;
        transition direct first a accept payload : Signal then b { alias unrelated for ordinary; }
        transition chained first region.nested accept payload : Signal then b;
        transition broken first region.nested.missing accept payload : Signal then b;
        transition missing first Missing accept payload : Signal then b;
        transition invalid first ordinary accept payload : Signal then b;
        transition untriggered first ordinary then b;
    }
}
