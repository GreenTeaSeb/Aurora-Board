/// <reference no-default-lib="true"/>
/// <reference lib="dom" />
/// <reference lib="esnext" />

import Bricks from "./libs/bricks.js";
document.getElements
const img_load = (e) => {
  e.currentTarget.parentNode.querySelector(".spinner").remove();
};
const populate = (times) => {
  for (let i = 0; i < times; i++) {
    const content = document.getElementById("content");
    const container = document
      .getElementById("tile-template")
      .content.firstElementChild.cloneNode(true);

    const h = Math.floor(Math.random() * (400 - 100 + 1) + 100);
    const img = container.querySelector("img");
    img.setAttribute("src", "https://placekitten.com/900/" + (h + 400));

    container.style.height = h + "px";
    content.appendChild(container);
    img.addEventListener("load", img_load, { once: true });
  }
};


const stickyElm = document.getElementById("topnav");
const dropdown_button = document.getElementById("dropdown-button");
const search = document.getElementById("search").querySelector("input");
console.debug(search);
search.addEventListener("keyup", (e) => {
  console.debug(e.key);
  if (e.key === "Enter") {
    e.preventDefault();
    window.open("/search/" + encodeURIComponent(search.value), "_self");
  }
});

dropdown_button.addEventListener("click", () => {
  dropdown_button.parentElement.classList.toggle("visible");
});

const observer = new IntersectionObserver(
  ([e]) => e.target.classList.toggle("isSticky", e.intersectionRatio < 1),
  { threshold: [1] }
);

observer.observe(stickyElm);

const sizes = [
  { columns: 1, gutter: 10 }, // mobile
  { mq: "481px", columns: 2, gutter: 12 }, // tablet
  { mq: "1024px", columns: 3, gutter: 12 }, // laptop
  { mq: "1900px", columns: 5, gutter: 12 }, // desktop
];
const masonry = Bricks({
  container: "#content",
  packed: "data-packed",
  sizes: sizes,
});
masonry.on("resize", (size) => console.log("new column count"));
window.addEventListener("scroll", () => {
  if (
    Math.floor(Math.round((window.scrollY / window.scrollMaxY) * 100)) >= 90
  ) {
    populate(20);
    masonry.update();
  }
});

populate(200);
masonry.resize(true).pack();
