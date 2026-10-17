package StateSubactionsPositive {
    state def Empty;
    state def EntryOnly { entry; }
    state def DoOnly { do; }
    state def ExitOnly { exit; }
    state def Anonymous { entry; do; exit; }
    state def Named { entry action entering; do action running; exit action leaving; }
    state def Parent { entry; do; exit; }
    state def Child :> Parent { entry; do; exit; }
    state instance : Parent { entry; do; exit; }
    state anonymous { entry; do; exit; }
    state named { entry action entering; do action running; exit action leaving; }
}
