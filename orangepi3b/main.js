const getPotId = (x, y) => "pot" + x + "-" + y;
const getPots = () => localStorage.pots ? JSON.parse(localStorage.pots) : {};
const setPots = (value) => localStorage.pots = JSON.stringify(value);
const getPot = (potId) => localStorage.pots ? getPots()[potId] : null;
const setPot = (value) => {
  let pots = getPots();
  pots[getPotId(value.x, value.y)] = value;
  setPots(pots);
}
const removePot = (value) => {
  let pots = getPots();
  pots[getPotId(value.x, value.y)] = null;
  setPots(pots);
}

const addPot = (pot) => {
  setPot(pot);
  setCurrentPot(pot);
}

const getCurrentPot = () => localStorage.currentPot ? JSON.parse(localStorage.currentPot) : null;
const setCurrentPot = (value) => localStorage.currentPot = JSON.stringify(value);

const getZoom = () => localStorage.zoom ? JSON.parse(localStorage.zoom) : 100;
const setZoom = (value) => localStorage.zoom = JSON.stringify(value);
const getScale = () => localStorage.scale ? JSON.parse(localStorage.scale) : 100;
const setScale = (value) => localStorage.scale = JSON.stringify(value);

const refreshPots = () => {
  for (let potId in getPots()) {
    let pot = getPot(potId);
    let potEl = document.querySelector("#pots > #" + potId);
    if (potEl) {
    } else {
      let potElement = document.getElementById("pot-template").content.cloneNode(true);
      potEl = potElement.querySelector(".pot");
      potEl.id = potId;
      document.querySelector("#pots").appendChild(potElement);
    }
    potEl.style.left = getZoom() / 100 * (pot.x - getScale() / 100 * pot.left) + "mm";
    potEl.style.top = getZoom() / 100 * (pot.y - getScale() / 100 * pot.top) + "mm";
    potEl.style.width = getZoom() / 100 * getScale() / 100 * (pot.left + pot.right) + "mm";
    potEl.style.height = getZoom() / 100 * getScale() / 100 * (pot.top + pot.bottom) + "mm";
    potEl.classList.add("is-" + pot.stage.toLowerCase());
  }
}

const refresh = () => {
  refreshPots();
  document.querySelector("#zoom-percentage").innerHTML = getZoom() + "%";
  document.querySelector("#scale-percentage").innerHTML = getScale() + "%";
  document.querySelector(".no-pot").innerHTML = Object.keys(getPots()).length + "";
  document.querySelector(".no-young").innerHTML = Object.values(getPots()).filter((e) => e.stage == "Young").length + "";
  document.querySelector(".no-ready").innerHTML = Object.values(getPots()).filter((e) => e.stage == "Ready").length + "";
  document.querySelector(".no-old").innerHTML = Object.values(getPots()).filter((e) => e.stage == "Old").length + "";
}

const zoomOut = () => {
  const val = getZoom();
  setZoom(val > 20 ? val - 10 : val);
  refresh();
}
const zoomIn = () => {
  const val = getZoom();
  setZoom(val < 500 ? val + 10 : val);
  refresh();
}
const zoomReset = () => {
  setZoom(100);
  refresh();
}
const scaleOut = () => {
  const val = getScale();
  setScale(val > 20 ? val - 10 : val);
  refresh();
}
const scaleIn = () => {
  const val = getScale();
  setScale(val < 500 ? val + 10 : val);
  refresh();
}
const scaleReset = () => {
  setScale(100);
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

  let pot = getCurrentPot();
  if (pot) {
    document.querySelector("#detail > .tabs > ul > li.pot-tab > a").innerHTML = getPotId(pot.x, pot.y);
    document.querySelector(".pot-position").innerHTML = '(' + pot.x + ', ' + pot.y + ')';
    document.querySelector(".pot-size").innerHTML = '(' + (pot.left + pot.right) + ', ' + (pot.top + pot.bottom) + ')';
    document.querySelector(".pot-stage").innerHTML = pot.stage;
  }
}
const showDetailLog = () => {
  showDetail();
  resetDetail();
  document.querySelector("#detail").classList.add("show-detail-log")
}

const openDetailPot = (ev) => {
  ev.stopPropagation();
  let potId = ev.target.parentElement.id;
  setCurrentPot(getPot(potId));
  showDetailPot();
}


// Pushing
const pushMsg = async (msg) => {
  await fetch('/push', {
    method: 'POST',
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(msg),
  });
}
const manualWater = async () => {
  pushMsg({
    'ManualWater': {
      'x': getCurrentPot().x,
      'y': getCurrentPot().y
    }
  });
}
const manualCheck = async () => {
  pushMsg({
    'ManualCheck': {
      'x': getCurrentPot().x,
      'y': getCurrentPot().y
    }
  });
}

const startAutoWater = async () => {
  await pushMsg({
    'AutoWater': true
  });
}
const startAutoCheck = async () => {
  await pushMsg({
    'AutoCheck': true
  });
}
const stopAutoWater = async () => {
  await pushMsg({
    'AutoWater': false
  });
}
const stopAutoCheck = async () => {
  await pushMsg({
    'AutoCheck': false
  });
}
const getAllPot = async () => {
  await pushMsg(
    'GetListPot'
  );
}

// handling
//

const reportPot = (pot) => {
  if (pot) {
    pot.top = 20;
    pot.left = 20;
    pot.bottom = 20;
    pot.right = 20;
    pot.stage = "Unknown";
    addPot(pot);
    refresh()
  }
}
const reportCheck = (pot) => {
  if (pot) {
    pot.top = 20;
    pot.left = 20;
    pot.bottom = 20;
    pot.right = 20;
    pot.stage = "Unknown";
    addPot(pot);
    refresh()
  }
}
const reportWater = (pot) => {
  if (pot) {
    pot.top = 20;
    pot.left = 20;
    pot.bottom = 20;
    pot.right = 20;
    pot.stage = "Unknown";
    addPot(pot);
    refresh()
  }
}

const reportStatus = (s) => {
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

const reportErr = (s) => {
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
  reportPot(msg.ReportPot);

  reportCheck(msg.ReportCheck);
  reportWater(msg.ReportWater);
  reportErr(msg.Error);
  reportStatus(msg.Status);

  refresh();
};

getAllPot();
