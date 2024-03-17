const getPotId = (x, y) => "pot-" + x + "-" + y;

const getData = (key, defaultValue) => localStorage[key] ? JSON.parse(localStorage[key]) : defaultValue;
const setData = (key, value) => {
  localStorage[key] = JSON.stringify(value);
}

const addPot = (potId, potInfo) => {
  if (document.querySelector(".pot." + potId)) {
    return;
  }

  let potTemplate = document.getElementById("pot-template").content.cloneNode(true).querySelector(".pot-card");
  let potOverlay = potTemplate.querySelector(".pot-card-overlay");
  document.querySelector(".pot-container").appendChild(potTemplate);

  let potTab = document.getElementById("pot-tab-template").content.cloneNode(true).querySelector(".side-pane-tab");
  let potTabA = potTab.querySelector("a");
  document.querySelector(".side-pane-tab-bar > ul").appendChild(potTab);

  let potDetail = document.getElementById("pot-detail-template").content.cloneNode(true).querySelector("table.side-pane-table");
  let potWaterNow = potDetail.querySelector("a.manual-water");
  let potCheckNow = potDetail.querySelector("a.manual-check");
  document.querySelector(".side-pane-content").appendChild(potDetail);

  Array.from([potWaterNow, potCheckNow]).forEach((pot) => {
    pot.setAttribute("pot-id", potId);
    pot.setAttribute("pot-x", potInfo["x"]);
    pot.setAttribute("pot-y", potInfo["y"]);
  })
  Array.from([potTemplate, potTab, potDetail, potOverlay, potTabA]).forEach((pot) => {
    pot.classList.add(potId);
    pot.setAttribute("pot-id", potId);
  })

  let x = potInfo["x"];
  let y = potInfo["y"];

  potTemplate.style.left = x + "mm";
  potTemplate.style.top = y + "mm";

  potTemplate.style.width = "20mm";
  potTemplate.style.height = "20mm";

  potTab.querySelector("a").innerHTML = "Pot (" + x + ", " + y + ")";

  potDetail.querySelector(".pot-position").innerHTML = "(" + x + ", " + y + ")";
  potDetail.querySelector(".pot-size").innerHTML = "20 &times; 20";
}

const deactivatePane = () => {
  document.body.classList.remove("show-detail");
}


const togglePane = () => {
  document.body.classList.toggle("show-detail");
}

const activateGeneral = () => {
  document.body.classList.add("show-detail");
  document.querySelector(".side-pane-tab.is-active").classList.remove("is-active");
  document.querySelector(".side-pane-table.is-active").classList.remove("is-active");
  document.querySelector(".side-pane-tab.general").classList.add("is-active");
  document.querySelector(".side-pane-table.general").classList.add("is-active");
  document.querySelector(".side-pane-tab").scrollIntoView(true);
  document.querySelectorAll(".pot-card.is-active").forEach((el) => el.classList.remove("is-active"));
}
const activatePot = (event) => {
  event.stopPropagation();
  document.body.classList.add("show-detail");
  let potId = event.target.getAttribute("pot-id");

  document.querySelectorAll(".pot-card.is-active").forEach((el) => el.classList.remove("is-active"));
  document.querySelector(".side-pane-tab.is-active").classList.remove("is-active");
  document.querySelector(".side-pane-table.is-active").classList.remove("is-active");

  document.querySelectorAll("." + potId).forEach((el) => {
    el.classList.add("is-active");
    el.scrollIntoView(true);
  });
}

const hoverPot = (event) => {
  event.stopPropagation();
  let potId = event.target.getAttribute("pot-id");
  let card = document.querySelector(".pot-card." + potId);
  let image = card.getAttribute("pot-image");
  card.style["background-image"] = 'url("' + image + '")'
}
const leavePot = (event) => {
  event.stopPropagation();
  let potId = event.target.getAttribute("pot-id");
  let card = document.querySelector(".pot-card." + potId);
  card.style["background-image"] = null;
}

const zoomIn = (event) => {
  if (localStorage["zoom"]) {

  }
}

