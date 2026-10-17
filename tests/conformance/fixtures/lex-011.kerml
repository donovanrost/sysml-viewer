// Quoted keywords, short names, aliases, case and dialect boundaries.
package KeywordsKerML {
    class <'true'> 'class';
    alias <'false'> 'return' for 'class';
    feature quotedType : 'class';
    feature viaShort : 'true';
    feature viaAlias : 'return';
    feature viaAliasShort : 'false';
    class part;
    feature foreign : part;
    class CLASS;
    feature uppercase : CLASS;
    class 'new';
    feature quotedNew : 'new';
    feature 'and' = 3;
    feature bound['and'];
}
