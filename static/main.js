let pots = [];
let zoom = 100;
let scale = 100;

const addPot = (pot) => {
  let potId = getPotId(pot.x, pot.y);
  pots[potId] = pot;
}

const getPotId = (x, y) => {
  return "pot" + x + "-" + y;
}
const removePot = (x, y) => {
  let id = getPotId(x, y);
  if (pots[id] != null) {
    pots[id] = null;
  }
  let el = document.getElementById(id);
  if (el) {
    el.remove();
  }
}

const refreshPots = () => {
  for (let potId in pots) {
    let pot = pots[potId];
    let potEl = document.querySelector("#pots > #" + potId);
    if (potEl) {
    } else {
      let potElement = document.getElementById("pot-template").content.cloneNode(true);
      potEl = potElement.querySelector(".pot");
      potEl.id = potId;
      document.querySelector("#pots").appendChild(potElement);
    }
    potEl.style.left = zoom / 100 * (pot.x - scale / 100 * pot.left) + "mm";
    potEl.style.top = zoom / 100 * (pot.y - scale / 100 * pot.top) + "mm";
    potEl.style.width = zoom / 100 * scale / 100 * (pot.left + pot.right) + "mm";
    potEl.style.height = zoom / 100 * scale / 100 * (pot.top + pot.bottom) + "mm";
    potEl.classList.add("is-" + pot.stage.toLowerCase());
  }
}

const refresh = () => {
  refreshPots();
  document.querySelector("#zoom-percentage").innerHTML = zoom + "%";
  document.querySelector("#scale-percentage").innerHTML = scale + "%";
}

const zoomOut = () => {
  zoom -= zoom > 20 ? 10 : 0;
  refresh();
}
const zoomIn = () => {
  zoom += zoom < 500 ? 10 : 0;
  refresh();
}
const zoomReset = () => {
  zoom = 100;
  refresh();
}
const scaleOut = () => {
  scale -= scale > 20 ? 10 : 0;
  refresh();
}
const scaleIn = () => {
  scale += scale < 500 ? 10 : 0;
  refresh();
}
const scaleReset = () => {
  scale = 100;
  refresh();
}

const hideDetail = () => {
  document.body.classList.remove("show-detail");
}
const showDetail = () => {
  document.body.classList.add("show-detail");
}
const toggleDetail = () => {
  document.body.classList.toggle("show-detail");
}
const resetDetail = () => {
  document.querySelector("#detail").classList.remove("show-detail-pot");
  document.querySelector("#detail").classList.remove("show-detail-log");
  document.querySelector("#detail").classList.remove("show-detail-general");
}
const showDetailGeneral = () => {
  showDetail();
  resetDetail();
  document.querySelector("#detail").classList.add("show-detail-general")
}
const showDetailPot = () => {
  showDetail();
  resetDetail();
  document.querySelector("#detail").classList.add("show-detail-pot")
}
const showDetailLog = () => {
  showDetail();
  resetDetail();
  document.querySelector("#detail").classList.add("show-detail-log")
}

const openDetailPot = (ev) => {
  ev.stopPropagation();
  let potId = ev.target.parentElement.id;
  console.log(potId);
  showDetailPot();

  document.querySelector("#detail > .tabs > ul > li.pot-tab > a").innerHTML = potId;

  const pot = pots[potId];
  document.querySelector(".pot-position").innerHTML = '(' + pot.x + ', ' + pot.y + ')';
  document.querySelector(".pot-size").innerHTML = '(' + (pot.left + pot.right) + ', ' + (pot.top + pot.bottom) + ')';
  document.querySelector(".pot-stage").innerHTML = pot.stage;
}

const waterNow = async () => {
  fetch("/push")
}

// handling

const report_pot = (pot) => {
  if (pot) {
    addPot(pot);
    refresh()
  }
}

const report_status = (s) => {
  if (s) {
    let logEl = document
      .getElementById("log-template")
      .content
      .cloneNode(true)
      .querySelector(".notification");

    logEl.innerHTML = s;
    document.querySelector(".detail-log").prepend(logEl);
  }
}

const report_err = (s) => {
  if (s) {
    let logEl = document
      .getElementById("err-template")
      .content
      .cloneNode(true)
      .querySelector(".notification");

    logEl.innerHTML = s;
    document.querySelector(".detail-log").prepend(logEl);
  }
}

new EventSource("/pull").onmessage = (event) => {
  let msg = JSON.parse(event.data);
  report_pot(msg.ReportPot);
  report_err(msg.Error);
  report_status(msg.Status);
};

// fads
//
report_status("Hell no");
report_err("Hell no");
