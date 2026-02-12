module.exports = [
  {
    // Ignore patterns
    ignores: [
      "crates/web-client/dist/**/*",
      "target/**/*",
      "**/target/**/*",
      "miden-node/**/*",
      "**/*.d.ts",
      "docs/**/*",
      "crates/idxdb-store/src/**",
      "crates/wasm-sqlite-store/src/**",
      "crates/react-sdk/dist/**",
      "packages/react-sdk/**",
    ],
  },
  {
    // Configuration for JavaScript files
    files: ["**/*.js", "**/*.jsx"],
    languageOptions: {
      parserOptions: {
        ecmaVersion: 2022,
        sourceType: "module",
      },
    },
    rules: {
      camelcase: ["error", { properties: "always" }],
      semi: ["error", "always"],
      "keyword-spacing": [
        "error",
        {
          before: true,
          after: true,
        },
      ],
      "comma-dangle": [
        "error",
        {
          arrays: "always-multiline",
          objects: "always-multiline",
          imports: "always-multiline",
          exports: "always-multiline",
          functions: "never",
        },
      ],
      "eol-last": ["error", "always"],
      "space-before-blocks": ["error", "always"],
      "no-multiple-empty-lines": [
        "error",
        {
          max: 1,
          maxBOF: 0,
          maxEOF: 0,
        },
      ],
    },
  },
  {
    files: ["**/*.ts", "**/*.tsx"],
    ignores: ["crates/rust-client/*"],
    languageOptions: {
      parser: require("@typescript-eslint/parser"),
      parserOptions: {
        ecmaVersion: 2022,
        sourceType: "module",
        project: "crates/web-client/tsconfig.json", // path to your tsconfig file
        tsconfigRootDir: __dirname,
      },
    },
    rules: {
      camelcase: ["error", { properties: "always" }],
      semi: ["error", "always"],
      "keyword-spacing": [
        "error",
        {
          before: true,
          after: true,
        },
      ],
      "comma-dangle": [
        "error",
        {
          arrays: "always-multiline",
          objects: "always-multiline",
          imports: "always-multiline",
          exports: "always-multiline",
          functions: "never",
        },
      ],
      "eol-last": ["error", "always"],
      "space-before-blocks": ["error", "always"],
      "no-multiple-empty-lines": [
        "error",
        {
          max: 1,
          maxBOF: 0,
          maxEOF: 0,
        },
      ],
    },
  },
];
