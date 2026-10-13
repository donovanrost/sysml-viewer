package ControlFunctions {
    calc def 'if' { in test; in thenValue; in elseValue; return result; }
}
package ShadowedConditional {
    attribute count : ScalarValues::Integer = if true ? 3 else 4;
    part sized [count];
}
