/// <reference no-default-lib="true"/>
/// <reference lib="dom" />
/// <reference lib="esnext" />

document.getElements;
const img_load = (e) => {
  e.currentTarget.parentNode.querySelector(".spinner").remove();
};
const stickyElm = document.getElementById("topnav");
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
