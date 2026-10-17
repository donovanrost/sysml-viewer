package StateSubactionsNegative {
    state def DuplicateEntry { entry; entry action another; }
    state def DuplicateDo { do action running; do; }
    state def DuplicateExit { exit; exit; }
    state duplicateEntry { entry; entry; }
    state duplicateDo { do; do action another; }
    state duplicateExit { exit action leaving; exit; }
    state def ValidPeer { entry; do; exit; }
}
