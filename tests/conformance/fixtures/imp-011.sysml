package RecursiveSource {
    package Nested { part def Device; private part def Secret; }
}
package RecursiveClient {
    private import RecursiveSource::**;
    part good : Device;
    part bad : Secret;
}
