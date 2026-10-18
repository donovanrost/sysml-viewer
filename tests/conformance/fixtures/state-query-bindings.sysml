package StateQueries {
    action external;
    state def Ordered {
        exit action leaving;
        entry action entering;
        do action running;
        action ordinary;
        alias namedAlias for ordinary;
    }
    state def Inherited :> Ordered;
    state inherited : Ordered;
    state local : Ordered { entry action start; }
    state def Anonymous { entry; do; exit; }
    state def Duplicates { entry action firstEntry; entry action secondEntry; }
    state missing { entry action unresolved : Missing; }
    state performed { entry action invoked references external; }
    state libraryState : States::StateAction;
    part def NotState;
}
