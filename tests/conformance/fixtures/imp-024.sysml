package RecursiveSource {
    package Nested { package Leaf; }
    private package Hidden { package Secret; }
    public import RecursiveConsumer::*;
}
package RecursiveConsumer { public import RecursiveSource::*::**; }
package MemberRecursive { private import RecursiveSource::Nested::**; }
package AllConsumer { private import all RecursiveSource::*; }
