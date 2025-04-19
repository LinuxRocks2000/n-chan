function before_submit() {
  localStorage.username = document.getElementsByClassName("username")[0].value;
}

window.onload = () => {
  if (localStorage.username) {
    for (el of document.getElementsByClassName("username")) {
      el.value = localStorage.username;
    }
  }
}
