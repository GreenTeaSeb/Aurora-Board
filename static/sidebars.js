// sidebar swipes
console.log("START");
console.log("window y:" + window.innerWidth);
let x_start = 0;
let x_end = 0;
let y = 0;
let threshold = 40;
let currentPage = "posts";

let started = false;

const content = document.getElementById("content");
const pages = document.getElementById("posts");
const sidebar_left = document.getElementById("sidebar-left");
const sidebar_right = document.getElementById("sidebar-right");

document.addEventListener("touchstart", (e) => {
  x_start = e.changedTouches[0].screenX;
  y = e.changedTouches[0].screenY;
});
document.addEventListener(
  "touchmove",
  (e) => {
    let x_cur = e.changedTouches[0].screenX;
    let diff = (((x_start - x_cur) / window.innerWidth) * 100).toFixed(2);
    if (Math.abs(y - e.touches[0].screenY) < 10) {
      started = true;
      content.style.overflowY = "hidden";
      switch (currentPage) {
        case "posts": {
          pages.style.transition = "none";
          if (diff > 0) {
            content.dataset.curPage = "right";
            pages.style.marginLeft = -diff + "%";
            pages.style.marginRight = "0";
          } else if (diff < 0) {
            content.dataset.curPage = "left";
            pages.style.marginRight = diff + "%";
            pages.style.marginLeft = "0";
          }
          break;
        }
        case "left": {
          sidebar_left.style.transition = "none";
          if (diff >= 20) {
            sidebar_left.style.marginLeft = -diff + "%";
          }
          break;
        }
        case "right": {
          sidebar_right.style.transition = "none";
          if (diff < -20) {
            sidebar_right.style.marginRight = diff + "%";
          }
          break;
        }
        default:
          break;
      }
    }
  },
  { passive: false }
);
document.addEventListener("touchend", (e) => {
  started = false;
  x_end = e.changedTouches[0].screenX;
  let dist = ((x_start - x_end) / window.innerWidth) * 100;
  switch (currentPage) {
    case "posts": {
      pages.style.transition = "0.3s";
      if (dist > threshold) {
        sidebar_right.style.transition = "0.3s";
        sidebar_right.style.marginRight = "0";
        pages.style.marginLeft = "-100%";
        currentPage = "right";
      } else if (dist < -threshold) {
        sidebar_left.style.transition = "0.3s";
        sidebar_left.style.marginLeft = "0";
        pages.style.marginRight = "-100%";
        currentPage = "left";
      } else if (Math.abs(dist) < threshold) {
        pages.style.marginLeft = 0;
        pages.style.marginRight = 0;
        content.style.overflowY = "scroll";
      }
      break;
    }
    case "left": {
      if (dist > threshold) {
        pages.style.transition = "0.3s";
        pages.style.marginRight = "0";
        sidebar_left.style.marginLeft = "-100%";
        currentPage = "posts";
      } else if (dist < threshold) {
        sidebar_left.style.transition = "0.3s";
        sidebar_left.style.marginLeft = "0";
        pages.style.marginRight = "-100%";
      }
      break;
    }
    case "right": {
      if (dist < -threshold) {
        pages.style.transition = "0.3s";
        pages.style.marginLeft = "0";
        sidebar_right.style.marginRight = "-100%";
        currentPage = "posts";
      } else if (dist > -threshold) {
        sidebar_right.style.transition = "0.3s";
        sidebar_right.style.marginRight = "0";
        pages.style.marginLeft = "-100%";
      }
      break;
    }
    default:
      break;
  }
  setTimeout(function() {  content.dataset.curPage = currentPage;
  }, 300);
});