// handling
//
const reportPot = (pot) => {
  if (pot) {
    let potId = getPotId(pot.x, pot.y);
    addPot(potId, pot);
  }
}
const reportCheck = (pot) => {
  if (pot) {
    let potId = getPotId(pot.x, pot.y);
    addPot(potId, pot);
    let card = document.querySelector(".pot-card." + potId);

    card.setAttribute("pot-top", pot.top + "");
    card.setAttribute("pot-left", pot.left + "");
    card.setAttribute("pot-bottom", pot.bottom + "");
    card.setAttribute("pot-right", pot.right + "");

    card.classList.remove("is-young");
    card.classList.remove("is-ready");
    card.classList.remove("is-old");
    card.classList.remove("is-unknown");
    card.classList.add("is-" + pot.stage.toLowerCase());
    card.style.width = pot.left + pot.right + "mm";
    card.style.height = pot.top + pot.bottom + "mm";
    let detail = document.querySelector(".side-pane-table." + potId);
    detail.querySelector(".pot-stage").innerHTML = pot.stage;

    let date = new Date(pot.timestamp * 1000);
    detail.querySelector(".pot-last-check").innerHTML =
      date.getDate() + "/" +
      date.getUTCMonth() + "/" +
      date.getFullYear() + " " +
      date.getHours() + ":" +
      date.getMinutes() + ":" +
      date.getSeconds();

    document.querySelector(".no-pot").innerHTML = document.querySelectorAll(".pot-card").length;
    document.querySelector(".no-young").innerHTML = document.querySelectorAll(".pot-card.is-young").length;
    document.querySelector(".no-ready").innerHTML = document.querySelectorAll(".pot-card.is-ready").length;
    document.querySelector(".no-old").innerHTML = document.querySelectorAll(".pot-card.is-old").length;
  }
}
const reportWater = (pot) => {
  if (pot) {
    let potId = getPotId(pot.x, pot.y);
    addPot(potId, pot);
    let detail = document.querySelector(".side-pane-table." + potId);
    let date = new Date(pot.timestamp * 1000);
    detail.querySelector(".pot-last-water").innerHTML =
      date.getDate() + "/" +
      date.getUTCMonth() + "/" +
      date.getFullYear() + " " +
      date.getHours() + ":" +
      date.getMinutes() + ":" +
      date.getSeconds();
  }
}
const reportPotImagePath = async (pot) => {
  if (pot) {
    let potId = getPotId(pot.x, pot.y);
    addPot(potId, pot);
    document.querySelector(".side-pane-table." + potId + " .pot-last-image").src = pot.file_path;
    document.querySelector(".pot-card." + potId).setAttribute("pot-image", pot.file_path);
  }
}
const reportAutoWater = (a) => {
  if (a != null) {
    document.querySelector(".auto-water-state").innerHTML = a ? "On" : "Off";
    document.querySelector(".auto-water-switch").innerHTML = a ? "Disable" : "Enable";
  }
}
const reportAutoCheck = (a) => {
  if (a != null) {
    document.querySelector(".auto-check-state").innerHTML = a ? "On" : "Off";
    document.querySelector(".auto-check-switch").innerHTML = a ? "Disable" : "Enable";
  }
}
const reportWatering = (a) => {
  if (a != null) {
    document.querySelector(".watering-state").innerHTML = a ? "Watering" : "Idle";
  }
}
const reportCapturing = (a) => {
  if (a != null) {
    document.querySelector(".capturing-state").innerHTML = a ? "Capturing" : "Idle";
  }
}
const reportMoving = (a) => {
  if (a != null) {
    document.querySelector(".moving-state").innerHTML = a ? "Moving" : "Idle";
  }
}
const reportImagePath = (p) => {
  if (p) {
    document.querySelector(".camera-image").src = p;
  }
}
const pullMsg = async () => {
  while (true) {

    let res = await fetch('/wait');
    let msg = await res.json();
    console.log("Pulling ", msg);

    reportPot(msg.ReportPot);
    reportCheck(msg.ReportCheck);
    reportWater(msg.ReportWater);
    reportAutoWater(msg.ReportAutoWater);
    reportAutoCheck(msg.ReportAutoCheck);
    reportWatering(msg.ReportWatering);
    reportMoving(msg.ReportMoving);
    reportCapturing(msg.ReportCapturing);
    reportImagePath(msg.ReportImageFile);
    reportPotImagePath(msg.ReportPotImageFile);
  }
}
//new EventSource("/pull").onmessage = (event) => {
//  console.log("Pulling ", event.data);
//  let msg = JSON.parse(event.data);
//
//  reportPot(msg.ReportPot);
//  reportCheck(msg.ReportCheck);
//  reportWater(msg.ReportWater);
//  reportAutoWater(msg.ReportAutoWater);
//  reportAutoCheck(msg.ReportAutoCheck);
//  reportWatering(msg.ReportWatering);
//  reportMoving(msg.ReportMoving);
//  reportCapturing(msg.ReportCapturing);
//  reportImagePath(msg.ReportImageFile);
//  reportPotImagePath(msg.ReportPotImageFile);
//};


// Pushing
const pushMsg = async (msg) => {
  console.log("Pushing ", msg);
  await fetch('/push', {
    method: 'POST',
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(msg),
  });
}
const manualWater = async (event) => {
  let x = Number(event.target.getAttribute("pot-x"));
  let y = Number(event.target.getAttribute("pot-y"));
  await pushMsg({
    'Water': {
      'x': x,
      'y': y,
    }
  });
}
const manualCheck = async (event) => {
  let x = Number(event.target.getAttribute("pot-x"));
  let y = Number(event.target.getAttribute("pot-y"));
  await pushMsg({
    'Check': {
      'x': x,
      'y': y,
    }
  });
}
const switchAutoWater = async () => {
  let sw = document.querySelector(".auto-water-switch").innerHTML;
  await pushMsg({
    'SetAutoWater': sw === "Enable" ? true : false
  });
}

const switchAutoCheck = async () => {
  let sw = document.querySelector(".auto-check-switch").innerHTML;
  await pushMsg({
    'SetAutoCheck': sw === "Enable" ? true : false
  });
}

const getReport = async () => {
  await pushMsg(
    'GetReport'
  );
}

getReport();
pullMsg();
