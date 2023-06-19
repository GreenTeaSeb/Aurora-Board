const search_options = document.getElementById('search-space')

console.log("aaa")

document.getElementById('search').addEventListener( 'submit', (e) => {
    e.preventDefault();
    switch(search_options.value){
        case "global":{
            e.target.action = "/"
        }
    }
    e.target.submit();
} )