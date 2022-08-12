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
const boardimg_modal = document.getElementById("boardimg-modal");
const boardimg_button = document.getElementById("boardimg-edit");
const edit_modal = document.getElementById("edit-modal");
const edit_button = document.getElementById("edit-button");
const posts = document.getElementById("posts");
const posts_loading = posts.querySelector(".loading");
const main = document.getElementById("content");
const query = window.location.search;
const params = new URLSearchParams(query);
const baseurl = window.location.pathname;
var offset = params.has("page") ? parseInt(params.get("page")) : 0;
let parser = new DOMParser();

const tile_template = document.getElementById("tile-template");

// newpost_modal.showModal();

const get_posts = async () => {
  if (main.offsetHeight + main.scrollTop >= main.scrollHeight) {
    posts_loading.style.display = "flex";
await new Promise(r => setTimeout(r, 2000));
    const url = baseurl + "?page=" + (offset + 1);
    console.debug("Fetching at: " + url);
    const res = await fetch(url);
    const html = await res.text();
    let html_parsed = parser.parseFromString(html, "text/html");
    const page = html_parsed.querySelector(".page");
    if (!page.childElementCount) {
	// alert("no more posts")
      posts_loading.style.display = "none";
      await new Promise((r) => setTimeout(r, 2000));
      return;
    }
    offset += 1;
    // history.pushState({ page: offset }, "page " + offset, "?page=" + offset);
    posts.insertBefore(page, posts_loading);
  }
};

const close_modal = (m) => {
    m.closest('.modal').close();
}

newpost_button.addEventListener("click", () => {
  newpost_modal.showModal();
});
boardimg_button.addEventListener("click", () => {
  boardimg_modal.showModal();
});

edit_button.addEventListener("click", () => {
    edit_modal.showModal();
});

main.addEventListener("scroll", get_posts);

