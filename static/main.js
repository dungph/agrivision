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
  if (pot) {
    pot.top = pot.top ? pot.top : 20;
    pot.bottom = pot.bottom ? pot.bottom : 20;
    pot.left = pot.left ? pot.left : 20;
    pot.right = pot.right ? pot.right : 20;
    pot.stage = pot.stage ? pot.stage : 'Unknown';
    setPot(pot);
    //setCurrentPot(pot);
  }
}

const getData = (key, defaultValue) => localStorage[key] ? JSON.parse(localStorage[key]) : defaultValue;
const setData = (key, value) => {
  localStorage[key] = JSON.stringify(value);
  refresh();
}

const refreshPots = () => {
  let offset_x = 0;
  let offset_y = 0;
  let height = 0;
  for (let potId in getPots()) {
    const pot = getPot(potId);
    const offset_left_x = pot.x - pot.left;
    const offset_x_tmp = offset_left_x < 0 ? -offset_left_x : 0;
    if (offset_x_tmp > offset_x) {
      offset_x = offset_x_tmp;
    }
    const offset_top_y = pot.y - pot.top;
    const offset_y_tmp = offset_top_y < 0 ? -offset_top_y : 0;
    if (offset_y_tmp > offset_y) {
      offset_y = offset_y_tmp;
    }
    if (pot.y > height) {
      height = pot.y;
    }
  }
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
    potEl.style.left = offset_x + 10 + getData("zoom", 100) / 100 * (pot.x - getData("scale", 100) / 100 * pot.left) + "mm";
    potEl.style.top = offset_y + 10 + getData("zoom", 100) / 100 * (height - pot.y - getData("scale", 100) / 100 * pot.top) + "mm";
    potEl.style.width = getData("zoom", 100) / 100 * getData("scale", 100) / 100 * (pot.left + pot.right) + "mm";
    potEl.style.height = getData("zoom", 100) / 100 * getData("scale", 100) / 100 * (pot.top + pot.bottom) + "mm";
    potEl.classList.remove("is-young");
    potEl.classList.remove("is-ready");
    potEl.classList.remove("is-old");
    potEl.classList.remove("is-unknown");
    potEl.classList.add("is-" + pot.stage.toLowerCase());
  }

}

const refresh = () => {
  refreshPots();

  if (getData("show-detail", false)) {
    document.body.classList.add("show-detail");
  } else {
    document.body.classList.remove("show-detail");
  }

  document.querySelector("#detail").classList.remove("show-detail-pot");
  document.querySelector("#detail").classList.remove("show-detail-log");
  document.querySelector("#detail").classList.remove("show-detail-general");
  document.querySelector("#detail")
    .classList
    .add(getData("show-detail-type", "show-detail-general"))


  let pot = getData("current-pot", Object.values(getPots())[0]);
  if (pot) {
    document.querySelector("#detail > .tabs > ul > li.pot-tab > a").innerHTML = getPotId(pot.x, pot.y);
    document.querySelector(".pot-position").innerHTML = '(' + pot.x + ', ' + pot.y + ')';
    document.querySelector(".pot-size").innerHTML = '(' + (pot.left + pot.right) + ', ' + (pot.top + pot.bottom) + ')';
    document.querySelector(".pot-stage").innerHTML = pot.stage;

    document.querySelector(".pot-last-check").innerHTML = new Date(pot.lastCheck * 1000);
    document.querySelector(".pot-last-water").innerHTML = new Date(pot.lastWater * 1000);
  }

  document.querySelector("#zoom-percentage").innerHTML = getData("zoom", 100) + "%";
  document.querySelector("#scale-percentage").innerHTML = getData("scale", 100) + "%";
  document.querySelector(".no-pot").innerHTML = Object.keys(getPots()).length + "";
  document.querySelector(".no-young").innerHTML = Object.values(getPots()).filter((e) => e.stage == "Young").length + "";
  document.querySelector(".no-ready").innerHTML = Object.values(getPots()).filter((e) => e.stage == "Ready").length + "";
  document.querySelector(".no-old").innerHTML = Object.values(getPots()).filter((e) => e.stage == "Old").length + "";

  document.querySelector(".auto-water-state").innerHTML = getData("auto-water", false) ? "On" : "Off";
  document.querySelector(".auto-water-switch").innerHTML = getData("auto-water", false) ? "Disable" : "Enable";

  document.querySelector(".auto-check-state").innerHTML = getData("auto-check", false) ? "On" : "Off";
  document.querySelector(".auto-check-switch").innerHTML = getData("auto-check", false) ? "Disable" : "Enable";

  document.querySelector(".watering-state").innerHTML = getData("watering", false) ? "Watering" : "Off";
  document.querySelector(".capturing-state").innerHTML = getData("capturing", false) ? "Capturing" : "Idle";
  document.querySelector(".moving-state").innerHTML = getData("moving", false) ? "Moving" : "Idle";
}

