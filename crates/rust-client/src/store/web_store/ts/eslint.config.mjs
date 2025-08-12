import eslint from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  eslint.configs.recommended,
  tseslint.configs.recommendedTypeChecked,
  {
    // Apply type-checked rules ONLY to TypeScript files
    files: ["**/*.ts"],
    languageOptions: {
      parserOptions: {
        project: ["./tsconfig.json"],
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    // Global ignore patterns (relative to project root)
    ignores: [
      "js/**", // Ignore entire js directory
      "**/node_modules/**",
      "**/*.mjs",
    ],
  }
);
