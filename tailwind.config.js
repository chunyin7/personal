/** @type {import('tailwindcss').Config} */
export default {
  content: ["./templates/**/*.html"],
  theme: {
    extend: {
      fontFamily: {
        cloister: ["Cloister", "serif"],
        serif: ["PT Serif", "serif"],
        yinit: ["Yinit", "serif"],
      },
    },
  },
};
