const errors = document.getElementById("errors");
const form = document.querySelector("form");
const prevent_file = (e) => {
  e.preventDefault();
  const formData = new FormData(form);
  fetch(e.target.action, {
    method: "POST",
    body: formData, // event.target is the form
  })
    .then((res) => {
      if (!res.redirected)
        return res.text().then((text) => {
          throw new Error(text);
        });
      window.location.replace(res.url);
    })
    .catch((err) => {
      errors.innerText = err;
    });
};


const prevent = (e) => {
  e.preventDefault();
  fetch(e.target.action, {
    method: "POST",
    body: new URLSearchParams(new FormData(e.target)),
  })
    .then((res) => {
      if (!res.redirected)
        return res.text().then((text) => {
          throw new Error(text);
        });
      window.location.replace(res.url);
    })
    .catch((err) => {
	errors.style.display = "block"
      errors.innerText = err;
    });
}

form.addEventListener('submit', prevent)
