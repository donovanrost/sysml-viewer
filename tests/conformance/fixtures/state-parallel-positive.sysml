package StateParallelPositive {
    state def Driver {
        action outsideA;
        action outsideB;
        state def SuccessionOnly parallel {
            action a;
            action b;
            first a then b;
        }
        state successionOnly parallel {
            action a;
            action b;
            first a then b;
        }
        state def Unrelated parallel {
            state local;
            transition first outsideA then outsideB;
        }
        state unrelated parallel {
            state local;
            transition first outsideA then outsideB;
        }
        state aliases parallel {
            alias borrowed for outsideA;
            transition first borrowed then outsideB;
        }
        state def Deep parallel {
            state region {
                state a;
                state b;
                transition first a then b;
            }
        }
        state def Serial {
            state a;
            state b;
            transition first a then b;
        }
        state def Inherited :> Serial parallel;
        state inherited : Serial parallel;
        state def Empty parallel;
        state empty parallel;
    }
}