const zoomOut = () => {
  const val = getData("zoom", 100);
  setData("zoom", val > 20 ? val - 10 : val);
}
const zoomIn = () => {
  const val = getData("zoom", 100);
  setData("zoom", val < 500 ? val + 10 : val);
}
const zoomReset = () => {
  setData("zoom", 100);
}
const scaleOut = () => {
  const val = getData("scale", 100);
  setData("scale", val > 20 ? val - 10 : val);
}
const scaleIn = () => {
  const val = getData("scale", 100);
  setData("scale", val < 500 ? val + 10 : val);
}
const scaleReset = () => {
  setData("scale", 100);
}

const hideDetail = () => {
  setData("show-detail", false);
}
const showDetail = () => {
  setData("show-detail", true);
}
const toggleDetail = () => {
  setData("show-detail", !getData("show-detail", false));
}
const showDetailGeneral = () => {
  showDetail();
  setData("show-detail-type", "show-detail-general");
}
const showDetailPot = () => {
  showDetail();
  setData("show-detail-type", "show-detail-pot");
}
const showDetailLog = () => {
  showDetail();
  setData("show-detail-type", "show-detail-log");
}

const hoverPot = (ev) => {
  ev.stopPropagation();
  let potId = ev.target.parentElement.id;
  document.querySelector("#" + potId).style["background-image"] = 'url("' + getPot(potId).image + '")';
}

const leavePot = (ev) => {
  ev.stopPropagation();
  let potId = ev.target.parentElement.id;
  document.querySelector("#" + potId).style["background-image"] = null;
}

const openDetailPot = (ev) => {
  ev.stopPropagation();
  let potId = ev.target.parentElement.id;
  setData("current-pot", getPot(potId));

  showDetailPot();
  document.querySelector(".pot-image").src = getPot(potId).image;
}


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
const manualWater = async () => {
  pushMsg({
    'Water': {
      'x': getData("current-pot", Object.values(getPots())[0]).x,
      'y': getData("current-pot", Object.values(getPots())[0]).y,
    }
  });
}
const manualCheck = async () => {
  pushMsg({
    'Check': {
      'x': getData("current-pot", Object.values(getPots())[0]).x,
      'y': getData("current-pot", Object.values(getPots())[0]).y,
    }
  });
}
const switchAutoWater = async () => {
  await pushMsg({
    'SetAutoWater': !getData("auto-water", false)
  });
}

const switchAutoCheck = async () => {
  await pushMsg({
    'SetAutoCheck': !getData("auto-check", false)
  });
}

const getReport = async () => {
  await pushMsg(
    'GetReport'
  );
}

// handling
//

const reportPot = (pot) => {
  if (pot) {
    if (getPot(getPotId(pot.x, pot.y))) {
    } else {
      addPot(pot);
    }
  }
}
const reportCheck = (pot) => {
  if (pot) {
    let oldPot = getPot(getPotId(pot.x, pot.y));
    if (oldPot) {
      oldPot.top = pot.top;
      oldPot.left = pot.left;
      oldPot.bottom = pot.bottom;
      oldPot.right = pot.right;
      oldPot.stage = pot.stage;
      oldPot.lastCheck = pot.timestamp;
    }
    addPot(oldPot);
  }
}
const reportWater = (pot) => {
  if (pot) {
    let oldPot = getPot(getPotId(pot.x, pot.y));
    if (oldPot) {
    } else {
      oldPot = pot;
    }
    oldPot.lastWater = pot.timestamp;
    addPot(oldPot);
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

const reportAutoWater = (a) => {
  if (a != null) {
    setData("auto-water", a);
  }
}
const reportAutoCheck = (a) => {
  if (a != null) {
    setData("auto-check", a);
  }
}
const reportWatering = (a) => {
  if (a != null) {
    setData("watering", a);
  }
}
const reportCapturing = (a) => {
  if (a != null) {
    setData("capturing", a);
  }
}
const reportMoving = (a) => {
  if (a != null) {
    setData("moving", a);
  }
}
const reportImagePath = (p) => {
  if (p) {
    setData("image-path", a);
    refresh();
  }
}
const reportPotImagePath = (p) => {
  if (p) {
    let pot = getPot(getPotId(p.x, p.y));
    if (pot) {
      pot.image = p.file_path;
      setPot(pot);
    } else {
      p.image = p.file_path;
      addPot(p);
    }
    
    refresh();
  }
}
new EventSource("/pull").onmessage = (event) => {
  console.log("Pulling ", event.data);
  let msg = JSON.parse(event.data);
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

  reportErr(msg.Error);
  reportStatus(msg.Status);


  refresh();
};

getReport();
