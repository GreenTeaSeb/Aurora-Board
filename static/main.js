/// <reference no-default-lib="true"/>
/// <reference lib="dom" />
/// <reference lib="esnext" />

document.getElements;
const img_load = (e) => {
  e.currentTarget.parentNode.querySelector(".spinner").remove();
};
const populate = (times) => {
  for (let i = 0; i < times; i++) {
    const content = document.getElementById("posts");
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
const search = document.getElementById("search").querySelector("input");
console.debug(search);
search.addEventListener("keyup", (e) => {
  console.debug(e.key);
  if (e.key === "Enter") {
    e.preventDefault();
    window.open("/search/" + encodeURIComponent(search.value), "_self");
  }
});

// const dropdown_button = document.getElementById("dropdown-button");
// dropdown_button.addEventListener("click", () => {
//   dropdown_button.parentElement.classList.toggle("visible");
// });
//
const observer = new IntersectionObserver(
  ([e]) => e.target.classList.toggle("isSticky", e.intersectionRatio < 1),
  { threshold: [1] }
);

observer.observe(stickyElm);
// populate(200);
