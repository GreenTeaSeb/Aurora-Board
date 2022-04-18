// import { init, exec } from "/pell/src/pell.js";
//
// const editor = init({
//   element: document.getElementById("editor"),
//   onChange: (html) => {
//     // document.getElementById("html-output").textContent = html;
//   },
//   defaultParagraphSeparator: "p",
//   styleWithCSS: true,
//   actions: [
//     "bold",
//     "italic",
//     "heading1",
//     "heading2",
//     "olist",
//     "ulist",
//     "image",
//     "link",
//   ],
// });

const newpost_button = document.getElementById("newpost-button");
const newpost_modal = document.getElementById("newpost-modal");
newpost_button.addEventListener('click', () => {
    newpost_modal.showModal();
})
