export default {
  "*.{ts,js,vue}": (filenames: string[]) => [
    `prettier --write ${filenames.join(" ")}`,
    `eslint --fix ${filenames.join(" ")}`,
    "vue-tsc --noEmit",
  ],
  "src/**/*.{ts,vue}": (filenames: string[]) => [
    `prettier --write ${filenames.join(" ")}`,
    `eslint --fix ${filenames.join(" ")}`,
    `vitest related --run ${filenames.join(" ")}`,
    "vue-tsc --noEmit",
  ],
  "src-tauri/**/*.rs": ["rustfmt --edition 2021", () => "cd src-tauri && cargo test"],
  "*.{json,md,html}": ["prettier --write"],
};
