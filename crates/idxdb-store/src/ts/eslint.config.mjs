import eslint from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  eslint.configs.recommended,
  tseslint.configs.recommendedTypeChecked,
  {
    files: ["**/*.ts"],
    languageOptions: {
      parserOptions: {
        project: ["../tsconfig.json"],
        tsconfigRootDir: import.meta.dirname,
      },
    },
    // This can be a bit annoying since we're used to use `let` coming from rust, but we can add it back.
    rules: { "prefer-const": "off" },
  },
  {
    ignores: ["js/**", "**/node_modules/**", "**/*.mjs", "**/*.test.ts"],
  }
);
