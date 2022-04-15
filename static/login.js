const errors = document.getElementById("errors");
const prevent =  (e) =>{
    e.preventDefault();
   fetch(e.target.action, {
      method: 'POST',
      body: new URLSearchParams(new FormData(e.target)) // event.target is the form
   }).then(res => {
       if(!res.ok)
	   return res.text().then(text => {throw new Error(text)})
       window.location.replace(res.url);
   }).catch(err =>{
       errors.innerText = err;
   })
     
;}

document.querySelector('form').addEventListener('submit', prevent)
