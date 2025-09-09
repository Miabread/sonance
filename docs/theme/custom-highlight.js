hljs.registerLanguage("sonance", (hljs) => ({
    name: "Sonance",
    keywords: {
        keyword: "module import export public type trait func do block let mut set match",
        built_in: "String Result Option Boolean U32",
        literal: "Pass Fail Some None True False Unit",
    },
    contains: [
        hljs.QUOTE_STRING_MODE,
        hljs.C_NUMBER_MODE,
        hljs.C_BLOCK_COMMENT_MODE,
        hljs.C_LINE_COMMENT_MODE,
    ],
}));

hljs.initHighlightingOnLoad();
