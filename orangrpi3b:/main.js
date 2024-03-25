const getPotId = (x, y) => "pot-" + x + "-" + y;

const addPot = (potId, potInfo) => {
  if (document.querySelector(".pot." + potId)) {
    return;
  }

  let potCard = document.getElementById("pot-template").content.cloneNode(true).querySelector(".pot-card");
  document.querySelector(".pot-container").appendChild(potCard);

  let potTab = document.getElementById("pot-tab-template").content.cloneNode(true).querySelector(".side-pane-tab");
  document.querySelector(".side-pane-tab-bar > ul").appendChild(potTab);

  let potDetail = document.getElementById("pot-detail-template").content.cloneNode(true).querySelector("table.side-pane-table");
  document.querySelector(".side-pane-content").appendChild(potDetail);

  Array.from([potCard, potTab, potDetail]).forEach((pot) => {
    pot.classList.add(potId);
  })

  let x = potInfo["x"];
  let y = potInfo["y"];

  potCard.setAttribute("pot-x", x + "");
  potCard.setAttribute("pot-y", y + "");
  potCard.setAttribute("pot-height", "20");
  potCard.setAttribute("pot-width", "20");

  let zoom = Number(document.querySelector(".zoom-reset-button").innerHTML);
  potCard.style.left = zoom * x / 100 + "mm";
  potCard.style.top = y * zoom / 100 + "mm";
  potCard.style.width = "20mm";
  potCard.style.height = "20mm";

  let potOverlay = potCard.querySelector(".pot-card-overlay");
  potOverlay.setAttribute("onclick", "activatePot('" + potId + "')");
  potOverlay.setAttribute("onmouseover", "hoverPot('" + potId + "')");
  potOverlay.setAttribute("onmouseout", "leavePot('" + potId + "')");

  let potTabA = potTab.querySelector("a");
  potTab.querySelector("a").innerHTML = "Pot (" + x + ", " + y + ")";
  potTabA.setAttribute("onclick", "activatePot('" + potId + "')");

  potDetail.querySelector(".pot-position").innerHTML = "(" + x + ", " + y + ")";
  potDetail.querySelector(".pot-size").innerHTML = "20 &times; 20";

  let potWaterNow = potDetail.querySelector("a.manual-water");
  let potCheckNow = potDetail.querySelector("a.manual-check");
  potWaterNow.setAttribute("onclick", "manualWater(" + x + ", " + y + ")");
  potCheckNow.setAttribute("onclick", "manualCheck(" + x + ", " + y + ")");

  document.querySelector(".no-pot").innerHTML = document.querySelectorAll(".pot-card").length;
  document.querySelector(".no-unknown-pot").innerHTML = document.querySelectorAll(".pot-card.is-unknown").length;
  document.querySelector(".no-young-pot").innerHTML = document.querySelectorAll(".pot-card.is-young").length;
  document.querySelector(".no-ready-pot").innerHTML = document.querySelectorAll(".pot-card.is-ready").length;
  document.querySelector(".no-old-pot").innerHTML = document.querySelectorAll(".pot-card.is-old").length;
  setZoom(zoom);
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
const activatePot = (potId) => {
  document.body.classList.add("show-detail");

  document.querySelectorAll(".pot-card.is-active").forEach((el) => el.classList.remove("is-active"));
  document.querySelector(".side-pane-tab.is-active").classList.remove("is-active");
  document.querySelector(".side-pane-table.is-active").classList.remove("is-active");

  document.querySelectorAll("." + potId).forEach((el) => {
    el.classList.add("is-active");
    el.scrollIntoView(true);
  });
}

const hoverPot = (potId) => {
  let card = document.querySelector(".pot-card." + potId);
  let image = card.getAttribute("pot-image");
  card.style["background-image"] = 'url("' + image + '")'
}
const leavePot = (potId) => {
  let card = document.querySelector(".pot-card." + potId);
  card.style["background-image"] = null;
}

const setZoom = (percentage) => {
  document.querySelector(".zoom-out-button")
    .setAttribute("onclick", "setZoom(" + (percentage > 20 ? percentage - 10 : 20) + ")");
  document.querySelector(".zoom-in-button")
    .setAttribute("onclick", "setZoom(" + (percentage < 500 ? percentage + 10 : 500) + ")");
  document.querySelector(".zoom-reset-button")
    .setAttribute("onclick", "setZoom(100)");
  document.querySelector(".zoom-reset-button")
    .innerHTML = percentage + "";

  let maxHeight = 0;
  document.querySelectorAll(".pot-card").forEach((el) => {
    let hei = Number(el.getAttribute("pot-y"));
    if (hei > maxHeight) {
      maxHeight = hei;
    }
  });

  document.querySelectorAll(".pot-card").forEach((card) => {
    let x = Number(card.getAttribute("pot-x"));
    let y = Number(card.getAttribute("pot-y"));
    card.style.left = x * percentage / 100 + "mm";
    card.style.top = (maxHeight - y) * percentage / 100 + "mm";
  });

}
const setScale = (percentage) => {
  document.querySelector(".scale-out-button")
    .setAttribute("onclick", "setScale(" + (percentage > 20 ? percentage - 10 : 20) + ")");
  document.querySelector(".scale-in-button")
    .setAttribute("onclick", "setScale(" + (percentage < 500 ? percentage + 10 : 500) + ")");
  document.querySelector(".scale-reset-button")
    .setAttribute("onclick", "setScale(100)");
  document.querySelector(".scale-reset-button")
    .innerHTML = percentage + "";

  document.querySelectorAll(".pot-card").forEach((card) => {
    let height = Number(card.getAttribute("pot-height"));
    let width = Number(card.getAttribute("pot-width"));
    card.style.height = height * percentage / 100 + "mm";
    card.style.width = width * percentage / 100 + "mm";
  });
}

const waterCards = (stage) => {
  document.querySelectorAll(".pot-card" + (stage ? ".is-" + stage : "")).forEach((card) => {
    let x = Number(card.getAttribute("pot-x"));
    let y = Number(card.getAttribute("pot-y"));
    manualWater(x, y);
  })
}

const checkCards = (stage) => {
  document.querySelectorAll(".pot-card" + (stage ? ".is-" + stage : "")).forEach((card) => {
    let x = Number(card.getAttribute("pot-x"));
    let y = Number(card.getAttribute("pot-y"));
    manualCheck(x, y);
  })
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


    card.classList.remove("is-young");
    card.classList.remove("is-ready");
    card.classList.remove("is-old");
    card.classList.remove("is-unknown");
    card.classList.add("is-" + pot.stage.toLowerCase());

    card.setAttribute("pot-x", pot.x + "");
    card.setAttribute("pot-y", pot.y + "");
    card.setAttribute("pot-height", pot.top + pot.bottom + "");
    card.setAttribute("pot-width", pot.left + pot.right + "");

    let scale = Number(document.querySelector(".scale-reset-button").innerHTML);

    card.style.width = scale * (pot.left + pot.right) / 100 + "mm";
    card.style.height = scale * (pot.top + pot.bottom) / 100 + "mm";
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
    document.querySelector(".no-unknown-pot").innerHTML = document.querySelectorAll(".pot-card.is-unknown").length;
    document.querySelector(".no-young-pot").innerHTML = document.querySelectorAll(".pot-card.is-young").length;
    document.querySelector(".no-ready-pot").innerHTML = document.querySelectorAll(".pot-card.is-ready").length;
    document.querySelector(".no-old-pot").innerHTML = document.querySelectorAll(".pot-card.is-old").length;
    document.querySelector(".side-pane-table." + potId + " .pot-last-image").src = ("data:image/jpg;base64," + pot.image);
    document.querySelector(".pot-card." + potId).setAttribute("pot-image", ("data:image/jpg;base64," + pot.image));
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
    // let potId = getPotId(pot.x, pot.y);
    // addPot(potId, pot);
    // document.querySelector(".side-pane-table." + potId + " .pot-last-image").src = pot.file_path;
    // document.querySelector(".pot-card." + potId).setAttribute("pot-image", pot.file_path);
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
const pullMsgs = async () => {
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
const manualWater = async (x, y) => {
  await pushMsg({
    'Water': {
      'x': x,
      'y': y,
    }
  });
}
const manualCheck = async (x, y) => {
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
pullMsgs();
