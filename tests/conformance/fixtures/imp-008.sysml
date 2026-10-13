package Bindings {
    part def <D> Device;
    alias <C> Copy for Device;
    private alias Local for Device;
    alias <OnlyShort> for Device;
    part initial : D;
    part second : Copy;
    part third : C;
    part fourth : OnlyShort;
}
