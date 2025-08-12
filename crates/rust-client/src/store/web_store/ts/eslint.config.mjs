// @ts-check

import eslint from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  eslint.configs.recommended,
  tseslint.configs.strictTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        allowDefaultProject: ["eslint.config.mjs"],
        project: "./tsconfig.json",
      },
    },
    ignores: ["**/**.js", "**/node_modules/**"],
  }
);
