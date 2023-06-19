const like = (e, f) => {
  e.preventDefault();
  fetch(e.target.action, {
    method: "POST",
    body: new URLSearchParams(new FormData(e.target)),
  }).then((res) => {
    if (res.redirected) {
      window.location.replace(res.url);
      return;
    }
    f.classList.remove("disliked");
    f.classList.add("liked");
  });
};

const dislike = (e, f) => {
  e.preventDefault();
  fetch(e.target.action, {
    method: "POST",
    body: new URLSearchParams(new FormData(e.target)),
  }).then((res) => {
    if (res.redirected) {
      window.location.replace(res.url);
      return;
    }
    f.classList.remove("liked");
    f.classList.add("disliked");
  });
};
