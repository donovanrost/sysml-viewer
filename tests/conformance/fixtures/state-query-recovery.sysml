package StateQueryRecovery {
    state def Recovered {
        entry action entering;
        attribute invalid = "bad\q";
        exit action leaving;
    }
}
