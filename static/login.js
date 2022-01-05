const prevent = async (e) =>{
    e.preventDefault();
const data = new URLSearchParams(new FormData(e.currentTarget));
   let res = await fetch('/login',{
	method: 'POST',
	body: data
    }).then(function(response) {
	return response.text()}) 
}
document.forms["login"].addEventListener('submit', prevent)
