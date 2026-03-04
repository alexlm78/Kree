/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",     // Indica que busque clases en tus archivos Rust
    "./index.html",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
