package Mission {
    private import Components::*;

    part vehicle {
        part a : SpecialDevice;
        part b : Device;
        connect a.signal to b.signal;
    }
}
