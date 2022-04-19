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
const posts = document.getElementById("posts");
const main = document.getElementById("content");
const query = window.location.search;
const params = new URLSearchParams(query);
const baseurl = window.location.pathname;
var offset = params.has("page") ? parseInt(params.get("page")) : 0;
let parser = new DOMParser();

const tile_template = document.getElementById("tile-template");

  newpost_modal.showModal();

const get_posts = async () => {
  if (main.offsetHeight + main.scrollTop >= main.scrollHeight) {
    const url = baseurl + "?page=" + (offset + 1);
    console.debug("Fetching at: " + url);
    const res = await fetch(url);
    const html = await res.text();
    let html_parsed = parser.parseFromString(html, "text/html");
    const page = html_parsed.querySelector(".page");
    if (!page.childElementCount) return;
    offset += 1;
    // history.pushState({ page: offset }, "page " + offset, "?page=" + offset);
    posts.appendChild(page);
  }
};

newpost_button.addEventListener("click", () => {
  newpost_modal.showModal();
});

main.addEventListener("scroll", get_posts);
