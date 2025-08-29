module.exports = [
  {
    // Ignore patterns
    ignores: [
      "crates/web-client/dist/**/*",
      "target/**/*",
      "**/target/**/*",
      "miden-node/**/*",
      "**/*.d.ts",
      "docs/book/**/*",
      // TODO: Instead of ignoring the web_store, we should check if we're able to merge that
      // eslint config file with this one.
      "crates/rust-client/src/store/web_store/**",
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
    },
  },
];
