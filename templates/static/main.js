window.onload = () => {
  const updateImages = () => {
    document.querySelectorAll("img.reload").forEach((el) => {
      if (el.complete) {
        el.src = el.src.split("?")[0] + "?" + new Date().getTime();
      }
    });
  }

  setInterval(updateImages, 200);
}


const gotoPosition = async (x, y) => {
  document.querySelector("#loading").classList.add("reload");
  document.querySelector("#camera-stream").style.display = '';
  await fetch("/action/goto?x=" + x + "&y=" + y);
  document.querySelector("#loading").classList.remove("reload");
  document.querySelector("#loading").style.display = 'none';
}

const checkPosition = async (x, y) => {
  document.querySelector("#loading").style.display = '';
  let image = document.querySelector("#current-card-image");
  image.src = "/camera/snapshot";
  image.classList.add("reload");
  await fetch("/action/check?x=" + x + "&y=" + y);
  document.querySelector("#loading").style.display = 'none';
  window.location.reload();
}

const waterPosition = async (x, y) => {
  document.querySelector("#loading").style.display = '';
  let image = document.querySelector("#current-card-image");
  image.src = "/camera/snapshot";
  image.classList.add("reload");
  await fetch("/action/water?x=" + x + "&y=" + y);
  document.querySelector("#loading").style.display = 'none';
  window.location.reload();
}
const checkStage = async (stage) => {
  document.querySelector("#loading").style.display = '';
  await fetch("/action/check/" + stage);
  document.querySelector("#loading").style.display = 'none';
  window.location.reload();
}

const waterStage = async (stage) => {
  document.querySelector("#loading").style.display = '';
  await fetch("/action/water/" + stage);
  document.querySelector("#loading").style.display = 'none';
  window.location.reload();
}
